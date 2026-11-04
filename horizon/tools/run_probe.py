"""Compare actual native Horizon generated chunks with the Java snapshot corpus."""
from __future__ import annotations
import argparse
import datetime as dt
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[2]


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def expected_chunk(raw: dict) -> dict:
    block_hash = hashlib.sha256()
    count = 0
    for index, length in raw["blocks"]:
        value = raw["blockPalette"][index]
        text = value["name"] + ":" + json.dumps(value["properties"], sort_keys=True, separators=(",", ":"), ensure_ascii=False) + "\n"
        block_hash.update(text.encode() * length)
        count += length
    biome_hash = hashlib.sha256()
    biome_count = 0
    for index, length in raw["biomes"]:
        biome_hash.update((raw["biomePalette"][index] + "\n").encode() * length)
        biome_count += length
    return {"blockSha256": block_hash.hexdigest(), "quartBiomeSha256": biome_hash.hexdigest(),
            "blockCount": count, "quartBiomeCount": biome_count, "surfaceColumnsChecked": 256}


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--stage", required=True, type=Path)
    parser.add_argument("--resources", required=True, type=Path)
    parser.add_argument("--snapshot", required=True, type=Path)
    parser.add_argument("--biomes", required=True, type=Path)
    parser.add_argument("--expected-mod-sha256", required=True)
    parser.add_argument("--build-only", action="store_true", help="Compile the real native probe before a Java snapshot is available")
    parser.add_argument("--report", type=Path, default=ROOT / "docs/validation/horizon-prepared-native-probe.json")
    args = parser.parse_args()
    started = dt.datetime.now(dt.timezone.utc).isoformat()
    stage = args.stage.resolve()
    if not stage.is_relative_to((ROOT / "build/qa/horizon").resolve()):
        parser.error("Probe builds must remain under build/qa/horizon")
    work = Path(tempfile.mkdtemp(prefix="probe-", dir=stage.parent))
    source = ROOT / "horizon/tests/prepared_probe.rs"
    probe = stage / "crates/horizon-world/examples/terraforged_prepared_probe.rs"
    probe.parent.mkdir(exist_ok=True)
    shutil.copy2(source, probe)
    base = ["cargo", "+stable", "--color", "never"]
    cargo_manifest = stage / "Cargo.toml"
    command = base + ["build", "--locked", "--manifest-path", str(cargo_manifest), "-p", "horizon-world", "--example", "terraforged_prepared_probe"]
    build = subprocess.run(command, cwd=ROOT, capture_output=True, text=True, encoding="utf-8", errors="replace", timeout=180)
    (work / "build.log").write_text(build.stdout + build.stderr, encoding="utf-8")
    if build.returncode:
        print(build.stdout + build.stderr)
        return build.returncode
    executable = stage / "target/debug/examples" / ("terraforged_prepared_probe.exe" if os.name == "nt" else "terraforged_prepared_probe")
    if args.build_only:
        print(json.dumps({"status": "compiled-only", "executable": str(executable), "sha256": digest(executable), "runtimeExecuted": False}))
        return 0
    manifest_path = args.snapshot.resolve() / "terraforged-prepared.json"
    manifest = json.loads(manifest_path.read_text())
    if manifest["sourceModSha256"] != args.expected_mod_sha256:
        parser.error("Snapshot was not prepared by the expected production mod artifact")
    environment = os.environ.copy()
    environment.update({"TERRAFORGED_HORIZON_PREPARED": str(args.snapshot.resolve()),
                        "HORIZON_HOME": str(work / "home"), "HORIZON_LAUNCHER_HOME": str(work / "launcher"),
                        "XDG_DATA_HOME": str(work / "data"), "XDG_CONFIG_HOME": str(work / "config"),
                        "HORIZON_NO_AUDIO": "1", "HORIZON_LAUNCHER_NO_SELF_UPDATE": "1"})
    result = subprocess.run([str(executable), str(args.resources.resolve()), str(args.snapshot.resolve()), str(args.biomes.resolve())],
                            env=environment, cwd=work, capture_output=True, text=True, encoding="utf-8", errors="replace", timeout=180)
    (work / "stdout.json").write_text(result.stdout, encoding="utf-8")
    (work / "stderr.log").write_text(result.stderr, encoding="utf-8")
    report = {"schema": 1, "status": "failed", "startedAt": started, "finishedAt": dt.datetime.now(dt.timezone.utc).isoformat(),
              "processExit": result.returncode, "nativeExecutableSha256": digest(executable), "probeSourceSha256": digest(source),
              "driverSha256": digest(Path(__file__)),
              "stagedGeneratorSha256": digest(stage / "crates/horizon-world/src/worldgen/generator.rs"),
              "nativeBundleSourceSha256": {path.name: digest(path) for path in sorted((ROOT / "horizon/mixins/terraforged_prepared").iterdir()) if path.is_file()},
              "horizonRevision": "9dd5c4093977c718c5c0349032fd3a5434e86afc", "minecraftNative": "26.3", "minecraftSource": "26.2",
              "javaModSha256": args.expected_mod_sha256, "manifestSha256": digest(manifest_path),
              "work": str(work), "resourcesWereLocalOnly": True,
              "rustc": subprocess.check_output(["rustc", "+stable", "--version"], text=True).strip(),
              "cargoLockSha256": digest(stage / "Cargo.lock"),
              "resourcesManifestSha256": digest(args.resources / "horizon-resources.json")}
    if result.returncode:
        report["error"] = result.stderr[-6000:]
    else:
        native = json.loads(result.stdout)
        expected = {}
        for key, sha in manifest["chunks"].items():
            x, z = key.split(",")
            path = args.snapshot / "chunks" / f"{x}.{z}.json"
            if digest(path) != sha:
                raise ValueError("Snapshot checksum changed during native probe")
            expected[key] = expected_chunk(json.loads(path.read_text()))
        if native["chunks"] != expected:
            report["error"] = "Generated native blocks/biomes differ from the Java corpus"
            report["expected"] = expected
        else:
            # Exercise the actual native decoder, including a checksum-valid
            # unknown biome. No fallback may make either malformed corpus pass.
            expected_failures = []
            sx, sz = (manifest["spawn"][0] >> 4, manifest["spawn"][1] >> 4)
            key = f"{sx},{sz}"
            relative = Path("chunks") / f"{sx}.{sz}.json"
            for case, message in [("unknown-biome", "prepared biome absent from Horizon registry"),
                                  ("corrupt-chunk", "prepared chunk checksum mismatch")]:
                case_root = work / case
                (case_root / "chunks").mkdir(parents=True)
                altered_manifest = dict(manifest)
                altered_manifest["chunks"] = dict(manifest["chunks"])
                source_chunk = args.snapshot / relative
                if case == "unknown-biome":
                    raw = json.loads(source_chunk.read_text())
                    raw["biomePalette"][0] = "terraforged:missing_native_test_biome"
                    (case_root / relative).write_text(json.dumps(raw), encoding="utf-8")
                    altered_manifest["chunks"][key] = digest(case_root / relative)
                else:
                    (case_root / relative).write_bytes(source_chunk.read_bytes() + b" ")
                (case_root / "terraforged-prepared.json").write_text(json.dumps(altered_manifest), encoding="utf-8")
                case_env = dict(environment, TERRAFORGED_HORIZON_PREPARED=str(case_root))
                negative = subprocess.run([str(executable), str(args.resources.resolve()), str(case_root), str(args.biomes.resolve())],
                                          env=case_env, cwd=work, capture_output=True, text=True, encoding="utf-8", errors="replace", timeout=90)
                (case_root / "stderr.log").write_text(negative.stderr, encoding="utf-8")
                if negative.returncode == 0 or message not in negative.stderr:
                    raise ValueError(f"Native decoder did not reject {case}: {negative.stderr[-1000:]}")
                expected_failures.append({"case": case, "rejected": True, "processExit": negative.returncode,
                                          "diagnostic": message})
            report.update({"status": "passed-prepared-terrain-native-probe", "native": native,
                           "expectedDecoderFailures": expected_failures,
                           "allJavaBlockAndQuartBiomeDigestsMatch": True,
                           "chunkCount": len(expected), "blockCount": sum(c["blockCount"] for c in expected.values()),
                           "quartBiomeCount": sum(c["quartBiomeCount"] for c in expected.values()),
                           "surfaceColumnsChecked": len(expected) * 256})
    report["finishedAt"] = dt.datetime.now(dt.timezone.utc).isoformat()
    args.report.parent.mkdir(parents=True, exist_ok=True)
    args.report.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({"status": report["status"], "processExit": result.returncode, "report": str(args.report), "error": report.get("error")}, indent=2))
    return 0 if report["status"].startswith("passed") else 1


if __name__ == "__main__":
    raise SystemExit(main())
