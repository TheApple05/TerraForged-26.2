"""Verify migrated legacy terrace output against the frozen original Noise2D oracle."""
from __future__ import annotations
import argparse
import datetime as dt
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]
SOURCES = ["common/src/main/java/raccoonman/reterraforged/world/worldgen/feature/ClassicPillarMath.java",
           "tests/terrain/ClassicPillarProbe.java"]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--java-home", type=Path, required=True)
    parser.add_argument("--report", type=Path, default=Path("build/qa/classic-pillars.json"))
    args = parser.parse_args()
    baseline = json.loads((ROOT / "tests/terrain/classic-pillar-reference.json").read_text())
    base = ROOT / "build/qa/classic-pillars"
    base.mkdir(parents=True, exist_ok=True)
    suffix = ".exe" if os.name == "nt" else ""
    with tempfile.TemporaryDirectory(dir=base) as temp:
        subprocess.run([str(args.java_home / "bin" / ("javac" + suffix)), "--release", "25", "-d", temp,
                        *[str(ROOT / source) for source in SOURCES]], check=True, timeout=90)
        result = subprocess.run([str(args.java_home / "bin" / ("java" + suffix)), "-cp", temp, "ClassicPillarProbe"],
                                capture_output=True, text=True, check=True, timeout=30)
    report = json.loads(result.stdout)
    if report["terraceRawBitsSha256"] != baseline["terraceRawBitsSha256"]:
        raise AssertionError("LegacyTerrace raw output changed from the independently compiled Noise2D source")
    report.update({"runAtUtc": dt.datetime.now(dt.timezone.utc).isoformat(), "reference": baseline,
                   "sourceSha256Lf": {name: hashlib.sha256((ROOT / name).read_bytes().replace(b"\r\n", b"\n")).hexdigest() for name in SOURCES},
                   "scope": "Legacy terrace scalar arithmetic and original mask/height equations. Packaged feature placement, geological layering and classic game appearance require runtime validation."})
    path = args.report if args.report.is_absolute() else ROOT / args.report
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({"status": report["status"], "assertions": report["assertions"], "report": str(path)}))


if __name__ == "__main__":
    main()
