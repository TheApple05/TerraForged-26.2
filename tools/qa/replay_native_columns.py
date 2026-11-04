"""Replay exact captured RuinedPortal column inputs on old/final packaged loaders.

Compares complete native column hashes. Timings remain functional diagnostics;
this driver does not establish a controlled throughput improvement.
"""
from __future__ import annotations
import argparse
import datetime as dt
import json
from pathlib import Path
import benchmark_matrix as driver

ROOT = driver.ROOT


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--java-home", type=Path, required=True)
    parser.add_argument("--fabric-qa", type=Path, required=True)
    parser.add_argument("--neoforge-qa", type=Path, required=True)
    parser.add_argument("--candidate-version", default="0.1.0-dev.20+26.2")
    parser.add_argument("--label", default="native-columns-18-20")
    parser.add_argument("--report", type=Path, required=True)
    args = parser.parse_args()
    if args.report.exists(): parser.error("Refusing to replace native replay evidence")
    captured = ROOT / "docs/validation/classic-biome-dev18-ruined-portal-columns.json"
    driver.JVM.append("-Dterraforged.qa.replayBaseColumns=" + str(captured))
    report = {"schemaVersion": 1, "status": "running", "startedAtUtc": dt.datetime.now(dt.timezone.utc).isoformat(),
        "input": captured.relative_to(ROOT).as_posix(), "inputSha256": driver.sha(captured), "runs": [], "comparisons": [],
        "performanceEvidence": False, "scope": "Actual native getBaseColumn on exact captured RuinedPortal inputs; all384 block states per column compared; timings are diagnostic"}
    try:
        for loader in ("fabric", "neoforge"):
            exported = json.loads((ROOT / f"docs/validation/server-{loader}-dev18-export.json").read_text())
            qa_export = exported.get("qaResult", exported.get("runtimeQa"))
            pack = Path(qa_export["export"]["presetPack"])
            results = []
            for version, tag in (("0.1.0-dev.18+26.2", "old18"), (args.candidate_version, "candidate")):
                prefix = "neoforge/" if loader == "neoforge" else ""
                suffix = "-neoforge" if loader == "neoforge" else ""
                artifact = ROOT / f"{prefix}build/libs/terraforged-revival-{version}{suffix}.jar"
                qa = getattr(args, loader + "_qa")
                path, raw, result = driver.call(loader, args, artifact, qa, "generate", args.label + "-" + loader + "-" + tag,
                    [], pack=pack)
                replay = result["nativeColumnReplay"]
                report["runs"].append({"loader": loader, "version": version, "report": path.relative_to(ROOT).as_posix(),
                    "reportSha256": driver.sha(path), "artifactSha256": driver.sha(artifact), "replay": replay})
                results.append(replay)
                args.report.parent.mkdir(parents=True, exist_ok=True)
                args.report.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
            before, after = results
            if (before["seed"], before["minY"], before["maxYInclusive"]) != (after["seed"], after["minY"], after["maxYInclusive"]):
                raise ValueError("Native replay world bounds/seed changed")
            if len(before["columns"]) != len(after["columns"]): raise ValueError("Native replay column count changed")
            differences = []
            for old, new in zip(before["columns"], after["columns"]):
                if any(old[key] != new[key] for key in ("x", "z", "blockStatesHashed", "columnSha256")):
                    differences.append({"old": old, "new": new})
            report["comparisons"].append({"loader": loader, "columns": len(before["columns"]), "differences": differences,
                "allNativeColumnBlocksMatch": not differences,
                "oldDiagnosticNativeCallTotalNanos": sum(item["nativeCallElapsedNanos"] for item in before["columns"]),
                "newDiagnosticNativeCallTotalNanos": sum(item["nativeCallElapsedNanos"] for item in after["columns"])})
            if differences: raise ValueError("Native column block hashes changed for " + loader)
        report["status"] = "passed"
    except Exception as error:
        report.update({"status": "failed", "error": repr(error)})
    report["finishedAtUtc"] = dt.datetime.now(dt.timezone.utc).isoformat()
    report["harnessSha256"] = driver.sha(Path(__file__))
    args.report.parent.mkdir(parents=True, exist_ok=True)
    args.report.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({"status": report["status"], "runs": len(report["runs"]), "error": report.get("error")}))
    return 0 if report["status"] == "passed" else 1


if __name__ == "__main__": raise SystemExit(main())
