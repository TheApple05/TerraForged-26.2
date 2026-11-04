"""Compile/test two actual production density adapters against resolved 26.2 jars."""
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
SOURCES = [
    "common/src/main/java/raccoonman/reterraforged/world/worldgen/densityfunction/ClampToNearestUnit.java",
    "common/src/main/java/raccoonman/reterraforged/world/worldgen/densityfunction/LinearSplineFunction.java",
    "common/src/main/java/raccoonman/reterraforged/world/worldgen/noise/NoiseUtil.java",
    "tests/density/DensityProbe.java",
]


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--java-home", type=Path, default=os.environ.get("JAVA_HOME"))
    parser.add_argument("--classpath-file", type=Path, default=ROOT / "build/qa/target/compile-classpath.txt")
    parser.add_argument("--report", type=Path, default=ROOT / "build/qa/density/report.json")
    args = parser.parse_args()
    if args.java_home is None:
        parser.error("Supply a full Java25 JDK with --java-home or JAVA_HOME")
    if not args.classpath_file.exists():
        parser.error("Resolved target classpath is absent. Run .\\gradlew.bat writePortClasspath first; do not run concurrent Gradle builds")
    classpath = args.classpath_file.read_text(encoding="utf-8").strip()
    artifacts = [Path(p) for p in classpath.split(os.pathsep)]
    if not classpath or any(not p.exists() for p in artifacts):
        parser.error("Resolved classpath contains absent artifacts; rerun writePortClasspath")
    java_home = args.java_home.resolve()
    java = str(java_home / "bin" / ("java.exe" if os.name == "nt" else "java"))
    javac = str(java_home / "bin" / ("javac.exe" if os.name == "nt" else "javac"))
    base = ROOT / "build/qa/density"
    base.mkdir(parents=True, exist_ok=True)
    if not base.resolve().is_relative_to(ROOT.resolve()):
        raise RuntimeError("QA workspace escaped repository")
    version = subprocess.run([java, "-version"], capture_output=True, text=True, timeout=15, check=True).stderr.strip()
    with tempfile.TemporaryDirectory(prefix="probe-", dir=base) as temp:
        classes = Path(temp) / "classes"
        classes.mkdir()
        compile_result = subprocess.run(
            [javac, "--release", "25", "-proc:none", "-encoding", "UTF-8", "-cp", classpath,
             "-d", str(classes), *[str(ROOT / p) for p in SOURCES]],
            capture_output=True, text=True, cwd=ROOT, timeout=90)
        if compile_result.returncode:
            print(compile_result.stderr, file=sys.stderr)
            return compile_result.returncode
        result = subprocess.run([java, "-cp", str(classes) + os.pathsep + classpath, "DensityProbe"],
                                capture_output=True, text=True, cwd=ROOT, timeout=90)
        log = result.stdout + result.stderr
        (base / "latest-output.log").write_text(log, encoding="utf-8")
        marker = "DENSITY_QA_JSON="
        lines = [line.split(marker, 1)[1] for line in result.stdout.splitlines() if marker in line]
        report = json.loads(lines[-1]) if lines else {"status": "fail", "reason": "No completed QA result"}
        report.update({
            "schema_version": 1,
            "run_at_utc": datetime.now(timezone.utc).isoformat(),
            "java": version,
            "process_exit": result.returncode,
            "compile_exit": compile_result.returncode,
            "source_sha256_lf": {p: hashlib.sha256((ROOT / p).read_bytes().replace(b"\r\n", b"\n")).hexdigest()
                                   for p in SOURCES},
            "classpath_artifact_names": [p.name for p in artifacts],
            "not_tested": ["Custom registry type dispatch", "Fabric loader launch", "Other density functions",
                           "Chunk generation", "Packaged JAR", "Visual parity", "Performance"],
        })
        game_artifacts = [p for p in artifacts if p.name == report.get("game_class_artifact")]
        if len(game_artifacts) == 1:
            report["game_class_artifact_sha256"] = hashlib.sha256(game_artifacts[0].read_bytes()).hexdigest()
        if result.returncode:
            report["status"] = "fail"
            print(log[-12000:], file=sys.stderr)
        report_path = args.report if args.report.is_absolute() else ROOT / args.report
        report_path.parent.mkdir(parents=True, exist_ok=True)
        report_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
        print(f"Density adapter regression {report['status'].upper()}: "
              f"{report.get('assertions', 0)} assertions; process exit {result.returncode}")
        print(f"Report: {report_path}")
        return 0 if report["status"] == "pass" else 1


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (OSError, ValueError, RuntimeError, subprocess.SubprocessError) as exc:
        print(f"Density regression error: {exc}", file=sys.stderr)
        raise SystemExit(2)
