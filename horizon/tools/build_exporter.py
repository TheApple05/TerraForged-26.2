"""Build the developer-only Fabric snapshot exporter against an exact production JAR."""
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
    parser.add_argument("--java-home", required=True, type=Path)
    parser.add_argument("--jar", required=True, type=Path)
    parser.add_argument("--output", type=Path, default=ROOT / "build/qa/horizon/terraforged-horizon-exporter.jar")
    args = parser.parse_args()
    output = args.output.resolve()
    output.parent.mkdir(parents=True, exist_ok=True)
    work = Path(tempfile.mkdtemp(prefix="exporter-", dir=output.parent))
    source = ROOT / "horizon/tests/HorizonExportMod.java"
    classpath_file = ROOT / "build/qa/target/compile-classpath.txt"
    classpath = classpath_file.read_text().strip()
    javac = args.java_home / "bin" / ("javac.exe" if os.name == "nt" else "javac")
    result = subprocess.run([str(javac), "--release", "25", "-proc:none", "-encoding", "UTF-8", "-cp",
                             str(args.jar.resolve()) + os.pathsep + classpath, "-d", str(work), str(source)], capture_output=True, text=True)
    (work / "compile.log").write_text(result.stdout + result.stderr, encoding="utf-8")
    if result.returncode:
        print(result.stdout + result.stderr)
        return result.returncode
    metadata = {"schemaVersion": 1, "id": "terraforged_horizon_exporter", "version": "0.1.0", "environment": "server", "license": "MIT",
                "entrypoints": {"main": ["terraforged.horizon.HorizonExportMod"]},
                "depends": {"reterraforged": "*", "fabricloader": ">=0.20.1", "fabric-lifecycle-events-v1": "*", "minecraft": "26.2", "java": ">=25"}}
    with zipfile.ZipFile(output, "w", compression=zipfile.ZIP_DEFLATED) as archive:
        archive.writestr("fabric.mod.json", json.dumps(metadata, indent=2) + "\n")
        for file in work.rglob("*.class"):
            archive.write(file, file.relative_to(work).as_posix())
    report = {"jar": str(output), "sha256": hashlib.sha256(output.read_bytes()).hexdigest(),
              "sourceSha256": hashlib.sha256(source.read_bytes()).hexdigest(),
              "compileClasspathSha256": hashlib.sha256(classpath_file.read_bytes()).hexdigest(),
              "compilerVersion": subprocess.check_output([str(javac), "-version"], text=True).strip(),
              "productionJarSha256": hashlib.sha256(args.jar.read_bytes()).hexdigest()}
    output.with_suffix(".build.json").write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(report, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
