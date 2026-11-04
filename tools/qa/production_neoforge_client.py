"""Launch an isolated production NeoForge client with verified game libraries and offline QA identity."""
from __future__ import annotations

import argparse
import concurrent.futures
import datetime as dt
import hashlib
import json
import os
from pathlib import Path
import platform
import shutil
import subprocess
import tomllib
import uuid
import zipfile

import production_client as common
from production_neoforge_server import NEO, INSTALLER_SHA
from production_server_smoke import download, MC_METADATA, NO_WINDOW

ROOT = common.ROOT
BASE = ROOT / "build/qa/neoforge-client"
LAUNCHER = BASE / "launcher"


def prepare(java: Path) -> tuple[dict, dict, list[dict], dict]:
    installer_url = f"https://maven.neoforged.net/releases/net/neoforged/neoforge/{NEO}/neoforge-{NEO}-installer.jar"
    installer = download(installer_url, BASE / "downloads" / f"neoforge-{NEO}-installer.jar", INSTALLER_SHA)
    LAUNCHER.mkdir(parents=True, exist_ok=True)
    # This profile belongs only to the disposable QA installation. Existing
    # launchers/accounts/configuration are never opened for writing.
    private_profile = LAUNCHER / "launcher_profiles.json"
    if not private_profile.exists():
        private_profile.write_text('{"profiles":{}}\n')
    profile_file = LAUNCHER / f"versions/neoforge-{NEO}/neoforge-{NEO}.json"
    patched = LAUNCHER / f"libraries/net/neoforged/minecraft-client-patched/{NEO}/minecraft-client-patched-{NEO}.jar"
    if not profile_file.is_file() or not patched.is_file():
        with (LAUNCHER / "installer.log").open("w", encoding="utf-8") as log:
            result = subprocess.run([str(java), "-jar", str(installer), "--installClient", str(LAUNCHER)],
                cwd=LAUNCHER, stdout=log, stderr=subprocess.STDOUT, timeout=240, creationflags=NO_WINDOW)
        if result.returncode:
            raise RuntimeError(f"NeoForge client installer exited {result.returncode}")
    profile = json.loads(profile_file.read_bytes())
    if profile.get("inheritsFrom") != "26.2" or profile.get("mainClass") != "net.neoforged.fml.startup.Client":
        raise ValueError("Unexpected installed NeoForge client profile")
    mc = common.metadata(MC_METADATA, "minecraft-26.2.json")
    if mc["id"] != "26.2" or mc["javaVersion"]["majorVersion"] != 25:
        raise ValueError("Unexpected official Minecraft descriptor")
    libraries = {}
    for role, source in (("minecraft-library", mc["libraries"]), ("neoforge-library", profile["libraries"])):
        for item in source:
            if common.applies(item.get("rules")) and item.get("downloads", {}).get("artifact"):
                group, artifact, version, *classifier = item["name"].split(":")
                libraries[(group, artifact, tuple(classifier))] = (role, item)
    def resolve(entry):
        role, item = entry
        descriptor = item["downloads"]["artifact"]
        path = common.verified(descriptor, item["name"], candidates=[LAUNCHER / "libraries" / descriptor["path"]])
        return {"role": role, "coordinate": item["name"], "path": common.relative(path), "sha256": common.hash_file(path)}
    with concurrent.futures.ThreadPoolExecutor(max_workers=4) as pool:
        runtime = list(pool.map(resolve, libraries.values()))
    client = common.verified(dict(mc["downloads"]["client"], path="minecraft/26.2/client.jar"),
        candidates=[LAUNCHER / "versions/26.2/26.2.jar"])
    runtime.append({"role": "minecraft-client", "coordinate": "com.mojang:minecraft:26.2",
        "path": common.relative(client), "sha256": common.hash_file(client)})
    prepared = {"minecraft": "26.2", "neoforge": NEO, "installerSha256": common.hash_file(installer),
        "matchesPublishedInstaller": True, "installedProfileSha256": common.hash_file(profile_file),
        "patchedClientSha256": common.hash_file(patched), "assets": common.assets(mc),
        "runtimeClasspath": runtime, "libraryCount": len(runtime), "libraryDirectory": common.relative(LAUNCHER / "libraries")}
    return mc, profile, runtime, prepared


