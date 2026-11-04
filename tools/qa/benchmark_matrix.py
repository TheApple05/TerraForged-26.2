"""Run sequential, matched new-world generation trials against exact loader JARs.

The vanilla control keeps TerraForged loaded but has no custom datapack/context.
These are local bounded chunk-generation measurements, not multiplayer or FPS
benchmarks. A process must pass artifact provenance, watchdog and clean shutdown
gates before its timings enter the summary.
"""
from __future__ import annotations

import argparse
import datetime as dt
import hashlib
import json
from pathlib import Path
import platform
import statistics
import subprocess
import sys
import os

ROOT = Path(__file__).resolve().parents[2]
JVM = ["-Xms1G", "-Xmx4G", "-XX:+UseG1GC", "-XX:ActiveProcessorCount=8"]
BUDGET = {"maxNewRegionWallSeconds": 30.0, "maxPeakProcessResidentBytes": 6 * 1024 ** 3}


def hardware():
    result = {"logicalProcessors": os.cpu_count(), "machine": platform.machine(), "os": platform.platform()}
    if os.name == "nt":
        import ctypes
        import winreg
        class MemoryStatus(ctypes.Structure):
            _fields_ = [("length", ctypes.c_ulong), ("load", ctypes.c_ulong)] + [
                (name, ctypes.c_ulonglong) for name in ("totalPhysical", "availablePhysical", "totalPageFile",
                "availablePageFile", "totalVirtual", "availableVirtual", "availableExtendedVirtual")]
        memory = MemoryStatus()
        memory.length = ctypes.sizeof(memory)
        if ctypes.windll.kernel32.GlobalMemoryStatusEx(ctypes.byref(memory)):
            result.update({"physicalMemoryBytes": memory.totalPhysical, "availablePhysicalBytesBeforeMatrix": memory.availablePhysical})
        with winreg.OpenKey(winreg.HKEY_LOCAL_MACHINE, r"HARDWARE\DESCRIPTION\System\CentralProcessor\0") as key:
            result["cpuName"] = winreg.QueryValueEx(key, "ProcessorNameString")[0].strip()
    return result


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def call(loader, args, artifact, qa, mode, label, mods, pack=None, reuse=None, control=False):
    report = ROOT / "docs/validation" / ("server-" + label + ".json")
    if report.exists():
        raise ValueError("Refusing to replace prior evidence: " + str(report))
    tool = "production_server_smoke.py" if loader == "fabric" else "production_neoforge_server.py"
    command = [sys.executable, str(ROOT / "tools/qa" / tool), "--java-home", str(args.java_home),
        "--jar", str(artifact), "--extra-mod", str(qa), "--qa-mode", mode,
        "--report", str(report), "--timeout", "300"]
    for extra in mods:
        command += ["--extra-mod", str(extra)]
    for option in JVM + (["-Dterraforged.qa.vanillaControl=true"] if control else []):
        command.append("--jvm-arg=" + option)
    if pack:
        command += ["--datapack", str(pack)]
    if reuse:
        command += ["--reuse-instance", str(reuse)]
    print("START " + label, flush=True)
    result = subprocess.run(command, cwd=ROOT, timeout=350, capture_output=True, text=True, encoding="utf-8", errors="replace")
    run_log = ROOT / "build/qa/benchmark-driver" / (label + ".log")
    run_log.parent.mkdir(parents=True, exist_ok=True)
    run_log.write_text(result.stdout + result.stderr, encoding="utf-8")
    data = json.loads(report.read_text())
    passed = data.get("result", data.get("status")) == "passed" and result.returncode == 0
    qa_result = data.get("qaResult", data.get("runtimeQa")) or {}
    if not passed or qa_result.get("status") != "passed":
        raise RuntimeError("Trial failed; timings excluded; see " + str(report))
    if data.get("serverProperties", {}).get("max-tick-time") != 60000:
        raise ValueError("Normal watchdog not active")
    print("PASS " + label, flush=True)
    return report, data, qa_result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--java-home", type=Path, required=True)
    parser.add_argument("--fabric-jar", type=Path, required=True)
    parser.add_argument("--neoforge-jar", type=Path, required=True)
    parser.add_argument("--fabric-qa", type=Path, required=True)
    parser.add_argument("--neoforge-qa", type=Path, required=True)
    parser.add_argument("--fabric-pack", type=Path)
    parser.add_argument("--neoforge-pack", type=Path)
    parser.add_argument("--stack", choices=["base", "optimized", "content"], default="base")
    parser.add_argument("--trials", type=int, default=3)
    parser.add_argument("--label", required=True, help="Unique candidate/workload name, e.g. dev12-base")
    parser.add_argument("--loader", choices=["fabric", "neoforge", "both"], default="both")
    parser.add_argument("--report", type=Path, required=True)
    args = parser.parse_args()
    if args.report.exists():
        parser.error("Refusing to overwrite an existing benchmark summary; use a new label/report")
    if not 1 <= args.trials <= 5:
        parser.error("Use one to five bounded trials")
    if not args.label.replace("-", "").isalnum():
        parser.error("Label must be alphanumeric or hyphens")
    manifest = json.loads((ROOT / "docs/validation/compatibility-26.2-artifacts.json").read_text())
    runs = []
    report = {"schemaVersion": 1, "status": "running", "startedAtUtc": dt.datetime.now(dt.timezone.utc).isoformat(),
        "platform": platform.platform(), "hardware": hardware(), "python": platform.python_version(), "jvmOptions": JVM,
        "stack": args.stack, "acceptanceBudget": BUDGET, "trialsPerGeneratorPerLoader": args.trials, "runs": runs,
        "controlScope": "Same loader and extra mods, TerraForged loaded, custom preset/context absent",
        "notTested": ["FPS", "Multiplayer load", "All hardware", "All modpack combinations", "Unbounded exploration"]}
    args.report.parent.mkdir(parents=True, exist_ok=True)
    try:
        for loader in (["fabric", "neoforge"] if args.loader == "both" else [args.loader]):
            artifact = getattr(args, loader + "_jar").resolve()
            qa = getattr(args, loader + "_qa").resolve()
            artifact_sha = sha(artifact)
            qa_sha = sha(qa)
            selected = []
            if args.stack in ("optimized", "content"):
                selected += [row for row in manifest["artifacts"] if row["loader"] == loader and row["slug"] in
                    ("c2me-fabric", "c2me-neoforge", "lithium", "ferrite-core")]
            if args.stack == "content":
                selected += [row for row in manifest["artifacts"] if row["loader"] == loader and row["slug"] in
                    ("repurposed-structures-fabric", "repurposed-structures-forge", "midnightlib")]
                bop = json.loads((ROOT / "docs/validation/modpack-biomesoplenty-26.2-artifacts.json").read_text())
                selected += [row for row in bop["artifacts"] if row["loader"] == loader]
            mods = []
            for row in selected:
                path = ROOT / row["file"]
                if sha(path) != row["sha256"]:
                    raise ValueError("Extra mod hash changed: " + str(path))
                mods.append(path)
            pack = getattr(args, loader + "_pack")
            if pack is None:
                _, exported, result = call(loader, args, artifact, qa, "export", args.label + "-" + loader + "-export", mods)
                pack = Path(result["export"]["presetPack"])
            # Alternate controls and custom trials to reduce simple time/order bias.
            for trial in range(1, args.trials + 1):
                for control in ([False, True] if trial % 2 else [True, False]):
                    generator = "vanilla" if control else "custom"
                    label = f"{args.label}-{loader}-{generator}-trial{trial}"
                    path, raw, result = call(loader, args, artifact, qa, "benchmark", label, mods,
                        pack=None if control else pack, control=control)
                    run = {"loader": loader, "generator": generator, "trial": trial, "report": path.relative_to(ROOT).as_posix(),
                        "reportSha256": sha(path), "artifactSha256": artifact_sha, "qaSha256": qa_sha,
                        "phases": result["benchmarkPhases"], "environment": result["benchmarkEnvironment"],
                        "processMemory": raw["processMemory"], "extraMods": selected}
                    runs.append(run)
                    args.report.write_text(json.dumps(report, indent=2) + "\n")
                    if trial == 1:
                        reload_path, _, reload_result = call(loader, args, artifact, qa, "benchmark-reload", label + "-reload", mods,
                            reuse=ROOT / raw["instance"], control=control)
                        run["reloadReport"] = reload_path.relative_to(ROOT).as_posix()
                        run["reloadReportSha256"] = sha(reload_path)
                        run["reloadColumnsVerified"] = any(node["name"] == "benchmark-reload-columns" and node["status"] == "passed"
                            for node in reload_result["nodes"])
                        if not run["reloadColumnsVerified"]:
                            raise ValueError("Reload column gate was not observed")
                    if sha(artifact) != artifact_sha or sha(qa) != qa_sha:
                        raise ValueError("Artifacts changed during the benchmark matrix")
        groups = []
        for loader in sorted({run["loader"] for run in runs}):
            for generator in ("custom", "vanilla"):
                matched = [run for run in runs if run["loader"] == loader and run["generator"] == generator]
                group = {"loader": loader, "generator": generator, "trials": len(matched), "phaseSummaries": []}
                for phase in ("first-new-region", "resident-reread", "second-new-region"):
                    values = [next(item for item in run["phases"] if item["name"] == phase)["elapsedNanos"] / 1e9 for run in matched]
                    group["phaseSummaries"].append({"name": phase, "allSeconds": values, "medianSeconds": statistics.median(values),
                        "minSeconds": min(values), "maxSeconds": max(values), "medianRequestsPerSecond": 64 / statistics.median(values)})
                group["peakProcessResidentBytes"] = [run["processMemory"]["peakResidentBytes"] for run in matched]
                groups.append(group)
        report["summary"] = groups
        violations = []
        for run in runs:
            for phase in run["phases"]:
                if phase["name"].endswith("new-region") and phase["elapsedNanos"] / 1e9 > BUDGET["maxNewRegionWallSeconds"]:
                    violations.append({"report": run["report"], "phase": phase["name"], "reason": "New-region wall time exceeds the predeclared local budget"})
            peak = run["processMemory"]["peakResidentBytes"]
            if peak is None or peak > BUDGET["maxPeakProcessResidentBytes"]:
                violations.append({"report": run["report"], "reason": "Native process peak resident memory is unavailable or exceeds local budget"})
        report["budgetViolations"] = violations
        report["status"] = "passed" if not violations else "failed-performance-budget"
    except Exception as failure:
        report["status"] = "failed"
        report["error"] = repr(failure)
    report["finishedAtUtc"] = dt.datetime.now(dt.timezone.utc).isoformat()
    report["harnessSha256"] = sha(Path(__file__))
    args.report.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({"status": report["status"], "completedRuns": len(runs), "report": str(args.report), "error": report.get("error")}))
    return 0 if report["status"] == "passed" else 1


if __name__ == "__main__":
    raise SystemExit(main())
