"""Check the initial Rust scalar/hash subset against frozen upstream Java output."""
from __future__ import annotations
import argparse
import datetime as dt
import hashlib
import json
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[2]


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def rows(text):
    result = {}
    for line in text.splitlines():
        key, value = line.split("\t", 1)
        if key in result:
            raise ValueError("Duplicate probe record: " + key)
        result[key] = value.split(",")
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--report", type=Path, default=ROOT / "docs/validation/horizon-native-scalars-initial.json")
    args = parser.parse_args()
    if args.report.exists():
        parser.error("Refusing to overwrite scalar evidence")
    fixture = ROOT / "tests/core/reference-output.tsv"
    metadata_path = ROOT / "tests/core/reference-metadata.json"
    metadata = json.loads(metadata_path.read_text())
    if digest(fixture) != metadata["fixture_sha256"]:
        raise ValueError("Frozen Java fixture bytes changed")
    expected = {}
    for key, value in rows(fixture.read_text()).items():
        if key.startswith("noise/"):
            expected[key] = [value[index] for index in (0, 1, 2, 3, 5)]
        elif key.startswith("math/"):
            expected[key] = [value[index] for index in (0, 1, 2, 5, 6, 7, 8, 9, 10, 11)]
        elif key == "legacy/floor-negative-integer":
            expected[key] = value
    if len(expected) != 1745:
        raise ValueError("Unexpected pinned reference subset")
    manifest = ROOT / "horizon/native-core/Cargo.toml"
    logs = ROOT / "build/qa/horizon/native-scalars"
    logs.mkdir(parents=True, exist_ok=True)
    report = {"schemaVersion": 1, "status": "failed", "startedAtUtc": dt.datetime.now(dt.timezone.utc).isoformat(),
        "upstreamCommit": metadata["upstream_commit"], "frozenJavaFixtureSha256": digest(fixture),
        "frozenJavaMetadataSha256": digest(metadata_path), "recordsPerProfile": len(expected),
        "valuesPerProfile": sum(map(len, expected.values())), "profiles": [],
        "scope": "Selected scalar/hash fields from genuine frozen Java records; not gradient tables, complete noise modules, terrain, chunks or native activation.",
        "notCovered": ["Vector/gradient tables", "Perlin and composed noises", "Cell/river/tile pipeline", "Every exported scalar API or nonfinite input", "Horizon infinite generation"]}
    for profile in ("debug", "release"):
        command = ["cargo", "+stable", "run", "--quiet", "--offline", "--locked", "--manifest-path", str(manifest), "--bin", "scalar_probe"]
        if profile == "release":
            command.append("--release")
        result = subprocess.run(command, cwd=ROOT, capture_output=True, text=True, encoding="utf-8", timeout=90)
        output = logs / (profile + ".tsv")
        output.write_text(result.stdout, encoding="utf-8", newline="\n")
        (logs / (profile + ".stderr.log")).write_text(result.stderr, encoding="utf-8")
        actual = rows(result.stdout) if result.returncode == 0 else {}
        differences = [{"key": key, "expected": expected.get(key), "actual": actual.get(key)}
            for key in sorted(set(actual) | set(expected)) if actual.get(key) != expected.get(key)]
        report["profiles"].append({"profile": profile, "exitCode": result.returncode,
            "outputSha256": digest(output), "differences": differences, "allSelectedValuesMatch": result.returncode == 0 and not differences})
    report["status"] = "passed-scalars-only" if all(row["allSelectedValuesMatch"] for row in report["profiles"]) else "failed"
    sources = [ROOT / "horizon/native-core/src/noise_util.rs", ROOT / "horizon/native-core/src/bin/scalar_probe.rs", manifest]
    report["sourceSha256Lf"] = {p.relative_to(ROOT).as_posix(): hashlib.sha256(p.read_bytes().replace(b"\r\n", b"\n")).hexdigest() for p in sources}
    report["finishedAtUtc"] = dt.datetime.now(dt.timezone.utc).isoformat()
    report["verifierSha256"] = digest(Path(__file__))
    args.report.parent.mkdir(parents=True, exist_ok=True)
    args.report.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({"status": report["status"], "recordsPerProfile": len(expected), "valuesPerProfile": report["valuesPerProfile"],
        "differences": [len(row["differences"]) for row in report["profiles"]], "report": str(args.report)}))
    return 0 if report["status"] == "passed-scalars-only" else 1


if __name__ == "__main__":
    raise SystemExit(main())
