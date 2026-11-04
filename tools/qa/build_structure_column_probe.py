"""Build only the optional telemetry mixin against an older packaged candidate."""
from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile
import zipfile

ROOT = Path(__file__).resolve().parents[2]


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--java-home", type=Path, required=True)
    parser.add_argument("--production-jar", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    args.output.parent.mkdir(parents=True, exist_ok=True)
    source = ROOT / "tests/runtime/terraforged/qa/mixin/StructureColumnProbe.java"
    work = Path(tempfile.mkdtemp(prefix="compile-", dir=args.output.parent))
    classes = work / "classes"
    classes.mkdir()
    cp = (ROOT / "build/qa/target/compile-classpath.txt").read_text().strip()
    result = subprocess.run([str(args.java_home / "bin/javac.exe"), "--release", "25", "-proc:none", "-cp",
        str(args.production_jar.resolve()) + os.pathsep + cp, "-d", str(classes), str(source)],
        text=True, capture_output=True, timeout=90)
    (work / "compile.log").write_text(result.stdout + result.stderr)
    if result.returncode:
        print(result.stdout + result.stderr)
        return result.returncode
    with zipfile.ZipFile(args.output, "w", zipfile.ZIP_DEFLATED) as jar:
        jar.writestr("fabric.mod.json", json.dumps({"schemaVersion": 1, "id": "terraforged_column_profile", "version": "0.1.0",
            "name": "Test-only structure column telemetry", "environment": "server", "license": "MIT",
            "mixins": ["terraforged-column-profile.mixins.json"], "depends": {"reterraforged": "*", "minecraft": "26.2", "java": ">=25"}}))
        jar.writestr("terraforged-column-profile.mixins.json", json.dumps({"required": True, "minVersion": "0.8",
            "package": "terraforged.qa.mixin", "compatibilityLevel": "JAVA_25", "mixins": ["StructureColumnProbe"], "injectors": {"defaultRequire": 1}}))
        for file in classes.rglob("*.class"):
            jar.write(file, file.relative_to(classes).as_posix())
    report = {"jar": str(args.output), "sha256": hashlib.sha256(args.output.read_bytes()).hexdigest(),
        "productionJarSha256": hashlib.sha256(args.production_jar.read_bytes()).hexdigest(),
        "sourceSha256Lf": hashlib.sha256(source.read_bytes().replace(b"\r\n", b"\n")).hexdigest(),
        "scope": "Optional exact RuinedPortalStructure.findSuitableY getBaseColumn coordinates/timing only; no game behavior modifications"}
    (args.output.parent / "build-result.json").write_text(json.dumps(report, indent=2) + "\n")
    print(json.dumps(report, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
