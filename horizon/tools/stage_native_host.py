"""Assemble the qualified kernel as a real Horizon bundle crate, then stage it.

Source-only developer workspace with opt-in world integration; no game downloads.
The finite prepared-world experiment uses its separate unchanged staging tool.
"""
from __future__ import annotations
import argparse
import hashlib
import io
import json
from pathlib import Path
import shutil
import subprocess
import tarfile
import tomllib

ROOT = Path(__file__).resolve().parents[2]
PIN = "0e0f70ff1f8191479dbb5b1b173a74d38d324005"


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main() -> int:
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--horizon", required=True, type=Path)
    p.add_argument("--mixins", required=True, type=Path)
    p.add_argument("--output", required=True, type=Path)
    p.add_argument("--registry", action="store_true", help="Also stage constructor-scoped custom density resource decoding; no world activation")
    p.add_argument("--router", action="store_true", help="Also stage world-local router definition supply; no world activation")
    p.add_argument("--world", action="store_true", help="Also stage the explicit procedural pipeline factory for QA; no automatic world selection")
    p.add_argument("--entry", action="store_true", help="Also select the native pipeline for the explicit TerraForged preset/saved stem")
    p.add_argument("--full", action="store_true", help="Export the full pinned engine for package/client integration; never download game resources")
    a = p.parse_args()
    if a.entry:
        a.world = True
    if a.world:
        a.router = True
    if a.router:
        a.registry = True
    output = a.output.resolve()
    if not output.is_relative_to((ROOT / "build/qa/horizon").resolve()) or output.exists():
        p.error("Choose a new output below build/qa/horizon; no replacement is allowed")
    source = a.horizon.resolve()
    git = ["git", "-c", f"safe.directory={source.as_posix()}", "-C", str(source)]
    if subprocess.check_output(git + ["rev-parse", PIN], text=True).strip() != PIN:
        p.error("Exact host API commit is unavailable")
    archive = subprocess.check_output(git + ["archive", PIN] + ([] if a.full else ["crates/horizon-core", "crates/horizon-world", "LICENSE", "Cargo.lock"]))
    pristine = output / "pristine"
    pristine.mkdir(parents=True)
    with tarfile.open(fileobj=io.BytesIO(archive)) as tar:
        members = tar.getmembers()
        for member in members:
            if not (pristine / member.name).resolve().is_relative_to(pristine) or not (member.isfile() or member.isdir() or member.issym()):
                raise ValueError("Unsafe source member")
            if member.issym():
                target = (pristine / member.name).parent / member.linkname
                if not target.resolve().is_relative_to(pristine):
                    raise ValueError("Source link escapes export")
                linked = tar.getmember(target.resolve().relative_to(pristine).as_posix())
                if not linked.isfile():
                    raise ValueError("Source link must name an archived regular file")
        tar.extractall(pristine, members=[m for m in members if not m.issym()])
        # Materialize internal documentation links so Windows needs no symlink privilege.
        for member in members:
            if member.issym():
                target = ((pristine / member.name).parent / member.linkname).resolve()
                shutil.copy2(target, pristine / member.name)
    if not a.full:
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
''', encoding="utf-8", newline="\n")
    (pristine / ".horizon-revision").write_text(PIN + "\n", encoding="utf-8")
    original_lock = tomllib.loads((pristine / "Cargo.lock").read_text(encoding="utf-8"))
    run = subprocess.run(["cargo", "+stable", "metadata", "--offline", "--locked", "--no-deps", "--format-version", "1", "--manifest-path", str(pristine / "Cargo.toml")] if a.full else ["cargo", "+stable", "update", "--offline", "--workspace", "--manifest-path", str(pristine / "Cargo.toml")], capture_output=True, text=True)
    (output / "dependency-pruning.log").write_text(run.stdout + run.stderr, encoding="utf-8")
    if run.returncode:
        raise RuntimeError(run.stderr[-2000:])
    identity = lambda entry: (entry["name"], entry["version"], entry.get("source"), entry.get("checksum"))
    selected = tomllib.loads((pristine / "Cargo.lock").read_text(encoding="utf-8"))
    if not set(map(identity, selected["package"])).issubset(set(map(identity, original_lock["package"]))):
        raise ValueError("Dependency changed outside pinned host lock")
    mods = output / "mods"
    bundle = mods / "terraforged_native"
    shutil.copytree(ROOT / "horizon/native-host", bundle)
    if a.registry:
        for name in ("registry_scope.rs", "decode_density.rs"):
            shutil.copy2(ROOT / "horizon/native-world" / name, bundle / name)
        with (bundle / "horizon-mixin.toml").open("a", encoding="utf-8", newline="\n") as manifest:
            manifest.write('''
