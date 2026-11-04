"""Verify production future-cache reentry and concurrency without Minecraft stubs."""
from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]
COMMIT = "29239f1f164d19dbfcccaca4a8277d64e784207c"
SOURCES = [
    "common/src/main/java/raccoonman/reterraforged/concurrent/task/LazyCallable.java",
    "common/src/main/java/raccoonman/reterraforged/concurrent/cache/CacheEntry.java",
    "common/src/main/java/raccoonman/reterraforged/concurrent/cache/ExpiringEntry.java",
    "common/src/main/java/raccoonman/reterraforged/concurrent/cache/SafeCloseable.java",
]


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--java-home", type=Path, default=os.environ.get("JAVA_HOME"))
    parser.add_argument("--report", type=Path, default=Path("build/qa/cache/report.json"))
    parser.add_argument("--prove-baseline-deadlock", action="store_true")
    args = parser.parse_args()
    if args.java_home is None:
        parser.error("Select a full Java 25 JDK using --java-home or JAVA_HOME")
    extension = ".exe" if os.name == "nt" else ""
    javac = str(args.java_home / "bin" / ("javac" + extension))
    java = str(args.java_home / "bin" / ("java" + extension))
    base = ROOT / "build/qa/cache"
    base.mkdir(parents=True, exist_ok=True)
    results = {}
    with tempfile.TemporaryDirectory(dir=base, prefix="probe-") as directory:
        for mode in (["candidate", "baseline"] if args.prove_baseline_deadlock else ["candidate"]):
            location = Path(directory) / mode
            classes = location / "classes"
            classes.mkdir(parents=True)
            sources = []
            for relative in SOURCES:
                content = ((ROOT / relative).read_bytes() if mode == "candidate" else
                           subprocess.check_output(["git", "-C", str(ROOT / ".reference/ReTerraForged"),
                                                    "show", f"{COMMIT}:{relative}"]))
                path = location / relative
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_bytes(content)
                sources.append(str(path))
            compilation = subprocess.run([javac, "--release", "25", "-d", str(classes),
                                          str(ROOT / "tests/cache/CacheEntryProbe.java"), *sources],
                                         cwd=ROOT, capture_output=True, text=True, timeout=30)
            if compilation.returncode:
                raise RuntimeError(f"{mode} compilation failed: {compilation.stderr[-3000:]}")
            process = subprocess.run([java, "-cp", str(classes), "CacheEntryProbe", mode],
                                     cwd=ROOT, capture_output=True, text=True, timeout=12)
            markers = [line.split("CACHE_QA_JSON=", 1)[1] for line in process.stdout.splitlines()
                       if "CACHE_QA_JSON=" in line]
            if process.returncode or not markers:
                raise RuntimeError(f"{mode} regression failed: {process.stderr[-3000:]}")
            results[mode] = {**json.loads(markers[-1]), "process_exit": process.returncode}
    report = {
        "schema_version": 1, "status": "pass", "results": results,
        "baseline_commit": COMMIT if args.prove_baseline_deadlock else None,
        "source_sha256_lf": {path: hashlib.sha256((ROOT / path).read_bytes().replace(b"\r\n", b"\n")).hexdigest()
                             for path in SOURCES},
        "scope": "Production cache future reentry, parallel result publication, failure and close behavior; packaged world generation is a separate gate",
    }
    report_path = ROOT / args.report
    report_path.parent.mkdir(parents=True, exist_ok=True)
    report_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    print(f"Cache regression PASS: {results['candidate']['assertions']} assertions; baseline deadlock reproduced: {'baseline' in results}")
    print(f"Report: {report_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
