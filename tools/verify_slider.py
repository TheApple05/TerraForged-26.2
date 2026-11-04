"""Exercise production Slider input paths with real Minecraft 26.2 classes."""
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
SOURCES = ["common/src/main/java/raccoonman/reterraforged/client/gui/widget/Slider.java", "tests/client/SliderProbe.java"]


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--java-home", type=Path, default=os.environ.get("JAVA_HOME"))
    parser.add_argument("--report", type=Path, default=ROOT / "build/qa/slider/report.json")
    args = parser.parse_args()
    if args.java_home is None:
        parser.error("Supply --java-home or JAVA_HOME pointing to a full Java25 JDK")
    cp_file = ROOT / "build/qa/target/compile-classpath.txt"
    if not cp_file.exists():
        parser.error("Run .\\gradlew.bat writePortClasspath first; do not run concurrent Gradle builds")
    cp = cp_file.read_text().strip()
    base = ROOT / "build/qa/slider"
    base.mkdir(parents=True, exist_ok=True)
    if not base.resolve().is_relative_to(ROOT.resolve()):
        raise RuntimeError("QA directory escaped workspace")
    java = str(args.java_home / "bin" / ("java.exe" if os.name == "nt" else "java"))
    javac = str(args.java_home / "bin" / ("javac.exe" if os.name == "nt" else "javac"))
    with tempfile.TemporaryDirectory(prefix="events-", dir=base) as temp:
        compiled = subprocess.run([javac, "--release", "25", "-proc:none", "-encoding", "UTF-8", "-cp", cp,
                                   "-d", temp, *[str(ROOT / p) for p in SOURCES]],
                                  capture_output=True, text=True, cwd=ROOT, timeout=90)
        if compiled.returncode:
            print(compiled.stderr, file=sys.stderr)
            return compiled.returncode
        result = subprocess.run([java, "-cp", temp + os.pathsep + cp, "SliderProbe"],
                                capture_output=True, text=True, cwd=ROOT, timeout=90)
        log = result.stdout + result.stderr
        (base / "events-output.log").write_text(log, encoding="utf-8")
        lines = [line.split("SLIDER_QA_JSON=", 1)[1] for line in result.stdout.splitlines() if "SLIDER_QA_JSON=" in line]
        report = json.loads(lines[-1]) if lines else {"status": "fail", "reason": "Probe did not complete"}
        errors = [line.strip() for line in log.splitlines() if "AssertionError:" in line]
        report.update({"schema_version": 1, "run_at_utc": datetime.now(timezone.utc).isoformat(),
                       "compile_exit": compiled.returncode, "process_exit": result.returncode,
                       "source_sha256_lf": {p: hashlib.sha256((ROOT / p).read_bytes().replace(b"\r\n", b"\n")).hexdigest()
                                            for p in SOURCES}, "observed_assertion_failures": errors,
                       "not_tested": ["Rendered slider appearance", "Full editor navigation and preset persistence", "Client GPU/window input routing"]})
        if result.returncode:
            report["status"] = "fail"
            print(log[-8000:], file=sys.stderr)
        output = args.report if args.report.is_absolute() else ROOT / args.report
        output.parent.mkdir(parents=True, exist_ok=True)
        output.write_text(json.dumps(report, indent=2) + "\n")
        print(f"Slider event regression {report['status'].upper()}: {report.get('assertions', 0)} assertions; exit {result.returncode}")
        return 0 if report["status"] == "pass" else 1


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (OSError, ValueError, RuntimeError, subprocess.SubprocessError) as exc:
        print(f"Slider regression error: {exc}", file=sys.stderr)
        raise SystemExit(2)
