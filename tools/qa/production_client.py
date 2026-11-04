#!/usr/bin/env python3
"""Prepare or explicitly launch a disposable production client for UI or companion QA."""
from __future__ import annotations

import argparse
import concurrent.futures
import datetime as dt
import hashlib
import json
import os
from pathlib import Path
import platform
import re
import shutil
import subprocess
import threading
import time
import urllib.request
import uuid
import zipfile

from production_server_smoke import MC, LOADER, API, API_SHA256, MC_METADATA

ROOT = Path(__file__).resolve().parents[2]
CLIENT = ROOT / "build/qa/client"
CACHE = CLIENT / "cache"
GRADLE = Path.home() / ".gradle"
MINECRAFT = Path(os.environ.get("APPDATA", str(Path.home()))) / ".minecraft"
USER_AGENT = "TerraForged-Production-QA/1.0"


def hash_file(path: Path, algorithm="sha256") -> str:
    value = hashlib.new(algorithm)
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            value.update(block)
    return value.hexdigest()


def request(url: str) -> bytes:
    with urllib.request.urlopen(urllib.request.Request(url, headers={"User-Agent": USER_AGENT}), timeout=60) as response:
        return response.read()


def relative(path: Path) -> str:
    try:
        return path.resolve().relative_to(ROOT).as_posix()
    except ValueError:
        return str(path)


def metadata(url: str, name: str, expected_sha1=None) -> dict:
    path = CACHE / "metadata" / name
    path.parent.mkdir(parents=True, exist_ok=True)
    if not path.is_file() or (expected_sha1 and hash_file(path, "sha1") != expected_sha1):
        content = request(url)
        if expected_sha1 and hashlib.sha1(content).hexdigest() != expected_sha1:
            raise ValueError(f"Metadata checksum mismatch: {url}")
        path.write_bytes(content)
    return json.loads(path.read_bytes())


def maven_path(name: str) -> str:
    group, artifact, version, *classifier = name.split(":")
    suffix = "-" + classifier[0] if classifier else ""
    return f"{group.replace('.', '/')}/{artifact}/{version}/{artifact}-{version}{suffix}.jar"


def verified(descriptor: dict, name: str | None = None, candidates=()) -> Path:
    part = descriptor.get("path") or maven_path(name)
    target = CACHE / "libraries" / part
    expected = descriptor.get("sha256") or descriptor.get("sha1")
    algorithm = "sha256" if descriptor.get("sha256") else "sha1"
    url = descriptor["url"]
    if not url.endswith((".jar", ".xml")):
        url = url.rstrip("/") + "/" + part
    if not expected:
        expected = request(url + ".sha256").decode().strip().split()[0]
        algorithm = "sha256"
    locations = [target, MINECRAFT / "libraries" / part, *candidates]
    if name:
        group, artifact, version, *_ = name.split(":")
        locations += list((GRADLE / "caches/modules-2/files-2.1" / group / artifact / version).glob("*/" + Path(part).name))
    for path in locations:
        if path.is_file() and hash_file(path, algorithm) == expected:
            return path.resolve()
    content = request(url)
    if hashlib.new(algorithm, content).hexdigest() != expected:
        raise ValueError(f"Artifact checksum mismatch: {url}")
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_bytes(content)
    return target


def applies(rules, features=None) -> bool:
    if not rules:
        return True
    features = features or {}
    os_name = {"Windows": "windows", "Darwin": "osx", "Linux": "linux"}[platform.system()]
    architecture = {"amd64": "x86_64", "arm64": "aarch64"}.get(platform.machine().lower(), platform.machine().lower())
    allowed = False
    for rule in rules:
        system = rule.get("os", {})
        matches = system.get("name", os_name) == os_name
        if "arch" in system:
            matches &= re.fullmatch(system["arch"], architecture) is not None
        if "version" in system:
            matches &= re.search(system["version"], platform.version()) is not None
        for feature, expected in rule.get("features", {}).items():
            matches &= features.get(feature, False) == expected
        if matches:
            allowed = rule["action"] == "allow"
    return allowed


