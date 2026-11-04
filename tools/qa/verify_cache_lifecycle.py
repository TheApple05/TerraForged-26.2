"""Reproduce exact upstream retained-cache leak and verify actual candidate lifecycle classes."""
from __future__ import annotations
import argparse
import datetime as dt
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[2]
UPSTREAM = "29239f1f164d19dbfcccaca4a8277d64e784207c"
PREFIX = "common/src/main/java/raccoonman/reterraforged/concurrent/cache/"
FILES = [PREFIX + part for part in ("Cache.java", "CacheManager.java", "ExpiringEntry.java",
    "map/LongMap.java", "map/StampedBoundLongMap.java", "map/StampedLongMap.java")]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--java-home", type=Path, required=True)
    parser.add_argument("--candidate-only", action="store_true", help="CI: verify candidate without cloning/reproducing upstream")
    parser.add_argument("--production-jar", type=Path, help="Exercise classes from this exact packaged candidate instead of compiling source")
    parser.add_argument("--classpath-file", type=Path, default=ROOT / "build/qa/target/compile-classpath.txt")
    parser.add_argument("--report", type=Path, default=ROOT / "docs/validation/cache-lifecycle-2026-11-04.json")
    args = parser.parse_args()
    java = args.java_home / "bin" / ("java.exe" if os.name == "nt" else "java")
    javac = args.java_home / "bin" / ("javac.exe" if os.name == "nt" else "javac")
    classpath = args.classpath_file.read_text().strip()
    report = {"schemaVersion": 1, "startedAtUtc": dt.datetime.now(dt.timezone.utc).isoformat(), "status": "failed", "upstreamCommit": UPSTREAM}
    temp_root = ROOT / "build/qa/cache-lifecycle"
    temp_root.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(dir=temp_root) as temp:
        work = Path(temp)
        outputs = {}
        for label in (["candidate"] if args.candidate_only else ["upstream", "candidate"]):
            classes = work / label / "classes"; classes.mkdir(parents=True)
            sources = []
            hashes = {}
            for name in FILES:
                content = subprocess.check_output(["git", "-C", str(ROOT / ".reference/ReTerraForged"), "show", UPSTREAM + ":" + name]) if label == "upstream" else (ROOT / name).read_bytes()
                hashes[name] = hashlib.sha256(content.replace(b"\r\n", b"\n")).hexdigest()
                if label == "upstream" or args.production_jar is None:
                    path = work / label / name; path.parent.mkdir(parents=True, exist_ok=True); path.write_bytes(content); sources.append(path)
            probes = [ROOT / "tests/cache/CacheLifecycleLeakProbe.java"]
            if label == "candidate": probes.append(ROOT / "tests/cache/CacheLifecycleProbe.java")
            packaged = args.production_jar.resolve() if label == "candidate" and args.production_jar else None
            compile_cp = str(packaged) + os.pathsep + classpath if packaged else classpath
            compiled = subprocess.run([str(javac), "--release", "25", "-proc:none", "-encoding", "UTF-8", "-cp", compile_cp, "-d", str(classes), *map(str, sources + probes)], capture_output=True, text=True, timeout=60)
            if compiled.returncode: raise RuntimeError(compiled.stdout + compiled.stderr)
            checks = {}
            for probe in [file.stem for file in probes]:
                origin_log = work / label / (probe + "-class-origin.log")
                result = subprocess.run([str(java), "-Xlog:class+load=info:file=" + origin_log.as_posix(), "-cp", str(classes) + os.pathsep + compile_cp, probe], capture_output=True, text=True, timeout=30)
                checks[probe] = {"exitCode": result.returncode, "stdout": result.stdout.strip(), "stderr": result.stderr.strip()}
                if packaged:
                    loaded = [line for line in origin_log.read_text().splitlines() if "raccoonman.reterraforged.concurrent.cache.Cache source:" in line]
                    checks[probe]["packagedClassOrigin"] = loaded
                    checks[probe]["packagedClassOriginObserved"] = bool(loaded and all(packaged.name in line for line in loaded))
            outputs[label] = {"sourceSha256Lf": hashes, "compileExit": compiled.returncode, "checks": checks,
                "productionJar": str(packaged) if packaged else None,
                "productionJarSha256": hashlib.sha256(packaged.read_bytes()).hexdigest() if packaged else None}
        report["results"] = outputs
        reproduced = outputs["upstream"]["checks"]["CacheLifecycleLeakProbe"]["exitCode"] != 0 if "upstream" in outputs else None
        fixed = all(check["exitCode"] == 0 and check.get("packagedClassOriginObserved", True) for check in outputs["candidate"]["checks"].values())
        report.update({"upstreamLeakReproduced": reproduced, "upstreamReproductionSkipped": args.candidate_only,
            "candidateChecksPassed": fixed, "status": "passed" if fixed and (args.candidate_only or reproduced) else "failed"})
    report["finishedAtUtc"] = dt.datetime.now(dt.timezone.utc).isoformat()
    report["harnessSha256Lf"] = hashlib.sha256(Path(__file__).read_bytes().replace(b"\r\n", b"\n")).hexdigest()
    report["probeSourceSha256Lf"] = {file.relative_to(ROOT).as_posix(): hashlib.sha256(file.read_bytes().replace(b"\r\n", b"\n")).hexdigest()
        for file in (ROOT / "tests/cache/CacheLifecycleLeakProbe.java", ROOT / "tests/cache/CacheLifecycleProbe.java")}
    args.report.parent.mkdir(parents=True, exist_ok=True); args.report.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(report, indent=2))
    return 0 if report["status"] == "passed" else 1


if __name__ == "__main__": raise SystemExit(main())