def stage(args, mc, profile, runtime, prepared) -> tuple[Path, dict]:
    label = dt.datetime.now(dt.timezone.utc).strftime("%Y%m%dT%H%M%SZ") + "-" + uuid.uuid4().hex[:6]
    instance = BASE / "instances" / label
    mods = instance / "mods"
    mods.mkdir(parents=True)
    artifact = args.jar.resolve()
    with zipfile.ZipFile(artifact) as archive:
        mod = tomllib.loads(archive.read("META-INF/neoforge.mods.toml").decode())["mods"][0]
    shutil.copy2(artifact, mods / artifact.name)
    extras = []
    for source in args.extra_mod:
        source = source.resolve()
        destination = mods / source.name
        if destination.exists():
            raise ValueError("Duplicate mod filename: " + source.name)
        shutil.copy2(source, destination)
        extras.append({"path": common.relative(source), "sha256": common.hash_file(source)})
    worlds = {}
    for name, source in (("TerraForgedQa", args.qa_world), ("TerraForgedVanillaQa", args.qa_vanilla_world)):
        if source:
            source = source.resolve()
            if not (source / "level.dat").is_file():
                raise ValueError("QA world must contain level.dat")
            destination = instance / "saves" / name
            shutil.copytree(source, destination, ignore=shutil.ignore_patterns("session.lock"))
            worlds[name] = {"source": common.relative(source), "sourceLevelDatSha256": common.hash_file(source / "level.dat")}
    natives = instance / "natives/java"
    natives.mkdir(parents=True)
    native_classifier = "natives-windows-arm64" if platform.machine().lower() in ("arm64", "aarch64") else "natives-windows"
    native_records = []
    for entry in runtime:
        if entry["coordinate"].split(":")[-1] != native_classifier:
            continue
        with zipfile.ZipFile(ROOT / entry["path"]) as archive:
            for name in archive.namelist():
                if name.lower().endswith(".dll"):
                    data = archive.read(name)
                    destination = natives / Path(name).name
                    expected = hashlib.sha256(data).hexdigest()
                    if destination.exists() and common.hash_file(destination) != expected:
                        raise ValueError("Conflicting native DLL: " + destination.name)
                    destination.write_bytes(data)
                    native_records.append({"name": destination.name, "sha256": expected})
    replacements = {"auth_player_name": "TerraForgedQa", "version_name": profile["id"],
        "game_directory": instance, "assets_root": ROOT / prepared["assets"]["root"],
        "assets_index_name": prepared["assets"]["index"], "auth_uuid": uuid.uuid3(uuid.NAMESPACE_DNS, "OfflinePlayer:TerraForgedQa").hex,
        "auth_access_token": "0", "clientid": "0", "auth_xuid": "0", "version_type": "release",
        "natives_directory": instance / "natives", "launcher_name": "TerraForgedProductionQA", "launcher_version": "1",
        "classpath": os.pathsep.join(str(ROOT / item["path"]) for item in runtime),
        "library_directory": LAUNCHER / "libraries", "resolution_width": 1280, "resolution_height": 720}
    features = {"has_custom_resolution": True}
    arguments = ["-Xms512M", "-Xmx3G", "-Xlog:class+load=info:file=class-load.log:level,tags"]
    if args.client_qa:
        arguments += ["-Dterraforged.clientqa.output=" + str(instance / "qa-output"),
            "-Dterraforged.clientqa.expectedModSha256=" + common.hash_file(artifact)]
        if args.qa_world:
            arguments.append("-Dterraforged.clientqa.world=TerraForgedQa")
        if args.qa_vanilla_world:
            arguments.append("-Dterraforged.clientqa.vanillaWorld=TerraForgedVanillaQa")
    arguments += args.jvm_arg
    arguments += common.argument_values(mc["arguments"]["jvm"], replacements, features)
    arguments += common.argument_values(profile["arguments"]["jvm"], replacements, features)
    arguments += [profile["mainClass"]]
    arguments += common.argument_values(mc["arguments"]["game"], replacements, features)
    arguments += common.argument_values(profile["arguments"]["game"], replacements, features)
    argfile = instance / "java.args"
    argfile.write_text("\n".join('"' + item.replace("\\", "\\\\").replace('"', '\\"') + '"' for item in arguments) + "\n")
    (instance / "options.txt").write_text(f"fullscreen:false\nrenderDistance:{args.render_distance}\nsimulationDistance:5\nmaxFps:60\nonboardAccessibility:false\ntutorialStep:none\n")
    plan = {"instance": common.relative(instance), "artifact": common.relative(artifact), "modId": mod["modId"],
        "modVersion": mod["version"], "artifactSha256": common.hash_file(artifact), "extraMods": extras,
        "copiedWorlds": worlds, "nativeDlls": native_records, "offlineIdentity": True, "developmentMode": False,
        "argfile": common.relative(argfile)}
    (instance / "launch-plan.json").write_text(json.dumps(plan, indent=2) + "\n")
    return instance, plan


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--java-home", type=Path, required=True)
    parser.add_argument("--jar", type=Path, required=True)
    parser.add_argument("--extra-mod", action="append", type=Path, default=[])
    parser.add_argument("--jvm-arg", action="append", default=[])
    parser.add_argument("--qa-world", type=Path)
    parser.add_argument("--qa-vanilla-world", type=Path)
    parser.add_argument("--client-qa", action="store_true")
    parser.add_argument("--launch", action="store_true")
    parser.add_argument("--wait-seconds", type=int, default=180)
    parser.add_argument("--render-distance", type=int, default=6)
    parser.add_argument("--report", type=Path, required=True)
    args = parser.parse_args()
    if not 2 <= args.render_distance <= 32:
        parser.error("Render distance must be between 2 and 32 chunks")
    report = {"schemaVersion": 1, "result": "pending", "loader": "neoforge", "clientLaunched": False,
        "uiReviewed": False, "customTerrainValidated": False}
    def save():
        args.report.parent.mkdir(parents=True, exist_ok=True)
        args.report.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    try:
        java = args.java_home.resolve() / "bin/java.exe"
        mc, profile, runtime, prepared = prepare(java)
        report["installation"] = prepared
        instance, plan = stage(args, mc, profile, runtime, prepared)
        report.update({"launchPlan": plan, "result": "staged-not-launched"})
        save()
        if args.launch:
            environment = os.environ.copy()
            for variable in ("JAVA_TOOL_OPTIONS", "JDK_JAVA_OPTIONS", "_JAVA_OPTIONS"):
                environment.pop(variable, None)
            with (instance / "launcher.log").open("w", encoding="utf-8") as log:
                process = subprocess.Popen([str(java), "@" + str(instance / "java.args")], cwd=instance,
                    stdout=log, stderr=subprocess.STDOUT, env=environment, creationflags=NO_WINDOW)
                report.update({"clientLaunched": True, "pid": process.pid, "result": "running",
                    "rawLog": common.relative(instance / "launcher.log")})
                save()
                print("NEOFORGE_CLIENT_INSTANCE=" + common.relative(instance), flush=True)
                print("NEOFORGE_CLIENT_PID=" + str(process.pid), flush=True)
                try:
                    process.wait(timeout=args.wait_seconds)
                except subprocess.TimeoutExpired:
                    report["timedOut"] = True
                    process.terminate()
                    try:
                        process.wait(timeout=10)
                    except subprocess.TimeoutExpired:
                        process.kill()
                        process.wait(timeout=10)
                    report["forcedStop"] = True
            report["processExitCode"] = process.returncode
            qa_result = instance / "qa-output/result.json"
            qa = json.loads(qa_result.read_text()) if qa_result.exists() else None
            report["clientQa"] = qa
            class_log = (instance / "class-load.log").read_text(errors="replace")
            report["packagedOriginObserved"] = any("raccoonman.reterraforged.RTFCommon source:" in line
                and args.jar.name in line for line in class_log.splitlines())
            report["artifactUnchanged"] = common.hash_file(args.jar) == common.hash_file(instance / "mods" / args.jar.name)
            critical_markers = ("Reported exception thrown!", "Caught exception in thread", "Exception stopping the server",
                "Failed to save chunk", "Couldn't save chunk")
            latest_log = instance / "logs/latest.log"
            report["criticalRuntimeErrors"] = [line[:1200] for line in latest_log.read_text(errors="replace").splitlines()
                if any(marker in line for marker in critical_markers)] if latest_log.is_file() else []
            passed = process.returncode == 0 and not report.get("forcedStop") and report["packagedOriginObserved"] \
                and report["artifactUnchanged"] and not report["criticalRuntimeErrors"]
            if args.client_qa:
                passed = passed and bool(qa and qa.get("status") == "passed")
            terrain = qa.get("terrainRendering") if qa else None
            report["customTerrainValidated"] = bool(passed and isinstance(terrain, dict) and terrain.get("customGeneratorActive"))
            report["result"] = "passed" if passed else "failed"
    except Exception as error:
        report.update({"result": "failed", "error": repr(error)})
    report["finishedAt"] = dt.datetime.now(dt.timezone.utc).isoformat()
    report["limitations"] = ["Offline QA identity; no existing account or launcher files written",
        "Companion stages and process exit are distinct from screenshot review", "No claim of complete visual parity or arbitrary modpack compatibility"]
    save()
    print("NEOFORGE_CLIENT_RESULT=" + report["result"], flush=True)
    return 0 if report["result"] in ("staged-not-launched", "passed") else 1


if __name__ == "__main__":
    raise SystemExit(main())