[[module]]
id = "registry-scope"
file = "crates/horizon-world/src/worldgen/generator.rs"
name = "terraforged_registry"
source = "registry_scope.rs"
visibility = "pub"

[[injection]]
id = "custom-density-decode"
target = "horizon_world::worldgen::density::Loader::decode"
kind = "HEAD"
cancellable = true
helper = "decode_density.rs"
bindings = [
  { name = "loader", ty = "&mut Loader", value = "self" },
  { name = "raw", ty = "&RawValue", value = "raw" },
  { name = "depth", ty = "usize", value = "depth" },
]
''')
    if a.world:
        shutil.copy2(ROOT / "horizon/native-world/world_runtime.rs", bundle / "world_runtime.rs")
        with (bundle / "horizon-mixin.toml").open("a", encoding="utf-8", newline="\n") as manifest:
            manifest.write('''
[[module]]
id = "native-world-pipeline"
file = "crates/horizon-world/src/worldgen/generator.rs"
name = "terraforged_world"
source = "world_runtime.rs"
visibility = "pub"
''')
    if a.entry:
        shutil.copy2(ROOT / "horizon/native-world/open_world.rs", bundle / "open_world.rs")
        with (bundle / "horizon-mixin.toml").open("a", encoding="utf-8", newline="\n") as manifest:
            manifest.write('''
[[injection]]
id = "native-world-selection"
target = "horizon_world::worldgen::generator::OverworldGenerator::for_world_with_settings"
kind = "HEAD"
cancellable = true
helper = "open_world.rs"
bindings = [
  { name = "data", ty = "&std::sync::Arc<GeneratorData>", value = "&data" },
  { name = "seed", ty = "i64", value = "seed" },
  { name = "preset", ty = "&str", value = "preset" },
  { name = "dimension", ty = "&str", value = "dimension" },
  { name = "with_structures", ty = "bool", value = "with_structures" },
  { name = "saved_stem", ty = "Option<&serde_json::Value>", value = "saved_stem" },
]
''')
    if a.router:
        shutil.copy2(ROOT / "horizon/native-world/supply_density.rs", bundle / "supply_density.rs")
        with (bundle / "horizon-mixin.toml").open("a", encoding="utf-8", newline="\n") as manifest:
            manifest.write('''