def assets(mc: dict) -> dict:
    descriptor = mc["assetIndex"]
    indexes = [
        GRADLE / "caches/fabric-loom/assets/indexes" / f"{descriptor['id']}.json",
        GRADLE / "caches/fabric-loom/assets/indexes" / f"{MC}-{descriptor['id']}.json",
        MINECRAFT / "assets/indexes" / f"{descriptor['id']}.json",
    ]
    target = CACHE / "assets/indexes" / f"{descriptor['id']}.json"
    target.parent.mkdir(parents=True, exist_ok=True)
    for path in [target, *indexes]:
        if path.is_file() and hash_file(path, "sha1") == descriptor["sha1"]:
            if path != target:
                shutil.copy2(path, target)
            break
    else:
        content = request(descriptor["url"])
        if hashlib.sha1(content).hexdigest() != descriptor["sha1"]:
            raise ValueError("Asset index checksum mismatch")
        target.write_bytes(content)
    index = json.loads(target.read_bytes())
    counts = {"existing": 0, "linkedOrCopied": 0, "downloaded": 0}
    lock = threading.Lock()

    def resolve(entry: dict):
        digest = entry["hash"]
        part = digest[:2] + "/" + digest
        destination = CACHE / "assets/objects" / part
        candidates = [destination, GRADLE / "caches/fabric-loom/assets/objects" / part, MINECRAFT / "assets/objects" / part]
        for source in candidates:
            if source.is_file() and hash_file(source, "sha1") == digest:
                outcome = "existing"
                if source != destination:
                    destination.parent.mkdir(parents=True, exist_ok=True)
                    try:
                        os.link(source, destination)
                    except OSError:
                        shutil.copy2(source, destination)
                    outcome = "linkedOrCopied"
                with lock:
                    counts[outcome] += 1
                return
        content = request(f"https://resources.download.minecraft.net/{part}")
        if hashlib.sha1(content).hexdigest() != digest:
            raise ValueError("Asset object checksum mismatch: " + digest)
        destination.parent.mkdir(parents=True, exist_ok=True)
        destination.write_bytes(content)
        with lock:
            counts["downloaded"] += 1

    # Duplicate asset names can reference the same object; resolve it just once.
    objects = {entry["hash"]: entry for entry in index["objects"].values()}
    with concurrent.futures.ThreadPoolExecutor(max_workers=4) as pool:
        list(pool.map(resolve, objects.values()))
    return {"root": relative(CACHE / "assets"), "index": descriptor["id"], "indexSha1": descriptor["sha1"], "objectsVerified": len(objects), **counts}


def prepare() -> tuple[dict, dict, list[dict], dict]:
    mc = metadata(MC_METADATA, f"minecraft-{MC}.json")
    if mc["id"] != MC or mc["javaVersion"]["majorVersion"] != 25:
        raise ValueError("Unexpected Minecraft target descriptor")
    profile = metadata(f"https://meta.fabricmc.net/v2/versions/loader/{MC}/{LOADER}/profile/json", f"fabric-{MC}-{LOADER}.json")
    if profile["inheritsFrom"] != MC or profile["mainClass"] != "net.fabricmc.loader.impl.launch.knot.KnotClient":
        raise ValueError("Unexpected Fabric production client profile")
    descriptors = []
    for library in mc["libraries"]:
        if applies(library.get("rules")) and library.get("downloads", {}).get("artifact"):
            descriptors.append(("minecraft-library", library["name"], library["downloads"]["artifact"]))
    for library in profile["libraries"]:
        if applies(library.get("rules")):
            descriptors.append(("fabric-library", library["name"], library))

    def resolve(item):
        role, coordinate, descriptor = item
        path = verified(descriptor, coordinate)
        return {"role": role, "coordinate": coordinate, "path": relative(path), "sha256": hash_file(path)}

    with concurrent.futures.ThreadPoolExecutor(max_workers=4) as pool:
        runtime = list(pool.map(resolve, descriptors))
    client = dict(mc["downloads"]["client"], path=f"minecraft/{MC}/client.jar")
    client_path = verified(client, candidates=[GRADLE / f"caches/fabric-loom/{MC}/minecraft-client.jar", MINECRAFT / f"versions/{MC}/{MC}.jar"])
    runtime.append({"role": "minecraft-client", "coordinate": f"com.mojang:minecraft:{MC}", "path": relative(client_path), "sha256": hash_file(client_path)})
    api_url = f"https://maven.fabricmc.net/net/fabricmc/fabric-api/fabric-api/{API}/fabric-api-{API}.jar"
    api_path = verified({"url": api_url, "path": f"fabric-api/{API}/fabric-api-{API}.jar", "sha256": API_SHA256}, f"net.fabricmc.fabric-api:fabric-api:{API}")
    prepared = {"minecraft": MC, "loader": LOADER, "fabricApi": API, "apiPath": relative(api_path), "apiSha256": hash_file(api_path), "assets": assets(mc), "libraryCount": len(runtime), "runtimeClasspath": runtime}
    if "client" in mc.get("logging", {}):
        logging = mc["logging"]["client"]
        path = verified(dict(logging["file"], path="logging/" + logging["file"]["id"]))
        prepared["logging"] = {"path": relative(path), "argument": logging["argument"], "sha256": hash_file(path)}
    return mc, profile, runtime, prepared


