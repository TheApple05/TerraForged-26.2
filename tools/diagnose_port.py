#!/usr/bin/env python3
"""Run the complete 26.2 compiler diagnostic and preserve its actual exit status."""
from __future__ import annotations

import argparse
import collections
import datetime as dt
import hashlib
import json
from pathlib import Path
import re
import subprocess


ROOT = Path(__file__).resolve().parents[1]
SOURCE_ROOTS = ("common/src/main/java", "fabric/src/main/java")
PUBLISHED_WRAPPER_SHA256 = "497c8c2a7e5031f6aa847f88104aa80a93532ec32ee17bdb8d1d2f67a194a9c7"


def source_snapshot() -> dict:
    paths = sorted(p for folder in SOURCE_ROOTS for p in (ROOT / folder).rglob("*.java"))
    digest = hashlib.sha256()
    for path in paths:
        digest.update(path.relative_to(ROOT).as_posix().encode())
        digest.update(b"\0")
        digest.update(hashlib.sha256(path.read_bytes()).digest())
    return {"fileCount": len(paths), "sha256": digest.hexdigest()}


def subsystem(file: str) -> str:
    if file.startswith("fabric/"):
        return "fabric integration"
    for part, category in (
        ("/client/", "client UI and preview"),
        ("/mixin/", "Minecraft and optional integration mixins"),
        ("/data/", "data generation and presets"),
        ("/densityfunction/", "density functions"),
        ("/world/", "terrain and world generation"),
        ("/platform/", "platform dispatch"),
    ):
        if part in file:
            return category
    return "registries configuration and common bootstrap"


def parse_diagnostics(text: str) -> dict:
    # Gradle repeats javac diagnostics under its failure explanation. Count only
    # the first compiler section, retaining the complete raw log separately.
    text = text.replace("\r\n", "\n")
    initial = text.split("> Task :compileJava FAILED", 1)[0]
    rows = []
    lines = initial.splitlines()
    for index, line in enumerate(lines):
        match = re.match(r"^(.+\.java):(\d+): error: (.+)$", line)
        if not match:
            continue
        source = match.group(1)
        try:
            file = Path(source).relative_to(ROOT).as_posix()
        except ValueError:
            file = source.replace("\\", "/")
        details = []
        for extra in lines[index + 1:index + 8]:
            if re.match(r"^.+\.java:\d+: (?:error|warning):", extra):
                break
            if extra.strip().startswith(("symbol:", "location:", "required:", "found:", "reason:")):
                details.append(extra.strip())
        rows.append({"file": file, "line": int(match.group(2)), "message": match.group(3), "details": details})

    toolchain_match = re.search(r"^PORT_TOOLCHAIN_JSON=(.+)$", text, re.M)
    error_total = re.search(r"^(\d+) errors?$", initial, re.M)
    warning_total = re.search(r"^(\d+) warnings?$", initial, re.M)
    reported_errors = int(error_total.group(1)) if error_total else None
    return {
        "toolchain": json.loads(toolchain_match.group(1)) if toolchain_match else None,
        "javacReportedErrors": reported_errors,
        "capturedErrors": len(rows),
        "diagnosticCountMatchesJavac": len(rows) == reported_errors if reported_errors is not None else None,
        "javacReportedWarnings": int(warning_total.group(1)) if warning_total else None,
        "uniqueAffectedFiles": len({row["file"] for row in rows}),
        "errorHeadings": dict(collections.Counter(row["message"] for row in rows).most_common()),
        "errorsBySubsystem": dict(collections.Counter(subsystem(row["file"]) for row in rows).most_common()),
        "topAffectedFiles": dict(collections.Counter(row["file"] for row in rows).most_common(15)),
        "diagnostics": rows,
    }


def git_output(*arguments: str) -> str | None:
    result = subprocess.run(["git", *arguments], cwd=ROOT, capture_output=True, text=True, encoding="utf-8", errors="replace")
    return result.stdout.strip() if result.returncode == 0 else None


def display_path(path: Path) -> str:
    try:
        return path.relative_to(ROOT).as_posix()
    except ValueError:
        return str(path)


