"""Run bounded fresh FULL/save/reload cases covering all seventeen actual variants."""
from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[2]


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--java-home", type=Path, required=True)
    parser.add_argument("--loader", choices=("fabric", "neoforge"), required=True)
    parser.add_argument("--candidate", required=True)
    parser.add_argument("--production-jar", type=Path, required=True)
    parser.add_argument("--qa-jar", type=Path, required=True)
    parser.add_argument("--datapack", type=Path, required=True)
    parser.add_argument("--pillar-targets", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--targets-manifest", type=Path, default=ROOT / "docs/validation/classic-biome-corpus-targets.json")
    args = parser.parse_args()
    if args.output.exists():
        parser.error("Corpus report already exists; use a new output/candidate label to preserve evidence")
    manifest = json.loads(args.targets_manifest.read_text())
    expected = {biome for group in manifest["groups"].values() for biome in group["biomes"]}
    if len(expected) != 17:
        parser.error("Corpus manifest must cover exactly seventeen distinct variants")
    production_hash, qa_hash = sha(args.production_jar), sha(args.qa_jar)
    report = {"schemaVersion": 1, "status": "running", "loader": args.loader, "candidate": args.candidate,
        "productionJar": str(args.production_jar), "productionJarSha256": production_hash,
        "qaJar": str(args.qa_jar), "qaJarSha256": qa_hash, "seed": manifest["seed"], "cases": [],
        "targetsManifest": str(args.targets_manifest), "targetsManifestSha256": sha(args.targets_manifest),
        "scope": "Six separate fresh worlds with two or three surveyed variant targets each; native FULL generation, source/stored agreement, forced save, and same-world reload column hashes. Default sixty-second watchdog remains enabled."}
    args.output.parent.mkdir(parents=True, exist_ok=True)

    def persist() -> None:
        args.output.write_text(json.dumps(report, indent=2) + "\n")

    persist()
    observed = set()
    harness = "production_server_smoke.py" if args.loader == "fabric" else "production_neoforge_server.py"
    try:
        for group, spec in manifest["groups"].items():
            targets = args.pillar_targets if group == "pillars" else ROOT / spec["path"]
            target_ids = set(json.loads(targets.read_text()))
            if target_ids != set(spec["biomes"]):
                raise ValueError(f"Target inventory changed for {group}")
            case = {"group": group, "targets": str(targets), "targetsSha256": sha(targets), "biomes": sorted(target_ids), "runs": []}
            report["cases"].append(case)
            instance = None
            for mode in ("generate", "reload"):
                path = args.output.parent / f"server-{args.loader}-{args.candidate}-classic-{group}-{mode}.json"
                if path.exists():
                    raise ValueError(f"Refusing to overwrite retained evidence: {path}")
                command = [sys.executable, str(ROOT / "tools/qa" / harness), "--java-home", str(args.java_home),
                    "--jar", str(args.production_jar), "--extra-mod", str(args.qa_jar), "--qa-mode", mode,
                    "--jvm-arg=-Dterraforged.qa.classicBiomeChunkTargets=" + str(targets.resolve()),
                    "--timeout", "240", "--report", str(path)]
                if mode == "generate":
                    command += ["--datapack", str(args.datapack)]
                    if group == "pillars":
                        command += ["--jvm-arg=-Dterraforged.qa.requirePillarEvidence=true", "--jvm-arg=-Dterraforged.qa.tracePillarPlacement=true"]
                else:
                    command += ["--reuse-instance", str(instance)]
                print(f"CLASSIC_CORPUS_START={args.loader}:{group}:{mode}", flush=True)
                process = subprocess.run(command, cwd=ROOT, capture_output=True, text=True, timeout=290)
                path.with_suffix(".runner.log").write_text(process.stdout + process.stderr)
                if not path.exists():
                    raise RuntimeError(f"Harness produced no report: {process.stdout[-2000:]} {process.stderr[-2000:]}")
                result = json.loads(path.read_text())
                status = result.get("result", result.get("status"))
                actual_hash = result.get("artifact", {}).get("sha256") if args.loader == "fabric" else result.get("artifactSha256")
                qa = result.get("qaResult", result.get("runtimeQa", {}))
                checks = qa.get("classicBiomeStoredChunkChecks", {})
                instance = ROOT / result["instance"]
                run = {"mode": mode, "report": str(path), "harnessExit": process.returncode, "status": status,
                    "instance": str(instance), "fullChunkTargets": checks.get("fullChunkTargets"), "observations": checks.get("observations"),
                    "reloadDigestsMatched": checks.get("reloadDigestsMatched"), "nativeSaveEverythingFlushCompleted": checks.get("nativeSaveEverythingFlushCompleted")}
                case["runs"].append(run)
                persist()
                if process.returncode or status != "passed" or actual_hash != production_hash or qa.get("status") != "passed":
                    raise RuntimeError(f"Native {args.loader} {group} {mode} failed or artifact changed: {path}")
                if set(row["biome"] for row in checks.get("observations", [])) != target_ids or checks.get("fullChunkTargets") != len(target_ids):
                    raise RuntimeError(f"Stored biome coverage differs for {group}")
                if mode == "reload" and not checks.get("reloadDigestsMatched"):
                    raise RuntimeError(f"Reload hashes did not match for {group}")
                if group == "pillars" and mode == "generate":
                    proof = checks.get("pillarSurfaceEvidence", [])
                    if len(proof) != 2 or any(not row.get("surfaceBaselineAvailable") or row.get("positiveAddedPillarColumns", 0) <= 0 for row in proof):
                        raise RuntimeError("Both fresh SURFACE-to-FULL pillar proofs are required")
                    case["pillarSurfaceEvidence"] = proof
                print(f"CLASSIC_CORPUS_PASS={args.loader}:{group}:{mode}", flush=True)
            observed.update(target_ids)
        if observed != expected or sha(args.production_jar) != production_hash or sha(args.qa_jar) != qa_hash:
            raise RuntimeError("Final coverage or artifact stability gate failed")
        report["status"] = "passed"
        report["actualStoredAndReloadedVariants"] = sorted(observed)
        report["variantCount"] = len(observed)
        persist()
        print(f"CLASSIC_CORPUS_COMPLETE={args.output}", flush=True)
        return 0
    except Exception as failure:
        report["status"] = "failed"
        report["error"] = repr(failure)
        persist()
        print(f"CLASSIC_CORPUS_FAILED={failure}", flush=True)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
