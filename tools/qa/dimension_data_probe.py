#!/usr/bin/env python3
"""Check custom dimension bounds and exact 26.2 default-overworld serialization."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess

ROOT = Path(__file__).resolve().parents[2]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--java-home", type=Path, required=True)
    parser.add_argument("--report", type=Path, default=Path("docs/validation/dimension-data-2026-11-03.json"))
    args = parser.parse_args()
    output = ROOT / "build/qa/data-migration"
    output.mkdir(parents=True, exist_ok=True)
    cp = (ROOT / "build/qa/target/compile-classpath.txt").read_text().strip()
    extension = ".exe" if os.name == "nt" else ""
    feature = ROOT / "common/src/main/java/raccoonman/reterraforged/data/worldgen/OverworldDimensionData.java"
    probe = ROOT / "tools/qa/DimensionDataProbe.java"
    classes = output / "dimension-classes"
    commands = [
        [str(args.java_home / "bin" / ("javac" + extension)), "-proc:none", "--release", "25", "-cp", cp, "-d", str(classes), str(feature), str(probe)],
        [str(args.java_home / "bin" / ("java" + extension)), "-cp", str(classes) + os.pathsep + cp, "DimensionDataProbe"],
    ]
    for index, command in enumerate(commands):
        result = subprocess.run(command, cwd=ROOT, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True, encoding="utf-8", errors="replace")
        (output / f"dimension-probe-{index}.log").write_text(result.stdout, encoding="utf-8")
        if result.returncode:
            print(result.stdout[-3000:])
            return result.returncode
    match = re.search(r"DIMENSION_PROBE_JSON=(\{.+\})", result.stdout)
    if not match:
        raise RuntimeError("Missing structured dimension result")
    report = {
        "schemaVersion": 1,
        "result": "passed",
        "minecraft": "26.2",
        **json.loads(match.group(1)),
        "checks": ["Complete encoded default dimension equals actual vanilla 26.2 overworld", "Nine worldDepth/worldHeight combinations retain min_y, height and logical_height", "Encoded bounds preserve all nine combinations"],
        "limitations": ["Does not generate chunks", "Decode round-trip requires loaded block/timeline tags, absent from standalone VanillaRegistries.createLookup; not claimed"],
        "sourceSha256": hashlib.sha256(feature.read_bytes()).hexdigest(),
        "probeSha256": hashlib.sha256(probe.read_bytes()).hexdigest(),
    }
    report_path = ROOT / args.report
    report_path.parent.mkdir(parents=True, exist_ok=True)
    report_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(report, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
