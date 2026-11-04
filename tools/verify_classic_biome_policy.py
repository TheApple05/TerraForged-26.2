"""Exercise classic-biome selection and settings with the real 26.2 classpath.

This is a synthetic policy/codec check, not evidence that a biome generated.
"""
from __future__ import annotations

import argparse
from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile

ROOT = Path(__file__).resolve().parents[1]
PROBE = ROOT / "tests/biome/ClassicBiomePolicyProbe.java"


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--java-home", type=Path, required=True)
    parser.add_argument("--production-jar", type=Path, required=True)
    parser.add_argument("--classpath-file", type=Path, default=ROOT / "build/qa/target/compile-classpath.txt")
    parser.add_argument("--report", type=Path, default=ROOT / "build/qa/classic-policy/report.json")
    args = parser.parse_args()
    production = args.production_jar.resolve()
    classpath = str(production) + os.pathsep + args.classpath_file.read_text(encoding="utf-8").strip()
    base = ROOT / "build/qa/classic-policy"
    base.mkdir(parents=True, exist_ok=True)
    if not base.resolve().is_relative_to(ROOT.resolve()):
        raise RuntimeError("QA temporary directory escaped the workspace")
    suffix = ".exe" if os.name == "nt" else ""
    with tempfile.TemporaryDirectory(prefix="compile-", dir=base) as work:
        compile_run = subprocess.run([str(args.java_home / "bin" / ("javac" + suffix)), "--release", "25", "-proc:none",
            "-encoding", "UTF-8", "-cp", classpath, "-d", work, str(PROBE)], capture_output=True, text=True, cwd=ROOT, timeout=90)
        if compile_run.returncode:
            print(compile_run.stdout + compile_run.stderr, file=sys.stderr)
            return compile_run.returncode
        run = subprocess.run([str(args.java_home / "bin" / ("java" + suffix)), "-cp", work + os.pathsep + classpath,
            "ClassicBiomePolicyProbe"], capture_output=True, text=True, cwd=ROOT, timeout=90)
        log = run.stdout + run.stderr
        (base / "output.log").write_text(log, encoding="utf-8")
        markers = [line.split("CLASSIC_BIOME_POLICY_JSON=", 1)[1] for line in run.stdout.splitlines() if "CLASSIC_BIOME_POLICY_JSON=" in line]
        report = json.loads(markers[-1]) if markers else {"status": "fail"}
        report.update({"schemaVersion": 1, "runAtUtc": datetime.now(timezone.utc).isoformat(), "processExit": run.returncode,
            "compileExit": compile_run.returncode, "productionJar": str(production),
            "productionSha256": hashlib.sha256(production.read_bytes()).hexdigest(),
            "probeSha256Lf": hashlib.sha256(PROBE.read_bytes().replace(b"\r\n", b"\n")).hexdigest(),
            "notEvidenceOf": ["Biome reachability in a real world", "Registry export", "Mixin application", "FULL chunk generation", "Visual appearance"]})
        if run.returncode:
            report["status"] = "fail"
            print(log[-7000:], file=sys.stderr)
        args.report.parent.mkdir(parents=True, exist_ok=True)
        args.report.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
        print(json.dumps(report, indent=2))
        return 0 if report["status"] == "pass" else 1


if __name__ == "__main__":
    raise SystemExit(main())