[[injection]]
id = "world-local-density-definitions"
target = "horizon_world::worldgen::density::Loader::resolve"
kind = "HEAD"
cancellable = true
helper = "supply_density.rs"
bindings = [
  { name = "loader", ty = "&mut Loader", value = "self" },
  { name = "name", ty = "&str", value = "name" },
]
''')
    kernel = bundle / "kernel"
    shutil.copytree(ROOT / "horizon/native-core/src", kernel / "src", ignore=shutil.ignore_patterns("bin", "data"))
    # Native packages allow data at a crate's top level, not under src/data.
    shutil.copytree(ROOT / "horizon/native-core/src/data", kernel / "data")
    for name in ("biome.rs", "math_table.rs"):
        path = kernel / "src" / name
        text = path.read_text(encoding="utf-8")
        if text.count('include_bytes!("data/') != 1:
            raise ValueError("Unexpected numerical table include: " + name)
        path.write_text(text.replace('include_bytes!("data/', 'include_bytes!("../data/'), encoding="utf-8", newline="\n")
    source_files = {}
    for file in sorted((ROOT / "horizon/native-core/src").rglob("*")):
        if file.is_file() and "bin" not in file.relative_to(ROOT / "horizon/native-core/src").parts:
            source_files[file.relative_to(ROOT).as_posix()] = sha(file)
    for name, destination in (("noise_registry.rs", "noise_registry.rs"), ("preset.rs", "preset_input.rs"), ("router.rs", "router.rs")):
        file = ROOT / "horizon/native-adapter/src" / name
        source_files[file.relative_to(ROOT).as_posix()] = sha(file)
        (kernel / "src" / destination).write_text(file.read_text(encoding="utf-8").replace("terraforged_native_core::", "crate::"), encoding="utf-8", newline="\n")
    if a.world:
        file = ROOT / "horizon/native-world/world_runtime.rs"
        source_files[file.relative_to(ROOT).as_posix()] = sha(file)
    with (kernel / "src/lib.rs").open("a", encoding="utf-8", newline="\n") as lib:
        lib.write("\npub mod noise_registry;\npub mod preset_input;\npub mod router;\npub use serde_json;\n")
    if a.entry:
        data_dir=kernel / "data"
        data_dir.mkdir(exist_ok=True)
        preset_source=ROOT / "horizon/tests/fixtures/kernel-v1/preset.json"
        noise_source=ROOT / "horizon/tests/fixtures/kernel-v1/noise-registry.json"
        shutil.copy2(preset_source,data_dir / "default-preset.json")
        noises=json.loads(noise_source.read_text(encoding="utf-8"))
        selected={name:noises[name] for name in ("reterraforged:terrain/ground","reterraforged:climate/biome_edge_shape")}
        (data_dir / "default-noises.json").write_text(json.dumps(selected,sort_keys=True,indent=2)+"\n",encoding="utf-8",newline="\n")
        source_files[preset_source.relative_to(ROOT).as_posix()]=sha(preset_source)
        source_files[noise_source.relative_to(ROOT).as_posix()]=sha(noise_source)
        with (kernel / "src/lib.rs").open("a", encoding="utf-8", newline="\n") as lib:
            lib.write('\npub fn builtin_preset_json() -> &\'static str { include_str!("../data/default-preset.json") }\n')
            lib.write('pub fn builtin_noises_json() -> &\'static str { include_str!("../data/default-noises.json") }\n')
    shutil.copy2(ROOT / "horizon/native-core/LICENSE-RETERRAFORGED", kernel / "LICENSE-RETERRAFORGED")
    shutil.copy2(ROOT / "horizon/LICENSE", kernel / "LICENSE")
    (kernel / "Cargo.toml").write_text('''[package]
name = "terraforged-kernel"
version = "0.1.0"
edition = "2024"
license = "MIT"
publish = false
[dependencies]
serde_json = { version = "=1.0.151", features = ["arbitrary_precision"] }
[lints]
workspace = true
''', encoding="utf-8", newline="\n")
    for file in sorted((ROOT / "horizon/native-host").rglob("*")):
        if file.is_file():
            source_files[file.relative_to(ROOT).as_posix()] = sha(file)
    if a.registry:
        for name in ("registry_scope.rs", "decode_density.rs"):
            file = ROOT / "horizon/native-world" / name
            source_files[file.relative_to(ROOT).as_posix()] = sha(file)
    if a.router:
        file = ROOT / "horizon/native-world/supply_density.rs"
        source_files[file.relative_to(ROOT).as_posix()] = sha(file)
    if a.entry:
        file = ROOT / "horizon/native-world/open_world.rs"
        source_files[file.relative_to(ROOT).as_posix()] = sha(file)
    plan = output / "plan.json"
    stage = output / "staged"
    for action, destination in (("plan", plan), ("apply", stage)):
        subprocess.run([str(a.mixins), action, "--workspace", str(pristine), "--mods", str(mods), "--out", str(destination)], check=True)
    report = {"schemaVersion": 1, "status": "source-staged-not-runtime-qualified", "horizonRevision": PIN,
              "hostSourceArchiveSha256": hashlib.sha256(archive).hexdigest(), "inputFiles": source_files,
              "planSha256": sha(plan), "cargoLockSha256": sha(stage / "Cargo.lock"),
              "stagedWorkspace": str(stage), "gameAssetsReadOrDownloaded": False,
              "worldActivationIncluded": a.world, "sourceCheckoutModified": False}
    report["registryDecoderIncluded"] = a.registry
    report["routerDefinitionSupplyIncluded"] = a.router
    report["automaticWorldSelectionIncluded"] = a.entry
    report["fullEngineSourceIncluded"] = a.full
    (output / "stage-result.json").write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8", newline="\n")
    print(json.dumps(report, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
