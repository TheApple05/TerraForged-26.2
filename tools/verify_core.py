"""Verify unchanged upstream math kernels without Minecraft or test-only stubs.

Normal verification never regenerates fixtures. Reference capture is an explicit
maintenance operation using git-show of the pinned upstream commit.
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
COMMIT = "29239f1f164d19dbfcccaca4a8277d64e784207c"
PREFIX = "common/src/main/java/raccoonman/reterraforged/world/worldgen/"
SOURCES = [PREFIX + p for p in (
    "noise/NoiseUtil.java", "util/PosUtil.java", "util/Seed.java", "util/FastRandom.java"
)]
PROBE = ROOT / "tests/core/CoreProbe.java"
FIXTURE = ROOT / "tests/core/reference-output.tsv"
METADATA = ROOT / "tests/core/reference-metadata.json"


def digest(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def run(command: list[str]) -> bytes:
    result = subprocess.run(command, capture_output=True, cwd=ROOT, timeout=90)
    if result.returncode:
        raise RuntimeError(f"Command failed ({result.returncode}): {command[0]}\n"
                           + result.stderr.decode("utf-8", errors="replace"))
    return result.stdout.replace(b"\r\n", b"\n")


def source_hash(data: bytes) -> str:
    # Git for Windows may convert checked-out text to CRLF. Hash canonical LF.
    return digest(data.replace(b"\r\n", b"\n"))


def compile_probe(java_home: Path, sources: dict[str, bytes], workspace: Path) -> Path:
    classes = workspace / "classes"
    classes.mkdir()
    files = []
    for relative, data in sources.items():
        path = workspace / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(data)
        files.append(str(path))
    run([str(java_home / "bin/javac.exe" if os.name == "nt" else java_home / "bin/javac"),
         "--release", "25", "-encoding", "UTF-8", "-d", str(classes), str(PROBE), *files])
    return classes


def probe(java_home: Path, classes: Path, mode: str) -> bytes:
    return run([str(java_home / "bin/java.exe" if os.name == "nt" else java_home / "bin/java"),
                "-Djava.util.concurrent.ForkJoinPool.common.parallelism=4",
                "-cp", str(classes), "CoreProbe", mode])


def first_difference(expected: bytes, actual: bytes) -> str:
    left, right = expected.decode().splitlines(), actual.decode().splitlines()
    for i, (a, b) in enumerate(zip(left, right), 1):
        if a != b:
            return f"record {i}: expected {a[:180]!r}; actual {b[:180]!r}"
    return f"record counts: expected {len(left)}, actual {len(right)}"


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--java-home", type=Path, default=os.environ.get("JAVA_HOME"))
    parser.add_argument("--report", type=Path, default=ROOT / "build/qa/core/report.json")
    parser.add_argument("--generate-reference", action="store_true",
                        help="Explicitly capture pinned upstream output, never candidate output")
    parser.add_argument("--upstream-repo", type=Path, default=ROOT / ".reference/ReTerraForged")
    parser.add_argument("--replace-fixture", action="store_true",
                        help="Allow replacing an existing fixture during explicit reference capture")
    parser.add_argument("--self-test", action="store_true",
                        help="Also prove a temporary hash-kernel mutation is rejected; never edit production source")
    args = parser.parse_args()
    if args.java_home is None:
        parser.error("Supply --java-home pointing to a full Java 25 JDK, or set JAVA_HOME")
    java_home = args.java_home.resolve()
    java_version = subprocess.run(
        [str(java_home / "bin/java.exe" if os.name == "nt" else java_home / "bin/java"), "-version"],
        capture_output=True, text=True, timeout=15, check=True).stderr.strip()
    base = ROOT / "build/qa/core"
    base.mkdir(parents=True, exist_ok=True)
    # TemporaryDirectory is always created within this resolved workspace path.
    if not base.resolve().is_relative_to(ROOT.resolve()):
        raise RuntimeError("QA temporary directory escaped the workspace")
    with tempfile.TemporaryDirectory(prefix="probe-", dir=base) as temp:
        workspace = Path(temp)
        if args.generate_reference:
            if (FIXTURE.exists() or METADATA.exists()) and not args.replace_fixture:
                parser.error("Fixture already exists; reference replacement requires --replace-fixture")
            source = {p: run(["git", "-C", str(args.upstream_repo), "show", f"{COMMIT}:{p}"])
                      for p in SOURCES}
            classes = compile_probe(java_home, source, workspace)
            output = probe(java_home, classes, "serial")
            metadata = {
                "schema_version": 1,
                "upstream_repository": "https://github.com/racoonman2/ReTerraForged",
                "upstream_commit": COMMIT,
                "source_sha256_lf": {p: source_hash(data) for p, data in source.items()},
                "probe_sha256_lf": source_hash(PROBE.read_bytes()),
                "fixture_sha256": digest(output),
                "record_count": len(output.splitlines()),
                "capture_java": java_version,
                "scope": "Unchanged math/hash/gradient kernels and utility classes; not full terrain or Minecraft integration",
            }
            FIXTURE.write_bytes(output)
            METADATA.write_text(json.dumps(metadata, indent=2) + "\n", encoding="utf-8")
            print(f"Captured pinned upstream fixture: {metadata['record_count']} records; SHA256 {digest(output)}")
            return 0

        metadata = json.loads(METADATA.read_text(encoding="utf-8"))
        expected = FIXTURE.read_bytes().replace(b"\r\n", b"\n")
        if digest(expected) != metadata["fixture_sha256"]:
            raise RuntimeError("Fixture hash disagrees with frozen metadata")
        if source_hash(PROBE.read_bytes()) != metadata["probe_sha256_lf"]:
            raise RuntimeError("Probe changed since reference capture; review and explicitly recapture upstream fixture")
        if metadata["upstream_commit"] != COMMIT:
            raise RuntimeError("Fixture upstream commit does not match the pinned reference")
        source = {p: (ROOT / p).read_bytes() for p in SOURCES}
        classes = compile_probe(java_home, source, workspace)
        results = []
        for mode in ["serial", "serial", "shuffled", "parallel"]:
            output = probe(java_home, classes, mode)
            matched = output == expected
            results.append({"mode": mode, "separate_jvm": True, "matched_reference": matched,
                            "record_count": len(output.splitlines()), "sha256": digest(output)})
            if not matched:
                results[-1]["first_difference"] = first_difference(expected, output)
        passed = all(item["matched_reference"] for item in results)
        mutation_check = None
        if args.self_test:
            mutated = dict(source)
            noise_path = PREFIX + "noise/NoiseUtil.java"
            old = b"public static int hash2D(int seed, int x, int y) {\n        int hash = seed;"
            new = b"public static int hash2D(int seed, int x, int y) {\n        int hash = seed ^ 1;"
            noise_bytes = mutated[noise_path].replace(b"\r\n", b"\n")
            if noise_bytes.count(old) != 1:
                raise RuntimeError("Self-test mutation site changed; review the mutation instead of guessing")
            mutated[noise_path] = noise_bytes.replace(old, new, 1)
            mutation_workspace = workspace / "mutation"
            mutation_workspace.mkdir()
            mutated_classes = compile_probe(java_home, mutated, mutation_workspace)
            mutated_output = probe(java_home, mutated_classes, "serial")
            detected = mutated_output != expected
            mutation_check = {"temporary_hash2D_seed_mutation_detected": detected,
                              "production_sources_modified": False,
                              "first_difference": first_difference(expected, mutated_output)}
            passed = passed and detected
        report = {
            "schema_version": 1,
            "status": "pass" if passed else "fail",
            "run_at_utc": datetime.now(timezone.utc).isoformat(),
            "java": java_version,
            "upstream_commit": COMMIT,
            "fixture_sha256": metadata["fixture_sha256"],
            "source_sha256_lf": {p: source_hash(data) for p, data in source.items()},
            "sources_identical_to_upstream": all(source_hash(source[p]) == metadata["source_sha256_lf"][p]
                                                   for p in SOURCES),
            "runs": results,
            "harness_self_test": mutation_check,
            "scope": metadata["scope"],
            "legacy_observation": "NoiseUtil.floor(-1.0f) returns -2; preserved as reference behavior, not endorsed as mathematical floor",
            "not_tested": ["Full noise module composition", "Continents/climate/rivers/erosion",
                           "Minecraft chunk generation or seams", "Tile caches", "Client or dedicated server",
                           "Packaged JAR", "Visual parity", "Performance"],
        }
        report_path = args.report if args.report.is_absolute() else ROOT / args.report
        report_path.parent.mkdir(parents=True, exist_ok=True)
        report_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
        print(f"Core kernel regression {'PASS' if passed else 'FAIL'}: {len(expected.splitlines())} records x "
              f"{len(results)} isolated JVM runs; SHA256 {metadata['fixture_sha256']}")
        print(f"Report: {report_path}")
        for item in results:
            if not item["matched_reference"]:
                print(item["first_difference"])
        return 0 if passed else 1


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (OSError, subprocess.SubprocessError, ValueError, RuntimeError) as exc:
        print(f"Core regression error: {exc}", file=sys.stderr)
        raise SystemExit(2)
