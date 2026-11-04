"""Verify a candidate's actual JARs against persisted client/server/image evidence."""
from __future__ import annotations
import argparse
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path, PureWindowsPath

ROOT = Path(__file__).resolve().parents[2]


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ValueError(message)


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def load(path: str) -> dict:
    return json.loads((ROOT / path).read_text(encoding="utf-8"))


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--manifest", type=Path, default=ROOT / "docs/validation/dev11-validation-inputs.json")
    parser.add_argument("--report", type=Path, default=ROOT / "docs/validation/dev11-acceptance.json")
    args = parser.parse_args()
    manifest = json.loads(args.manifest.read_text(encoding="utf-8"))
    build = load(manifest["buildReport"])
    require(build["status"] == "passed", "Candidate build did not pass")
    expected = {item["loader"]: item["artifact"] for item in build["packages"]}
    require(set(expected) == {"fabric", "neoforge"}, "Both loader artifacts are required")
    for item in build["packages"]:
        require(not item["sourceChangedDuringRun"], "Build source changed during packaging")
        require(digest(ROOT / item["artifact"]["path"]) == item["artifact"]["sha256"],
                "Local artifact no longer matches the validated candidate")
    clients = []
    for item in manifest["clients"]:
        data = load(item["report"])
        qa = data["clientQa"]
        runtime = qa["runtime"]
        require(data["result"].startswith("passed") and data["processExitCode"] == 0
                and qa["status"] == "passed", f"Client failed: {item['report']}")
        require(runtime["modSha256"] == expected[item["loader"]]["sha256"]
                and runtime["modVersion"] == build["version"] and runtime["minecraft"] == "26.2"
                and runtime["developmentEnvironment"] is False, "Wrong artifact or development runtime")
        require(data["artifactUnchanged"] and data["customTerrainValidated"]
                and not data["criticalRuntimeErrors"], "Artifact, terrain or runtime error gate failed")
        require(data.get("packagedArtifactOriginObserved") or data.get("packagedOriginObserved"),
                "Packaged production class origin was not observed")
        immediate = qa["immediatePostDone"]
        require(immediate["safeScreenOwnership"] and not immediate["parentVisible"],
                "Premature Create World exposure")
        require(qa["presetApplication"]["actualDoneButton"]
                and qa["terrainRendering"]["createdThroughActualUi"]
                and qa["terrainRendering"]["customGeneratorActive"]
                and not qa["vanillaTerrainRendering"]["customGeneratorActive"], "World transition gate failed")
        require(len(qa["screenshots"]) == 7, "Incomplete full-flow screenshot set")
        for shot in qa["screenshots"]:
            archived = ROOT / item["images"] / PureWindowsPath(shot["path"]).name
            require(digest(archived) == shot["sha256"], f"Archived screenshot bytes changed: {archived}")
        clients.append({**item, "status": "passed", "reportSha256": digest(ROOT / item["report"]),
                        "artifactSha256": runtime["modSha256"], "processExit": 0,
                        "safeImmediateDone": True, "archivedScreenshotsVerified": 7})
    require({(item["loader"], item["stack"]) for item in clients}
            == {(loader, stack) for loader in expected for stack in ("base", "bop-beta-stack")},
            "Expected four loader/stack client workflows")
    servers = load(manifest["serverReport"])
    require(servers["status"] == "passed" and not servers["pendingRuns"], "Dedicated checks incomplete")
    require(len(servers["stacks"]) == 2 and {item["loader"] for item in servers["stacks"]} == set(expected),
            "Dedicated evidence must cover both loaders")
    for item in servers["stacks"]:
        require(item["productionSha256"] == expected[item["loader"]]["sha256"]
                and item["allProcessesExitZero"] and item["reloadDigestMatches"]
                and item["normalWatchdogMillis"] == 60000
                and item["allConsumedPackFileHashesMatchValidatedExport"]
                and item["materialRuleMapCodecTypeAndIdentityVerified"], "Dedicated evidence gate failed")
    report = {"schemaVersion": 1, "checkedAtUtc": datetime.now(timezone.utc).isoformat(),
              "status": "passed-scoped-development-candidate", "minecraft": "26.2", "version": build["version"],
              "artifacts": list(expected.values()), "productionClients": clients,
              "dedicatedServerEvidence": manifest["serverReport"],
              "serverReportSha256": digest(ROOT / manifest["serverReport"]),
              "buildReportSha256": digest(ROOT / manifest["buildReport"]),
              "serverStatus": "passed-both-loaders-generation-and-reload", "stableRelease": False,
              "userVisualAcceptance": False, "checkerSha256": digest(Path(__file__)),
              "limits": ["Pinned beta content stack only; not all modpacks", "No complete classic feature/visual parity",
                         "No complete post-feature block determinism", "Windows runtime; Linux CI compilation is separate",
                         "Specific Neo camera/menu capture limitations remain in the visual-review notes"]}
    args.report.parent.mkdir(parents=True, exist_ok=True)
    args.report.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8", newline="\n")
    print("Verified 2 exact JARs, 4 production client workflows, 28 unchanged images and both server stacks.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
