"""Test the real spawn search scope helper with the resolved Minecraft 26.2 jars."""
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
SOURCES = ["common/src/main/java/raccoonman/reterraforged/world/worldgen/SpawnSearchContext.java",
           "tests/spawn/SpawnScopeProbe.java"]


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--java-home", type=Path, default=os.environ.get("JAVA_HOME"))
    parser.add_argument("--report", type=Path, default=ROOT / "build/qa/spawn/report.json")
    args = parser.parse_args()
    if args.java_home is None:
        parser.error("Supply --java-home or JAVA_HOME pointing to a full Java25 JDK")
    cp_file = ROOT / "build/qa/target/compile-classpath.txt"
    if not cp_file.exists():
        parser.error("Run .\\gradlew.bat writePortClasspath before this test; do not run concurrent Gradle builds")
    cp = cp_file.read_text().strip()
    base = ROOT / "build/qa/spawn"
    base.mkdir(parents=True, exist_ok=True)
    if not base.resolve().is_relative_to(ROOT.resolve()):
        raise RuntimeError("QA directory escaped the workspace")
    java = str(args.java_home / "bin" / ("java.exe" if os.name == "nt" else "java"))
    javac = str(args.java_home / "bin" / ("javac.exe" if os.name == "nt" else "javac"))
    with tempfile.TemporaryDirectory(prefix="scope-", dir=base) as temp:
        compile_result = subprocess.run([javac, "--release", "25", "-proc:none", "-encoding", "UTF-8", "-cp", cp,
                                         "-d", temp, *[str(ROOT / p) for p in SOURCES]],
                                        capture_output=True, text=True, cwd=ROOT, timeout=90)
        if compile_result.returncode:
            print(compile_result.stderr, file=sys.stderr)
            return compile_result.returncode
        result = subprocess.run([java, "-cp", temp + os.pathsep + cp, "SpawnScopeProbe"],
                                capture_output=True, text=True, cwd=ROOT, timeout=90)
        log = result.stdout + result.stderr
        (base / "scope-output.log").write_text(log, encoding="utf-8")
        lines = [line.split("SPAWN_QA_JSON=", 1)[1] for line in result.stdout.splitlines() if "SPAWN_QA_JSON=" in line]
        report = json.loads(lines[-1]) if lines else {"status": "fail"}
        report.update({"schema_version": 1, "run_at_utc": datetime.now(timezone.utc).isoformat(),
                       "compile_exit": compile_result.returncode, "process_exit": result.returncode,
                       "source_sha256_lf": {p: hashlib.sha256((ROOT / p).read_bytes().replace(b"\r\n", b"\n")).hexdigest()
                                            for p in SOURCES},
                       "not_tested": ["Mixin injection/application", "Actual continent-center spawn", "Island preference",
                                      "Dedicated-server or client spawning", "Optional TerraBlender runtime"]})
        if result.returncode:
            report["status"] = "fail"
            print(log[-8000:], file=sys.stderr)
        output = args.report if args.report.is_absolute() else ROOT / args.report
        output.parent.mkdir(parents=True, exist_ok=True)
        output.write_text(json.dumps(report, indent=2) + "\n")
        print(f"Spawn scope regression {report['status'].upper()}: {report.get('assertions', 0)} assertions; exit {result.returncode}")
        return 0 if report["status"] == "pass" else 1


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (OSError, ValueError, RuntimeError, subprocess.SubprocessError) as exc:
        print(f"Spawn scope regression error: {exc}", file=sys.stderr)
        raise SystemExit(2)
