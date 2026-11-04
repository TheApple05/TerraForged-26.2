"""Exercise real editor/world/render paths with a pinned content/renderer stack."""
from __future__ import annotations
import argparse
import datetime as dt
import hashlib
import json
from pathlib import Path
import sqlite3
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[2]


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def distant_data(instance):
    rows = []
    for file in instance.rglob("*.sqlite"):
        record = {"file": file.relative_to(ROOT).as_posix(), "bytes": file.stat().st_size, "sha256": sha(file)}
        with sqlite3.connect(file.as_uri() + "?mode=ro", uri=True) as database:
            tables = [row[0] for row in database.execute("SELECT name FROM sqlite_master WHERE type='table'")]
            record["tableRows"] = {name: database.execute('SELECT COUNT(*) FROM "' + name.replace('"', '""') + '"').fetchone()[0] for name in tables}
        rows.append(record)
    return rows


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--loader", choices=["fabric", "neoforge"], required=True)
    parser.add_argument("--java-home", type=Path, required=True)
    parser.add_argument("--jar", type=Path, required=True)
    parser.add_argument("--client-qa", type=Path, required=True)
    parser.add_argument("--vanilla-world", type=Path, required=True)
    parser.add_argument("--shader", action="store_true")
    parser.add_argument("--report", type=Path, required=True)
    args = parser.parse_args()
    if args.report.exists():
        raise ValueError("Refusing to replace prior evidence")
    expanded = json.loads((ROOT / "docs/validation/compatibility-26.2-artifacts.json").read_text())
    bop = json.loads((ROOT / "docs/validation/modpack-biomesoplenty-26.2-artifacts.json").read_text())
    selected = [row for row in expanded["artifacts"] if row["loader"] == args.loader and row["slug"] in
                ("sodium", "iris", "distanthorizons", "lithium", "ferrite-core", "repurposed-structures-fabric", "repurposed-structures-forge", "midnightlib")]
    selected += [row for row in bop["artifacts"] if row["loader"] == args.loader]
    if not any(row["slug"] == "distanthorizons" for row in selected):
        raise ValueError("Pinned Distant Horizons entry missing from selected stack")
    for row in selected:
        if sha(ROOT / row["file"]) != row["sha256"]:
            raise ValueError("Pinned mod bytes changed: " + row["file"])
    raw_report = ROOT / "build/qa/compat-client" / args.report.name
    raw_report.parent.mkdir(parents=True, exist_ok=True)
    tool = "production_client.py" if args.loader == "fabric" else "production_neoforge_client.py"
    command = [sys.executable, str(ROOT / "tools/qa" / tool), "--java-home", str(args.java_home), "--jar", str(args.jar),
               "--extra-mod", str(args.client_qa), "--qa-vanilla-world", str(args.vanilla_world), "--launch", "--client-qa",
               "--jvm-arg=-Dterraforged.clientqa.createFromUi=true", "--jvm-arg=-Dterraforged.clientqa.requireTranslations=true",
               "--jvm-arg=-Dterraforged.clientqa.requireBop=true", "--jvm-arg=-Dterraforged.clientqa.requireRenderReady=true",
               "--render-distance", "8", "--wait-seconds", "480", "--report", str(raw_report)]
    for row in selected:
        command += ["--extra-mod", str(ROOT / row["file"])]
    shader = None
    if args.shader:
        shader = next(row for row in expanded["artifacts"] if row.get("category") == "shaderpack")
        if sha(ROOT / shader["file"]) != shader["sha256"]:
            raise ValueError("Pinned shader pack bytes changed")
        command += ["--shader-pack", str(ROOT / shader["file"]), "--jvm-arg=-Dterraforged.clientqa.requireShader=true",
                    "--jvm-arg=-Dterraforged.clientqa.expectedShaderName=" + Path(shader["file"]).name]
    report = {"schemaVersion": 1, "status": "failed", "startedAtUtc": dt.datetime.now(dt.timezone.utc).isoformat(),
              "loader": args.loader, "mods": selected, "shader": shader, "command": command,
              "artifactSha256": sha(args.jar), "qaSha256": sha(args.client_qa),
              "scope": "Actual packaged client; real editor, new custom world, active shader when requested, same-process copied vanilla world, saves and clean exit. No FPS or infinite-distance visual verdict."}
    try:
        with (raw_report.parent / (args.report.stem + ".log")).open("w", encoding="utf-8") as log:
            result = subprocess.run(command, cwd=ROOT, stdout=log, stderr=subprocess.STDOUT, timeout=530)
        data = json.loads(raw_report.read_text())
        report["client"] = data
        qa = data.get("clientQa", {})
        if result.returncode or qa.get("status") != "passed" or not data.get("customTerrainValidated"):
            raise RuntimeError("Pinned client stack did not pass its real UI/terrain gates")
        instance = ROOT / data["launchPlan"]["instance"] if "launchPlan" in data else ROOT / data["instance"]
        report["distantHorizonsDatabases"] = distant_data(instance)
        report["status"] = "passed"
    except Exception as error:
        report["error"] = repr(error)
    report["finishedAtUtc"] = dt.datetime.now(dt.timezone.utc).isoformat()
    report["harnessSha256"] = sha(Path(__file__))
    args.report.parent.mkdir(parents=True, exist_ok=True)
    args.report.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({"status": report["status"], "report": str(args.report), "error": report.get("error")}))
    return 0 if report["status"] == "passed" else 1


if __name__ == "__main__":
    raise SystemExit(main())
