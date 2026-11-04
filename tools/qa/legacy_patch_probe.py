#!/usr/bin/env python3
"""Compare actual 1.20.1 vanilla and migrated 26.2 random-patch feature executions."""
from __future__ import annotations

import argparse
import concurrent.futures
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import urllib.request

ROOT = Path(__file__).resolve().parents[2]


def sha(path: Path, algorithm: str = "sha256") -> str:
    return hashlib.new(algorithm, path.read_bytes()).hexdigest()


def run(command: list[str], log: Path) -> str:
    result = subprocess.run(command, cwd=ROOT, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True, encoding="utf-8", errors="replace")
    log.write_text(result.stdout, encoding="utf-8")
    if result.returncode:
        raise RuntimeError(f"Command exited {result.returncode}; see {log}:\n{result.stdout[-3000:]}")
    return result.stdout


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--java-home", type=Path, required=True)
    parser.add_argument("--loom-cache", type=Path, default=Path.home() / ".gradle/caches/fabric-loom")
    parser.add_argument("--target-classpath", type=Path, default=Path("build/qa/target/compile-classpath.txt"))
    parser.add_argument("--report", type=Path, default=Path("docs/validation/legacy-patch-parity-2026-11-03.json"))
    args = parser.parse_args()
    output = ROOT / "build/qa/legacy-patch"
    output.mkdir(parents=True, exist_ok=True)
    baseline_info = args.loom_cache / "1.20.1/mojang_minecraft_info.json"
    if not baseline_info.is_file():
        parser.error("A named Minecraft 1.20.1 Loom cache is required; create the reference workspace first.")
    metadata = json.loads(baseline_info.read_text(encoding="utf-8"))
    if metadata["id"] != "1.20.1":
        parser.error("The baseline version descriptor must be Minecraft 1.20.1.")
    game_jars = []
    for component in ("minecraft-common", "minecraft-clientonly"):
        matches = list((args.loom_cache / "minecraftMaven/net/minecraft" / component).glob(f"1.20.1-*/*1.20.1*.jar"))
        if len(matches) != 1:
            parser.error(f"Expected one named 1.20.1 {component} JAR, found {len(matches)}; use --loom-cache to select the reference cache.")
        game_jars.append(matches[0])
    library_cache = Path.home() / ".gradle/caches/modules-2/files-2.1"
    downloads = output / "old-libs"
    downloads.mkdir(exist_ok=True)
    libraries = [item for item in metadata["libraries"] if "artifact" in item.get("downloads", {}) and ":natives-" not in item["name"]]

    def resolve(item: dict) -> Path:
        artifact = item["downloads"]["artifact"]
        group, name, version, *_ = item["name"].split(":")
        filename = Path(artifact["path"]).name
        cached = list((library_cache / group / name / version).rglob(filename))
        path = cached[0] if cached else downloads / filename
        if not path.is_file():
            urllib.request.urlretrieve(artifact["url"], path)
        if sha(path, "sha1") != artifact["sha1"]:
            raise RuntimeError(f"Official library checksum mismatch: {item['name']}")
        return path

    with concurrent.futures.ThreadPoolExecutor(max_workers=4) as pool:
        old_libraries = list(pool.map(resolve, libraries))
    baseline_cp = os.pathsep.join(str(path) for path in [*game_jars, *old_libraries])
    target_cp = (ROOT / args.target_classpath).read_text(encoding="utf-8").strip()
    extension = ".exe" if os.name == "nt" else ""
    javac = str(args.java_home / "bin" / ("javac" + extension))
    java = str(args.java_home / "bin" / ("java" + extension))
    probe = ROOT / "tools/qa/LegacyPatchProbe.java"
    migrated_feature = ROOT / "common/src/main/java/raccoonman/reterraforged/world/worldgen/feature/LegacyRandomPatchFeature.java"
    observations = {}
    for mode, classpath, release in (("baseline", baseline_cp, "17"), ("modern", target_cp, "25")):
        classes = output / f"{mode}-classes"
        sources = [str(probe)] + ([str(migrated_feature)] if mode == "modern" else [])
        run([javac, "-proc:none", "--release", release, "-cp", classpath, "-d", str(classes), *sources], output / f"{mode}-compile.log")
        result = run([java, "-cp", str(classes) + os.pathsep + classpath, "LegacyPatchProbe", mode], output / f"{mode}.log")
        match = re.search(r"PATCH_PROBE_JSON=(\{.+\})", result)
        if not match:
            raise RuntimeError(f"Missing structured probe result for {mode}")
        observations[mode] = json.loads(match.group(1))
    matched = observations["baseline"] == observations["modern"]
    report = {
        "schemaVersion": 1,
        "result": "passed" if matched else "failed",
        "comparison": "actual Minecraft 1.20.1 RandomPatchFeature versus migrated LegacyRandomPatchFeature on actual Minecraft 26.2",
        "baseline": observations["baseline"],
        "modern": observations["modern"],
        "tested": ["attempt counts 0/1/24/48/56/64", "six seeds including signed-long extremes", "positive and negative origins", "spreads 7/3, 0/0, 1/9", "nested placement RNG consumption", "success flags", "final RNG states"],
        "baselineGameJars": [{"component": name, "sha256": sha(path)} for name, path in zip(("common", "clientonly"), game_jars)],
        "baselineVersionDescriptorSha256": sha(baseline_info),
        "verifiedOfficialLibraries": len(old_libraries),
        "migratedFeatureSha256": sha(migrated_feature),
        "probeSha256": sha(probe),
        "limitations": ["Controlled recording child feature, not an in-world vegetation render", "WorldGenLevel test proxy only permits placement and height queries", "Reference named 1.20.1 Loom cache required; no Minecraft JAR is redistributed"],
    }
    report_path = ROOT / args.report
    report_path.parent.mkdir(parents=True, exist_ok=True)
    report_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({"result": report["result"], **observations}, indent=2))
    return 0 if matched else 1


if __name__ == "__main__":
    raise SystemExit(main())