def write_summary(path: Path, report: dict) -> None:
    outcome = "compiler diagnostic passed" if report["exitCode"] == 0 else "compiler diagnostic failed"
    categories = "\n".join(f"- {name}: {count}." for name, count in report["errorsBySubsystem"].items())
    content = f"""# 26.2 compiler diagnostic

Recorded: {report['startedAt']}. Result: **{outcome}**, exit code {report['exitCode']}.

Command: `{report['reproductionCommand']}`

Source files: {report['sourceBefore']['fileCount']}. Captured errors: {report['capturedErrors']}; javac reported errors: {report['javacReportedErrors']}; warnings: {report['javacReportedWarnings']}; affected files: {report['uniqueAffectedFiles']}.

Source changed during the run: {report['sourceChangedDuringRun']}. A changed snapshot requires a new run after edits settle before using this as a coherent build result.

## Compiler categories

{categories or '- No source-error categories captured; inspect the result classification and raw log.'}

## Evidence and limitations

The [JSON report]({path.with_suffix('.json').name}) includes the actual resolved toolchain/classpath, source fingerprints, error headings and relative-path diagnostics. The raw log remains at `{report['rawLog']}` (SHA-256 `{report['rawLogSha256']}`). Gradle can stop before javac if configuration or dependency resolution fails; the result classification distinguishes that from source errors.

All common and Fabric Java sources are included. Legacy access widening is unregistered, annotation processing is disabled and ExpectPlatform weaving is unresolved. Packaging remains guarded. This diagnostic does not prove mixin application, runtime loading, world generation or feature parity; see [decision 0001](../decisions/0001-build-layout.md).

Error totals are a compiler-pass inventory. Resolving missing types can expose additional errors, so a smaller count is not a fixed success gate. Only an exit-zero compiler result establishes compilation within this deliberately limited diagnostic scope.
"""
    path.write_text(content, encoding="utf-8")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--report", type=Path, required=True, help="Output JSON report; a sibling Markdown summary is also written.")
    parser.add_argument("--log", type=Path, default=Path("build/qa/target/compile.log"), help="Raw combined stdout/stderr log.")
    parser.add_argument("--java-home", type=Path, help="Explicit full Java 25 JDK directory.")
    args = parser.parse_args()
    if args.report.suffix.lower() != ".json":
        parser.error("--report must end in .json")
    report_path = (ROOT / args.report).resolve()
    log_path = (ROOT / args.log).resolve()
    report_path.parent.mkdir(parents=True, exist_ok=True)
    log_path.parent.mkdir(parents=True, exist_ok=True)
    command = ["powershell", "-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-File", str(ROOT / "tools/gradle.ps1")]
    if args.java_home:
        command += ["-JavaHome", str(args.java_home.resolve())]
    command += ["portToolchain", "compileJava", "--console=plain"]
    before = source_snapshot()
    started = dt.datetime.now().astimezone().isoformat(timespec="seconds")
    execution_error = None
    try:
        with log_path.open("wb") as log:
            result = subprocess.run(command, cwd=ROOT, stdout=log, stderr=subprocess.STDOUT)
        exit_code = result.returncode
    except OSError as error:
        execution_error = str(error)
        exit_code = 2
        log_path.write_text(execution_error + "\n", encoding="utf-8")
    raw = log_path.read_bytes()
    encoding = "utf-16" if b"\x00" in raw[:120] else "utf-8-sig"
    parsed = parse_diagnostics(raw.decode(encoding, errors="replace"))
    after = source_snapshot()
    classification = (
        "failed-to-execute" if execution_error else
        "compiled-diagnostic-scope" if exit_code == 0 else
        "failed-source-compilation" if parsed["capturedErrors"] else
        "failed-before-source-diagnostics"
    )
    wrapper_hash = hashlib.sha256((ROOT / "gradle/wrapper/gradle-wrapper.jar").read_bytes()).hexdigest()
    report = {
        "schemaVersion": 1,
        "startedAt": started,
        "finishedAt": dt.datetime.now().astimezone().isoformat(timespec="seconds"),
        "result": classification,
        "exitCode": exit_code,
        "executionError": execution_error,
        "reproductionCommand": "python tools/diagnose_port.py --report " + display_path(report_path),
        "gradleArguments": ["portToolchain", "compileJava", "--console=plain"],
        "gitHead": git_output("rev-parse", "HEAD"),
        "dirtySourceFiles": git_output("status", "--porcelain", "--", *SOURCE_ROOTS),
        "sourceBefore": before,
        "sourceAfter": after,
        "sourceChangedDuringRun": before != after,
        "diagnosticScope": {
            "allCommonAndFabricSources": True, "sourceExclusions": [],
            "legacyAccessWidenerRegistered": False, "mixinAnnotationProcessing": False,
            "expectPlatformWeaving": False, "packagingGuardEnabled": True, "runtimeTested": False,
        },
        "wrapperVerification": {
            "sha256": wrapper_hash, "publishedSha256": PUBLISHED_WRAPPER_SHA256,
            "matchesPublished": wrapper_hash == PUBLISHED_WRAPPER_SHA256,
            "source": "https://services.gradle.org/distributions/gradle-9.5.1-wrapper.jar.sha256",
        },
        "rawLog": display_path(log_path),
        "rawLogSha256": hashlib.sha256(raw).hexdigest(),
        **parsed,
    }
    report_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    write_summary(report_path.with_suffix(".md"), report)
    print(f"{classification}: exit {exit_code}, {parsed['capturedErrors']} errors in {parsed['uniqueAffectedFiles']} files")
    print(f"Report: {display_path(report_path)}")
    print(f"Raw log: {display_path(log_path)}")
    # Preserve failure: callers must not confuse a successfully written report
    # with a successful compiler invocation.
    return exit_code if exit_code >= 0 else 1


if __name__ == "__main__":
    raise SystemExit(main())
