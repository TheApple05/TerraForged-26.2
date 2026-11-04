"""Export pinned Horizon core/world sources and apply the actual native mixin bundle.

No game files are fetched and the source checkout is never modified. The output
is a small developer workspace, not a complete game distribution.
"""
from __future__ import annotations
import argparse
import hashlib
import io
import json
from pathlib import Path
import subprocess
import tarfile
import tomllib

ROOT = Path(__file__).resolve().parents[2]
PIN = "9dd5c4093977c718c5c0349032fd3a5434e86afc"


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--horizon", required=True, type=Path)
    parser.add_argument("--mixins", required=True, type=Path)
    parser.add_argument("--output", type=Path, default=ROOT / "build/qa/horizon")
    args = parser.parse_args()
    source = args.horizon.resolve()
    output = args.output.resolve()
    if not output.is_relative_to((ROOT / "build/qa/horizon").resolve()):
        parser.error("All staging paths must remain under build/qa/horizon")
    pristine = output / "pristine"
    if pristine.exists():
        parser.error("Choose a fresh staging output; existing sources are not overwritten")
    command = ["git", "-c", f"safe.directory={source.as_posix()}", "-C", str(source)]
    revision = subprocess.check_output(command + ["rev-parse", PIN], text=True).strip()
    if revision != PIN:
        parser.error("Pinned Horizon object is unavailable")
    archive = subprocess.check_output(command + ["archive", PIN, "crates/horizon-core", "crates/horizon-world", "LICENSE", "Cargo.lock"])
    pristine.mkdir(parents=True)
    with tarfile.open(fileobj=io.BytesIO(archive)) as tar:
        members = tar.getmembers()
        for member in members:
            path = (pristine / member.name).resolve()
            if not path.is_relative_to(pristine) or not (member.isfile() or member.isdir()):
                raise ValueError("Unsafe source archive member")
        tar.extractall(pristine, members=members)
    (pristine / "Cargo.toml").write_text('''[workspace]
members = ["crates/horizon-core", "crates/horizon-world"]
resolver = "2"
[workspace.package]
edition = "2024"
license = "MIT"
rust-version = "1.85"
[workspace.lints.clippy]
too_many_arguments = "allow"
type_complexity = "allow"
needless_range_loop = "allow"
large_enum_variant = "allow"
[profile.dev]
debug = 0
''', encoding="utf-8")
    (pristine / ".horizon-revision").write_text(PIN + "\n", encoding="utf-8")
    # Prune unrelated workspace members while retaining the actual pinned
    # Horizon dependency versions. Never resolve a newer dependency silently.
    original_lock = tomllib.loads((pristine / "Cargo.lock").read_text(encoding="utf-8"))
    resolved = subprocess.run(["cargo", "+stable", "update", "--offline", "--workspace", "--manifest-path", str(pristine / "Cargo.toml")], capture_output=True, text=True)
    (output / "dependency-pruning.log").write_text(resolved.stdout + resolved.stderr, encoding="utf-8")
    if resolved.returncode:
        raise RuntimeError("Could not retain pinned dependency versions: " + resolved.stderr[-2000:])
    selected_lock = tomllib.loads((pristine / "Cargo.lock").read_text(encoding="utf-8"))
    identity = lambda package: (package["name"], package["version"], package.get("source"))
    if not set(map(identity, selected_lock["package"])).issubset(set(map(identity, original_lock["package"]))):
        raise ValueError("Isolated workspace selected a dependency outside the pinned Horizon lock")
    before = sha(source / "crates/horizon-world/src/worldgen/generator.rs")
    mods = ROOT / "horizon/mixins"
    plan = output / "plan.json"
    subprocess.run([str(args.mixins), "plan", "--workspace", str(pristine), "--mods", str(mods), "--out", str(plan)], check=True)
    stage = output / "staged"
    subprocess.run([str(args.mixins), "apply", "--workspace", str(pristine), "--mods", str(mods), "--out", str(stage)], check=True)
    if before != sha(source / "crates/horizon-world/src/worldgen/generator.rs"):
        raise ValueError("Original Horizon source changed during staging")
    report = {"schema": 1, "horizonRevision": revision, "sourceCheckoutUnchanged": True,
              "sourceArchiveSha256": hashlib.sha256(archive).hexdigest(),
              "plan": str(plan), "planSha256": sha(plan), "stagedWorkspace": str(stage),
              "constructorCompatibilityCommit": PIN,
              "requiresPinnedHorizonSource": True,
              "cargoLockSha256": sha(stage / "Cargo.lock"),
              "allDependencyVersionsRetainedFromPinnedLock": True,
              "sourcePinIsNotFullRuntimeValidation": True}
    (output / "stage-result.json").write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(report, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