def argument_values(entries, replacements: dict, features: dict) -> list[str]:
    values = []
    for entry in entries:
        if isinstance(entry, str):
            selected = [entry]
        elif applies(entry.get("rules"), features):
            selected = entry["value"] if isinstance(entry["value"], list) else [entry["value"]]
        else:
            continue
        for value in selected:
            for key, replacement in replacements.items():
                value = value.replace("${" + key + "}", str(replacement))
            if "${" in value:
                raise ValueError("Unresolved launch argument: " + value)
            values.append(value)
    return values


def stage(args, mc, profile, runtime, prepared) -> tuple[Path, dict]:
    label = dt.datetime.now(dt.timezone.utc).strftime("%Y%m%dT%H%M%SZ") + "-" + uuid.uuid4().hex[:6]
    instance = CLIENT / "instances" / label
    instance.mkdir(parents=True)
    mods = instance / "mods"
    mods.mkdir()
    artifact = args.jar.resolve()
    with zipfile.ZipFile(artifact) as archive:
        mod = json.loads(archive.read("fabric.mod.json"))
    shutil.copy2(artifact, mods / artifact.name)
    api = ROOT / prepared["apiPath"]
    shutil.copy2(api, mods / api.name)
    extra_mods = []
    for extra in args.extra_mod:
        source = extra.resolve()
        destination = mods / source.name
        if destination.exists():
            raise ValueError("Duplicate installed mod filename: " + source.name)
        shutil.copy2(source, destination)
        extra_mods.append({"source": relative(source), "installed": relative(destination), "sha256": hash_file(source)})
    world_copy = None
    if args.qa_world:
        world_source = args.qa_world.resolve()
        if not (world_source / "level.dat").is_file():
            raise ValueError("--qa-world must name a closed QA world containing level.dat")
        world_destination = instance / "saves/TerraForgedQa"
        shutil.copytree(world_source, world_destination, ignore=shutil.ignore_patterns("session.lock"))
        world_copy = {"source": relative(world_source), "destination": relative(world_destination), "sourceLevelDatSha256": hash_file(world_source / "level.dat")}
    vanilla_world_copy = None
    if args.qa_vanilla_world:
        world_source = args.qa_vanilla_world.resolve()
        if not (world_source / "level.dat").is_file():
            raise ValueError("--qa-vanilla-world must name a closed QA world containing level.dat")
        world_destination = instance / "saves/TerraForgedVanillaQa"
        shutil.copytree(world_source, world_destination, ignore=shutil.ignore_patterns("session.lock"))
        vanilla_world_copy = {"source": relative(world_source), "destination": relative(world_destination), "sourceLevelDatSha256": hash_file(world_source / "level.dat")}
    natives = instance / "natives/java"
    natives.mkdir(parents=True)
    native_records = []
    # Mojang intentionally lists several Windows architectures under OS-only
    # rules. Keep those JARs on its declared classpath, but extract only the DLL
    # variant matching this Java process into the explicit native directory.
    machine = platform.machine().lower()
    native_classifier = "natives-windows-arm64" if machine in ("arm64", "aarch64") else "natives-windows-x86" if machine in ("x86", "i386", "i686") else "natives-windows"
    for entry in runtime:
        if entry["coordinate"].split(":")[-1] != native_classifier:
            continue
        with zipfile.ZipFile(ROOT / entry["path"]) as archive:
            for name in archive.namelist():
                if name.lower().endswith(".dll"):
                    data = archive.read(name)
                    destination = natives / Path(name).name
                    digest = hashlib.sha256(data).hexdigest()
                    if destination.exists() and hash_file(destination) != digest:
                        raise ValueError("Conflicting native DLL: " + destination.name)
                    destination.write_bytes(data)
                    native_records.append({"name": destination.name, "sha256": digest, "container": entry["coordinate"]})
    replacements = {
        "auth_player_name": "TerraForgedQa", "version_name": MC,
        "game_directory": instance, "assets_root": ROOT / prepared["assets"]["root"],
        "assets_index_name": prepared["assets"]["index"],
        "auth_uuid": uuid.uuid3(uuid.NAMESPACE_DNS, "OfflinePlayer:TerraForgedQa").hex,
        "auth_access_token": "0", "clientid": "0", "auth_xuid": "0", "version_type": "release",
        "natives_directory": instance / "natives", "launcher_name": "TerraForgedProductionQA", "launcher_version": "1",
        "classpath": os.pathsep.join(str(ROOT / entry["path"]) for entry in runtime),
        "resolution_width": 1280, "resolution_height": 720,
    }
    features = {"has_custom_resolution": True}
    jvm = ["-Xms512M", "-Xmx3G", "-Dfabric.development=false", "-Xlog:class+load=info:file=class-load.log:level,tags"]
    if args.client_qa:
        jvm += ["-Dterraforged.clientqa.output=" + str(instance / "qa-output"), "-Dterraforged.clientqa.expectedModSha256=" + hash_file(artifact)]
        if world_copy:
            jvm += ["-Dterraforged.clientqa.world=TerraForgedQa"]
        if vanilla_world_copy:
            jvm += ["-Dterraforged.clientqa.vanillaWorld=TerraForgedVanillaQa"]
    jvm += args.jvm_arg
    jvm += argument_values(mc["arguments"]["jvm"], replacements, features)
    jvm += argument_values(profile.get("arguments", {}).get("jvm", []), replacements, features)
    if "logging" in prepared:
        jvm.append(prepared["logging"]["argument"].replace("${path}", str(ROOT / prepared["logging"]["path"])))
    game = argument_values(mc["arguments"]["game"], replacements, features)
    game += argument_values(profile.get("arguments", {}).get("game", []), replacements, features)
    arguments = jvm + [profile["mainClass"]] + game
    argfile = instance / "java.args"
    argfile.write_text("\n".join('"' + value.replace("\\", "\\\\").replace('"', '\\"') + '"' for value in arguments) + "\n", encoding="utf-8")
    (instance / "options.txt").write_text(f"fullscreen:false\nrenderDistance:{args.render_distance}\nsimulationDistance:5\nmaxFps:60\nonboardAccessibility:false\ntutorialStep:none\n")
    plan = {"instance": relative(instance), "artifact": {"source": relative(artifact), "installed": relative(mods / artifact.name), "id": mod["id"], "version": mod["version"], "sha256": hash_file(artifact)}, "extraMods": extra_mods, "copiedQaWorld": world_copy, "copiedVanillaQaWorld": vanilla_world_copy, "javaArguments": arguments, "nativeDlls": native_records, "offlineIdentity": True, "developmentMode": False, "argfile": relative(argfile)}
    (instance / "launch-plan.json").write_text(json.dumps(plan, indent=2) + "\n")
    return instance, plan


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--java-home", type=Path, required=True)
    parser.add_argument("--jar", type=Path)
    parser.add_argument("--launch", action="store_true", help="Explicitly open the native game window and wait for manual closure.")
    parser.add_argument("--extra-mod", action="append", default=[], type=Path)
    parser.add_argument("--jvm-arg", action="append", default=[])
    parser.add_argument("--client-qa", action="store_true", help="Read the separate client QA companion's result; requires that companion via --extra-mod.")
    parser.add_argument("--qa-world", type=Path, help="Copy a closed QA world into this fresh isolated client's saves for optional integrated-world rendering.")
    parser.add_argument("--qa-vanilla-world", type=Path, help="Copy a closed vanilla QA world for a second same-process load after the custom world.")
    parser.add_argument("--render-distance", type=int, default=6, choices=range(2, 33), help="Render distance in this isolated QA instance.")
    parser.add_argument("--wait-seconds", type=int, default=900)
    parser.add_argument("--report", type=Path, required=True)
    args = parser.parse_args()
    if args.launch and not args.jar:
        parser.error("--launch requires --jar")
    report_path = ROOT / args.report
    report_path.parent.mkdir(parents=True, exist_ok=True)
    report = {"schemaVersion": 1, "startedAt": dt.datetime.now(dt.timezone.utc).isoformat(), "result": "pending", "clientLaunched": False, "uiReviewed": False, "customTerrainValidated": False}

    def save():
        report_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")

    try:
        java = args.java_home.resolve() / "bin" / ("java.exe" if os.name == "nt" else "java")
        version = subprocess.run([str(java), "-version"], capture_output=True, text=True, timeout=15, creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0))
        report["javaVersion"] = (version.stdout + version.stderr).strip()
        if version.returncode or not re.search(r'version "25(?:\.|\")', report["javaVersion"]):
            raise ValueError("Java 25 is required")
        mc, profile, runtime, prepared = prepare()
        report["installation"] = prepared
        report["result"] = "prepared-not-launched"
        if args.jar:
            instance, plan = stage(args, mc, profile, runtime, prepared)
            report["launchPlan"] = plan
            save()
        if args.launch:
            environment = os.environ.copy()
            for key in ("JAVA_TOOL_OPTIONS", "JDK_JAVA_OPTIONS", "_JAVA_OPTIONS"):
                environment.pop(key, None)
            log_path = instance / "launcher.log"
            with log_path.open("w", encoding="utf-8") as log:
                process = subprocess.Popen([str(java), "@" + str(instance / "java.args")], cwd=instance, stdout=log, stderr=subprocess.STDOUT, env=environment, creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0))
                report.update({"result": "running-awaiting-manual-ui-review", "clientLaunched": True, "pid": process.pid, "rawLog": relative(log_path), "classLoadLog": relative(instance / "class-load.log")})
                save()
                print("CLIENT_INSTANCE=" + relative(instance), flush=True)
                print("CLIENT_PID=" + str(process.pid), flush=True)
                try:
                    process.wait(timeout=args.wait_seconds)
                    report["processExitCode"] = process.returncode
                except subprocess.TimeoutExpired:
                    # The UI review belongs to the parent/user. Preserve its game
                    # window and owned PID; do not terminate an interactive review.
                    report["result"] = "still-running-manual-close-required"
                    report["processExitCode"] = None
                else:
                    report["result"] = "exited-cleanly-awaiting-ui-verdict" if process.returncode == 0 else "failed-client-runtime"
            class_log = instance / "class-load.log"
            origins = []
            if class_log.is_file():
                for line in class_log.read_text(encoding="utf-8", errors="replace").splitlines():
                    if "raccoonman.reterraforged." in line and " source: " in line:
                        origins.append(line.replace(ROOT.as_uri(), "<workspace>"))
            report["loadedModClassOrigins"] = origins[:40]
            report["packagedArtifactOriginObserved"] = any(args.jar.name in line for line in origins)
            report["artifactUnchanged"] = hash_file(args.jar.resolve()) == plan["artifact"]["sha256"] == hash_file(ROOT / plan["artifact"]["installed"])
            report["logTail"] = [line[:1200].replace(str(ROOT), "<workspace>") for line in log_path.read_text(encoding="utf-8", errors="replace").splitlines()[-50:]]
            critical_markers = ("Reported exception thrown!", "Caught exception in thread", "Exception stopping the server", "Failed to save chunk", "Couldn't save chunk")
            latest_log = instance / "logs/latest.log"
            report["criticalRuntimeErrors"] = [line[:1200] for line in latest_log.read_text(encoding="utf-8", errors="replace").splitlines() if any(marker in line for marker in critical_markers)] if latest_log.is_file() else []
            if args.client_qa:
                result_file = instance / "qa-output/result.json"
                report["clientQaResultFile"] = relative(result_file)
                if result_file.is_file():
                    qa = json.loads(result_file.read_text(encoding="utf-8"))
                    report["clientQa"] = qa
                    passed = qa.get("status") == "passed" and process.returncode == 0 and report["artifactUnchanged"] and report["packagedArtifactOriginObserved"] and not report["criticalRuntimeErrors"]
                    report["result"] = "passed-client-qa-awaiting-visual-review" if passed else "failed-client-qa"
                    report["customTerrainValidated"] = passed and qa.get("terrainRendering", {}).get("customGeneratorActive") is True
                elif process.returncode is not None:
                    report["result"] = "failed-client-qa-no-result"
                    progress_file = instance / "qa-output/progress.json"
                    if progress_file.is_file():
                        report["clientQaProgress"] = json.loads(progress_file.read_text(encoding="utf-8"))
    except Exception as error:
        report["result"] = "failed-client-setup"
        report["exception"] = f"{type(error).__name__}: {error}".replace(str(ROOT), "<workspace>")
    report["finishedAt"] = dt.datetime.now(dt.timezone.utc).isoformat()
    report["limitations"] = ["Offline QA identity; no launcher profile or account authentication accessed", "The separate QA companion requests actual save/disconnect and clean client shutdown; screenshot review is separate" if args.client_qa else "UI review and clean in-game closure are performed separately by the parent/user", "No visual parity verdict is inferred from process loading or exit"]
    save()
    print(json.dumps({key: report.get(key) for key in ("result", "pid", "processExitCode", "packagedArtifactOriginObserved", "exception")}, indent=2))
    print("REPORT=" + relative(report_path))
    return 1 if report["result"].startswith("failed-") else 0


if __name__ == "__main__":
    raise SystemExit(main())
