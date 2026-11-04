"""Package both loaders and record coherent source/configuration and artifact fingerprints."""
from __future__ import annotations

import argparse
from concurrent.futures import ThreadPoolExecutor
import datetime as dt
import hashlib
import json
from pathlib import Path
import re
import subprocess
import zipfile

ROOT = Path(__file__).resolve().parents[2]


def snapshot(loader: str) -> dict:
    roots = [ROOT / "common/src", ROOT / ("fabric/src" if loader == "fabric" else "neoforge/src")]
    files = {path for root in roots for path in root.rglob("*") if path.is_file()}
    configs = ["gradle.properties", "gradlew", "gradlew.bat", "gradle/wrapper/gradle-wrapper.jar",
        "gradle/wrapper/gradle-wrapper.properties", "tools/gradle.ps1", "LICENSE", "THIRD_PARTY_NOTICES.md"]
    configs += ["build.gradle", "settings.gradle"] if loader == "fabric" else [
        "neoforge/build.gradle", "neoforge/settings.gradle", "neoforge/gradle.properties"]
    files.update(ROOT / name for name in configs)
    files.update(path for path in (ROOT / "licenses").rglob("*") if path.is_file())
    hashes = {path.relative_to(ROOT).as_posix(): hashlib.sha256(path.read_bytes().replace(b"\r\n", b"\n")
        if path.suffix in {".java", ".json", ".toml", ".gradle", ".cfg", ".accesswidener", ".properties", ".md", ".ps1", ".bat"}
        else path.read_bytes()).hexdigest() for path in sorted(files)}
    return {"files": len(hashes), "javaSources": sum(path.suffix == ".java" for path in files),
        "sha256": hashlib.sha256(json.dumps(hashes, sort_keys=True, separators=(",", ":")).encode()).hexdigest(),
        "normalization": "CRLF to LF for text source/configuration; filename plus content SHA-256 map"}


def quote(value: str) -> str:
    return "'" + value.replace("'", "''") + "'"


def build(loader: str, args) -> dict:
    before = snapshot(loader)
    task_args = (["--project-dir=neoforge"] if loader == "neoforge" else []) + [
        "jar", "portToolchain", "-PexperimentalPackage", "-Pmod_version=" + args.version, "--console=plain"]
    command = "& " + quote(str(ROOT / "tools/gradle.ps1")) + " -JavaHome " + quote(str(args.java_home.resolve()))
    command += " -GradleArgs @(" + ",".join(quote(item) for item in task_args) + ")"
    log = ROOT / f"build/qa/packages/{loader}-{args.version}.log"
    log.parent.mkdir(parents=True, exist_ok=True)
    with log.open("wb") as stream:
        result = subprocess.run(["powershell", "-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass",
            "-Command", command], cwd=ROOT, stdout=stream, stderr=subprocess.STDOUT, timeout=600)
    after = snapshot(loader)
    prefix = "neoforge/" if loader == "neoforge" else ""
    classifier = "-neoforge" if loader == "neoforge" else ""
    artifact = ROOT / f"{prefix}build/libs/terraforged-revival-{args.version}{classifier}.jar"
    report = {"loader": loader, "exitCode": result.returncode, "sourceBefore": before, "sourceAfter": after,
        "sourceChangedDuringRun": before != after, "log": log.relative_to(ROOT).as_posix(),
        "logSha256": hashlib.sha256(log.read_bytes()).hexdigest(), "gradleArguments": task_args}
    raw = log.read_text(encoding="utf-8", errors="replace")
    marker = "NEOFORGE_TOOLCHAIN_JSON=" if loader == "neoforge" else "PORT_TOOLCHAIN_JSON="
    toolchain = re.search(re.escape(marker) + "(.*)", raw)
    if toolchain:
        report["toolchain"] = json.loads(toolchain.group(1))
    if result.returncode == 0 and artifact.is_file():
        with zipfile.ZipFile(artifact) as archive:
            names = archive.namelist()
            provider = archive.read("META-INF/services/raccoonman.reterraforged.platform.PlatformServices").decode().strip()
            report["artifact"] = {"path": artifact.relative_to(ROOT).as_posix(),
                "sha256": hashlib.sha256(artifact.read_bytes()).hexdigest(), "bytes": artifact.stat().st_size,
                "classes": sum(name.endswith(".class") for name in names), "platformProviders": provider.splitlines(),
                "qaClassesPackaged": any(name.startswith("terraforged/qa/") for name in names),
                "languageEntries": len(json.loads(archive.read("assets/reterraforged/lang/en_us.json"))),
                "licenseNoticesPresent": "LICENSE" in names and "THIRD_PARTY_NOTICES.md" in names}
            if len(provider.splitlines()) != 1 or report["artifact"]["qaClassesPackaged"]:
                raise ValueError("Invalid platform provider count or QA classes in production package")
    report["status"] = "passed" if result.returncode == 0 and before == after and "artifact" in report else "failed"
    return report


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--java-home", required=True, type=Path)
    parser.add_argument("--version", required=True)
    parser.add_argument("--report", required=True, type=Path)
    args = parser.parse_args()
    if not re.fullmatch(r"[A-Za-z0-9.+_-]+", args.version):
        parser.error("Version must contain only alphanumeric characters and . + _ -")
    started = dt.datetime.now(dt.timezone.utc).isoformat()
    with ThreadPoolExecutor(max_workers=2) as pool:
        reports = list(pool.map(lambda loader: build(loader, args), ("fabric", "neoforge")))
    report = {"schemaVersion": 1, "startedAt": started, "version": args.version,
        "status": "passed" if all(item["status"] == "passed" for item in reports) else "failed",
        "experimental": True, "packages": reports, "notEvidenceOf": ["Runtime loading", "Custom generation",
            "Visual parity", "Third-party modpack compatibility"]}
    args.report.parent.mkdir(parents=True, exist_ok=True)
    args.report.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(report, indent=2))
    return 0 if report["status"] == "passed" else 1


if __name__ == "__main__":
    raise SystemExit(main())
