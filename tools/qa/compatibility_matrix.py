"""Functional export/generate/reload checks for pinned loader-specific stacks.

These runs may overlap other work and must not be cited as controlled benchmarks.
"""
from __future__ import annotations
import argparse
import datetime as dt
import json
from pathlib import Path

from benchmark_matrix import ROOT, call, sha


def inventory(path):
    return {file.relative_to(path).as_posix(): sha(file) for file in sorted(path.rglob("*")) if file.is_file()}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--java-home", type=Path, required=True)
    parser.add_argument("--fabric-jar", type=Path, required=True)
    parser.add_argument("--neoforge-jar", type=Path, required=True)
    parser.add_argument("--fabric-qa", type=Path, required=True)
    parser.add_argument("--neoforge-qa", type=Path, required=True)
    parser.add_argument("--loader", choices=["both", "fabric", "neoforge"], default="both")
    parser.add_argument("--case", choices=["all", "optimized", "modernfix-community", "content-structures"], default="all")
    parser.add_argument("--label", required=True)
    parser.add_argument("--continue-on-failure", action="store_true", help="Continue independent cases after a failure; default stops for investigation")
    parser.add_argument("--report", type=Path, required=True)
    args = parser.parse_args()
    if args.report.exists():
        parser.error("Refusing to overwrite an existing compatibility summary; use a new label/report")
    manifest = json.loads((ROOT / "docs/validation/compatibility-26.2-artifacts.json").read_text())
    bop = json.loads((ROOT / "docs/validation/modpack-biomesoplenty-26.2-artifacts.json").read_text())
    cases = ["optimized", "modernfix-community", "content-structures"] if args.case == "all" else [args.case]
    report = {"schemaVersion": 1, "status": "running", "startedAtUtc": dt.datetime.now(dt.timezone.utc).isoformat(),
        "performanceEvidence": False, "scope": "Functional production export, own-stack preset consumption, actual generation and disk reload; optional actual structure checks",
        "cases": [], "notTested": ["All modpacks", "Client rendering", "Multiplayer", "Controlled performance"]}
    failures = []
    for loader in (["fabric", "neoforge"] if args.loader == "both" else [args.loader]):
        artifact = getattr(args, loader + "_jar").resolve()
        qa = getattr(args, loader + "_qa").resolve()
        for case in cases:
            row = {"loader": loader, "case": case, "status": "running", "artifactSha256": sha(artifact), "qaSha256": sha(qa)}
            report["cases"].append(row)
            try:
                slugs = ["modernfix-mvus"] if case == "modernfix-community" else ["c2me-fabric", "c2me-neoforge", "lithium", "ferrite-core"]
                if case == "content-structures": slugs += ["repurposed-structures-fabric", "repurposed-structures-forge", "midnightlib"]
                selected = [item for item in manifest["artifacts"] if item["loader"] == loader and item["slug"] in slugs]
                if case == "content-structures": selected += [item for item in bop["artifacts"] if item["loader"] == loader]
                mods = []
                for item in selected:
                    path = ROOT / item["file"]
                    if sha(path) != item["sha256"]: raise ValueError("Pinned dependency changed: " + str(path))
                    mods.append(path)
                row["extraMods"] = selected
                label = args.label + "-" + loader + "-" + case
                export_path, _, exported = call(loader, args, artifact, qa, "export", label + "-export", mods)
                row.update({"exportReport": export_path.relative_to(ROOT).as_posix(), "exportReportSha256": sha(export_path)})
                pack = Path(exported["export"]["presetPack"])
                hashes = inventory(pack)
                if hashes != exported["export"]["fileSha256"]: raise ValueError("Export files differ from actual QA inventory")
                mode = "structures" if case == "content-structures" else "generate"
                row["generationReport"] = "docs/validation/server-" + label + "-generation.json"
                gen_path, generated, result = call(loader, args, artifact, qa, mode, label + "-generation", mods, pack=pack)
                consumed = ROOT / generated["instance"] / "qa-world/datapacks/terraforged-qa"
                if inventory(consumed) != hashes: raise ValueError("Consumed pack differs from same-stack exported pack")
                reload_path, reloaded, reload_result = call(loader, args, artifact, qa, "reload", label + "-reload", mods,
                    reuse=ROOT / generated["instance"])
                row.update({"status": "passed", "exportReport": export_path.relative_to(ROOT).as_posix(),
                    "generationReport": gen_path.relative_to(ROOT).as_posix(), "reloadReport": reload_path.relative_to(ROOT).as_posix(),
                    "exportReportSha256": sha(export_path), "generationReportSha256": sha(gen_path), "reloadReportSha256": sha(reload_path),
                    "exportedFiles": len(hashes), "consumedPackMatchesEveryExportedFile": True,
                    "generationDigest": result["generation"]["digestSha256"],
                    "reloadDigest": reload_result["generation"]["digestSha256"],
                    "registeredClassicVariants": result.get("classicBiomeChecks", {}).get("registeredVariants"),
                    "classicSourceCodecRoundTrip": result.get("classicBiomeChecks", {}).get("sourceCodecRoundTrip"),
                    "observedBiomes": sorted({item["biome"] for item in result["generation"]["samples"]}),
                    "structureChecks": result.get("structureChecks"), "structureReloadChecks": reload_result.get("structureReloadChecks")})
                if sha(artifact) != row["artifactSha256"] or sha(qa) != row["qaSha256"]: raise ValueError("Candidate or companion changed during the case")
            except Exception as failure:
                row.update({"status": "failed", "error": repr(failure)})
                failures.append({"loader": loader, "case": case, "error": repr(failure)})
            args.report.parent.mkdir(parents=True, exist_ok=True)
            args.report.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
            if failures and not args.continue_on_failure:
                break
        if failures and not args.continue_on_failure:
            break
    report.update({"status": "passed" if not failures else "failed", "failures": failures,
        "finishedAtUtc": dt.datetime.now(dt.timezone.utc).isoformat(), "harnessSha256": sha(Path(__file__))})
    args.report.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({"status": report["status"], "cases": [{"loader": row["loader"], "case": row["case"], "status": row["status"], "error": row.get("error")} for row in report["cases"]]}))
    return 0 if not failures else 1


if __name__ == "__main__": raise SystemExit(main())
