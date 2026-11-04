"""Build a separate Fabric runtime QA companion; never adds test code to production."""
from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import zipfile

ROOT = Path(__file__).resolve().parents[2]


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--java-home", required=True, type=Path)
    parser.add_argument("--classpath-file", type=Path, default=ROOT / "build/qa/target/compile-classpath.txt")
    parser.add_argument("--production-jar", type=Path, help="Compile against this packaged artifact instead of production classes")
    parser.add_argument("--output", type=Path, default=ROOT / "build/qa/runtime/terraforged-runtime-qa.jar")
    parser.add_argument("--noise-composition", action="store_true", help="Include the optional same-package genuine Java graph oracle (Fabric QA only).")
    parser.add_argument("--noise-shaping", action="store_true", help="Include the optional same-package genuine Java shaping oracle (Fabric QA only).")
    parser.add_argument("--noise-cellular", action="store_true", help="Include the optional same-package genuine Java cellular oracle (Fabric QA only).")
    parser.add_argument("--noise-final", action="store_true", help="Include the optional same-package genuine Java line/terrace/domain/erosion oracle (Fabric QA only).")
    parser.add_argument("--noise-visitors", action="store_true", help="Include the optional same-package genuine Java traversal oracle (Fabric QA only).")
    parser.add_argument("--noise-registry", action="store_true", help="Include the optional same-package genuine Java registry oracle (Fabric QA only).")
    args = parser.parse_args()
    output = args.output.resolve()
    output.parent.mkdir(parents=True, exist_ok=True)
    production = args.production_jar.resolve() if args.production_jar else ROOT / "build/classes/java/main"
    if not production.exists():
        parser.error("Build actual production classes/JAR first; no substitutes are generated")
    classpath = args.classpath_file.read_text(encoding="utf-8").strip()
    artifacts = [Path(value) for value in classpath.split(os.pathsep)]
    if not classpath or any(not artifact.exists() for artifact in artifacts):
        parser.error("Target classpath is absent/stale; refresh writePortClasspath when no Gradle process is active")
    javac = args.java_home.resolve() / "bin" / ("javac.exe" if os.name == "nt" else "javac")
    sources = sorted((ROOT / "tests/runtime").rglob("*.java"))
    sources.append(ROOT / "tests/terrain/TemplateBiomeDecoratorProbe.java")
    if args.noise_composition:
        sources.append(ROOT / "horizon/tests/java/NativeNoiseCompositionProbe.java")
    if args.noise_shaping:
        sources.append(ROOT / "horizon/tests/java/NativeNoiseShapingProbe.java")
    if args.noise_cellular:
        sources.append(ROOT / "horizon/tests/java/NativeNoiseCellularProbe.java")
    if args.noise_final:
        sources.append(ROOT / "horizon/tests/java/NativeNoiseFinalProbe.java")
    if args.noise_visitors:
        sources.append(ROOT / "horizon/tests/java/NativeNoiseVisitorProbe.java")
    if args.noise_registry:
        sources.append(ROOT / "horizon/tests/java/NativeNoiseRegistryProbe.java")
    work = Path(tempfile.mkdtemp(prefix="compile-", dir=output.parent))
    classes = work / "classes"
    classes.mkdir()
    result = subprocess.run(
        [str(javac), "--release", "25", "-proc:none", "-encoding", "UTF-8", "-cp",
         str(production) + os.pathsep + classpath, "-d", str(classes), *map(str, sources)],
        capture_output=True, text=True, cwd=ROOT, timeout=90)
    (work / "compile.log").write_text(result.stdout + result.stderr, encoding="utf-8")
    if result.returncode:
        print(result.stdout + result.stderr, file=sys.stderr)
        return result.returncode
    metadata = {
        "schemaVersion": 1, "id": "terraforged_runtime_qa", "version": "0.1.0",
        "name": "TerraForged runtime QA companion", "environment": "server", "license": "MIT",
        "description": "Test-only artifact, deliberately separate from the production mod.",
        "entrypoints": {"main": ["terraforged.qa.RuntimeQaMod"]},
        "mixins": ["terraforged-runtime-qa.mixins.json"],
        "depends": {"reterraforged": "*", "fabricloader": ">=0.20.1", "fabric-lifecycle-events-v1": "*", "minecraft": "26.2", "java": ">=25"},
    }
    with zipfile.ZipFile(output, "w", compression=zipfile.ZIP_DEFLATED) as archive:
        archive.writestr("fabric.mod.json", json.dumps(metadata, indent=2) + "\n")
        archive.writestr("terraforged-runtime-qa.mixins.json", json.dumps({"required": True, "minVersion": "0.8", "package": "terraforged.qa.mixin",
            "compatibilityLevel": "JAVA_25", "mixins": ["StructureColumnProbe", "ClassicPillarFeatureProbe"], "injectors": {"defaultRequire": 1}}, indent=2) + "\n")
        archive.writestr("META-INF/MANIFEST.MF", "Manifest-Version: 1.0\nImplementation-Title: Test-only TerraForged runtime QA\n\n")
        for file in sorted(classes.rglob("*.class")):
            archive.write(file, file.relative_to(classes).as_posix())
    report = {
        "schemaVersion": 1, "status": "built", "compileExit": result.returncode,
        "jar": str(output), "sha256": hashlib.sha256(output.read_bytes()).hexdigest(),
        "compileAgainst": str(production),
        "productionJarSha256": hashlib.sha256(production.read_bytes()).hexdigest() if production.is_file() else None,
        "sourceSha256Lf": {file.relative_to(ROOT).as_posix(): hashlib.sha256(file.read_bytes().replace(b"\r\n", b"\n")).hexdigest() for file in sources},
        "notEvidenceOf": ["Successful server launch", "Preset export", "Custom world generation", "Visual parity"],
    }
    (output.parent / "build-result.json").write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(report, indent=2))
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (OSError, ValueError, subprocess.SubprocessError) as exc:
        print(f"Runtime QA companion build error: {exc}", file=sys.stderr)
        raise SystemExit(2)
