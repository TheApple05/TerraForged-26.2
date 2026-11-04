#!/usr/bin/env python3
"""Install and test an isolated production Fabric server with an exact packaged JAR."""
from __future__ import annotations

import argparse
import datetime as dt
import hashlib
import json
import os
from pathlib import Path
import queue
import re
import shutil
import socket
import subprocess
import threading
import time
import urllib.request
import uuid
import zipfile

ROOT = Path(__file__).resolve().parents[2]
SERVER_ROOT = ROOT / "build/qa/server"
MC = "26.2"
LOADER = "0.20.1"
API = "0.161.0+26.2"
INSTALLER = "1.1.2"
INSTALLER_SHA256 = "61e035bf7bf70153e127440ce34de47c9036f0a2d0c65d1529454bd35ceefe4f"
API_SHA256 = "e5b858ceb13290c274e31cb888ff5a1a40cb91067e7ffab0b5b775aec51e216a"
MC_METADATA = "https://piston-meta.mojang.com/v1/packages/09165f570b16394790c4e37c93b0918729615d17/26.2.json"
NO_WINDOW = subprocess.CREATE_NO_WINDOW if os.name == "nt" else 0


def digest(path: Path, algorithm="sha256") -> str:
    return hashlib.new(algorithm, path.read_bytes()).hexdigest()


def relative(path: Path) -> str:
    try:
        return path.resolve().relative_to(ROOT).as_posix()
    except ValueError:
        return str(path)


def owned(path: Path) -> Path:
    resolved = path.resolve()
    if not resolved.is_relative_to(SERVER_ROOT.resolve()):
        raise ValueError("Server state must stay under build/qa/server")
    return resolved


def download(url: str, path: Path, expected: str, algorithm="sha256") -> Path:
    path.parent.mkdir(parents=True, exist_ok=True)
    if not path.is_file():
        with urllib.request.urlopen(url, timeout=60) as response, path.open("wb") as target:
            shutil.copyfileobj(response, target)
    if digest(path, algorithm) != expected:
        raise ValueError(f"Artifact checksum mismatch: {url}")
    return path


def prepare(java: Path) -> dict:
    downloads = SERVER_ROOT / "downloads"
    installer_url = f"https://maven.fabricmc.net/net/fabricmc/fabric-installer/{INSTALLER}/fabric-installer-{INSTALLER}.jar"
    api_url = f"https://maven.fabricmc.net/net/fabricmc/fabric-api/fabric-api/{API}/fabric-api-{API}.jar"
    installer = download(installer_url, downloads / f"fabric-installer-{INSTALLER}.jar", INSTALLER_SHA256)
    api = download(api_url, downloads / f"fabric-api-{API}.jar", API_SHA256)
    with urllib.request.urlopen(MC_METADATA, timeout=30) as response:
        metadata = json.load(response)
    if metadata["id"] != MC or metadata["javaVersion"]["majorVersion"] != 25:
        raise ValueError("Unexpected official Minecraft descriptor")
    template = owned(SERVER_ROOT / f"template-mc{MC}-loader{LOADER}")
    marker = template / ".install-complete.json"
    if not marker.is_file():
        template.mkdir(parents=True, exist_ok=True)
        command = [str(java), "-jar", str(installer), "server", "-mcversion", MC, "-loader", LOADER, "-downloadMinecraft", "-dir", str(template)]
        result = subprocess.run(command, cwd=ROOT, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True, encoding="utf-8", errors="replace", timeout=180, creationflags=NO_WINDOW)
        (template / "installer.log").write_text(result.stdout, encoding="utf-8")
        if result.returncode:
            raise RuntimeError(f"Fabric installer exited {result.returncode}: {result.stdout[-2500:]}")
        server = template / "server.jar"
        if digest(server, "sha1") != metadata["downloads"]["server"]["sha1"]:
            raise ValueError("Installed Minecraft server does not match Mojang's published SHA-1")
        if not (template / "fabric-server-launch.jar").is_file():
            raise ValueError("Fabric installer did not produce its production launcher")
        marker.write_text(json.dumps({"minecraft": MC, "loader": LOADER, "installer": INSTALLER, "serverSha1": digest(server, "sha1")}, indent=2) + "\n")
    if digest(template / "server.jar", "sha1") != metadata["downloads"]["server"]["sha1"]:
        raise ValueError("Cached server JAR checksum changed")
    return {
        "template": relative(template),
        "installer": {"version": INSTALLER, "url": installer_url, "sha256": digest(installer), "matchesPublished": True},
        "fabricApi": {"version": API, "url": api_url, "sha256": digest(api), "matchesPublished": True, "path": relative(api)},
        "minecraft": {"version": MC, "metadataUrl": MC_METADATA, "serverSha1": digest(template / "server.jar", "sha1"), "serverSha256": digest(template / "server.jar")},
        "loader": LOADER,
        "launcherSha256": digest(template / "fabric-server-launch.jar"),
    }


def free_port() -> int:
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as connection:
        connection.bind(("127.0.0.1", 0))
        return connection.getsockname()[1]


def mod_info(path: Path) -> dict:
    with zipfile.ZipFile(path) as archive:
        metadata = json.loads(archive.read("fabric.mod.json"))
    return {"name": path.name, "id": metadata["id"], "version": metadata["version"], "sha256": digest(path)}


def instance_for(args, prepared: dict) -> Path:
    if args.reuse_instance:
        instance = owned((ROOT / args.reuse_instance).resolve())
        marker = instance / ".terraforged-qa-instance.json"
        if not marker.is_file() or json.loads(marker.read_text()).get("createdBy") != "production_server_smoke.py":
            raise ValueError("Refusing to reuse a directory without this harness's QA-instance marker")
        return instance
    label = dt.datetime.now(dt.timezone.utc).strftime("%Y%m%dT%H%M%SZ") + "-" + uuid.uuid4().hex[:6]
    instance = owned(SERVER_ROOT / "instances" / label)
    shutil.copytree(ROOT / prepared["template"], instance)
    (instance / ".terraforged-qa-instance.json").write_text(json.dumps({"createdBy": "production_server_smoke.py", "minecraft": MC, "loader": LOADER}, indent=2) + "\n")
    return instance


def smoke(args, java: Path, prepared: dict, report: dict) -> None:
    artifact = args.jar.resolve()
    artifact_info = mod_info(artifact)
    instance = instance_for(args, prepared)
    run_label = dt.datetime.now(dt.timezone.utc).strftime("%Y%m%dT%H%M%SZ") + "-" + uuid.uuid4().hex[:4]
    run_dir = instance / "qa-runs" / run_label
    run_dir.mkdir(parents=True)
    mods = instance / "mods"
    mods.mkdir(exist_ok=True)
    installed = mods / artifact.name
    shutil.copy2(artifact, installed)
    shutil.copy2(ROOT / prepared["fabricApi"]["path"], mods / Path(prepared["fabricApi"]["path"]).name)
    extras = []
    for path in args.extra_mod:
        path = path.resolve()
        extras.append(mod_info(path))
        shutil.copy2(path, mods / path.name)
    qa_output = instance / "qa-output"
    qa_output.mkdir(exist_ok=True)
    qa_result = qa_output / "result.json"
    if qa_result.is_file():
        shutil.copy2(qa_result, run_dir / "previous-qa-result.json")
        qa_result.unlink()
    if args.datapack:
        source = args.datapack.resolve()
        target = instance / "qa-world/datapacks/terraforged-qa"
        if target.exists():
            raise ValueError("Datapack destination already exists; reuse without --datapack to reload that world")
        target.parent.mkdir(parents=True, exist_ok=True)
        if source.is_dir():
            shutil.copytree(source, target)
        else:
            shutil.copy2(source, target.with_suffix(".zip"))
    port = free_port()
    settings = {
        "server-ip": "127.0.0.1", "server-port": port,
        "online-mode": "false", "enable-query": "false", "enable-rcon": "false",
        "level-name": "qa-world", "level-seed": 8675309, "level-type": "minecraft:normal",
        "spawn-protection": 0, "view-distance": 2, "simulation-distance": 2,
        "max-players": 1, "sync-chunk-writes": "true", "pause-when-empty-seconds": -1,
        "max-tick-time": args.max_tick_time,
        "motd": "TerraForged isolated production QA",
    }
    if args.datapack:
        suffix = ".zip" if args.datapack.is_file() else ""
        settings["initial-enabled-packs"] = "vanilla,file/terraforged-qa" + suffix
    (instance / "server.properties").write_text("\n".join(f"{key}={value}" for key, value in settings.items()) + "\n")
    (instance / "eula.txt").write_text("# Authorized isolated disposable server QA\neula=true\n")
    class_log = run_dir / "class-load.log"
    class_log_arg = class_log.relative_to(instance).as_posix()
    command = [str(java), "-Xms512M", "-Xmx3G", "-Dfabric.development=false", f"-Xlog:class+load=info:file={class_log_arg}:level,tags"]
    command.extend(args.jvm_arg)
    if args.qa_mode:
        command.extend([f"-Dterraforged.qa.mode={args.qa_mode}", f"-Dterraforged.qa.output={qa_output}", f"-Dterraforged.qa.expectedModSha256={artifact_info['sha256']}"])
    command.extend(["-jar", "fabric-server-launch.jar", "nogui"])
    report.update({"instance": relative(instance), "artifact": artifact_info, "extraMods": extras, "serverProperties": settings, "bindAddress": "127.0.0.1", "port": port, "developmentMode": False, "qaMode": args.qa_mode, "qaOutput": relative(qa_output), "rawLog": relative(run_dir / "stdout.log"), "classLoadLog": relative(class_log), "customTerrainValidated": False, "featureParityValidated": False})
    print("SERVER_INSTANCE=" + relative(instance), flush=True)
    events: queue.Queue[str] = queue.Queue()
    lines: list[str] = []
    environment = os.environ.copy()
    for variable in ("JAVA_TOOL_OPTIONS", "JDK_JAVA_OPTIONS", "_JAVA_OPTIONS"):
        environment.pop(variable, None)
    process = subprocess.Popen(command, cwd=instance, stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True, encoding="utf-8", errors="replace", bufsize=1, creationflags=NO_WINDOW, env=environment)
    report["runtimeExecuted"] = True
    report["pid"] = process.pid
    print("SERVER_PID=" + str(process.pid), flush=True)
    launched_at = time.monotonic()
    ready_at = None
    dump_captured = False
    ready = False
    save_observed = False
    commands_sent = False
    stop_sent = False
    forced_stop = False
    qa_data = None
    deadline = time.monotonic() + args.timeout
    stop_deadline = None

    def reader():
        with (run_dir / "stdout.log").open("w", encoding="utf-8") as log:
            for line in process.stdout:
                log.write(line)
                log.flush()
                events.put(line.rstrip("\r\n"))

    thread = threading.Thread(target=reader, daemon=True)
    thread.start()

    def send(text: str):
        if process.poll() is None:
            try:
                process.stdin.write(text + "\n")
                process.stdin.flush()
            except (BrokenPipeError, OSError):
                report["stdinClosedDuringShutdown"] = True

    try:
        while process.poll() is None:
            try:
                line = events.get(timeout=0.2)
                lines.append(line)
                if re.search(r'Done \(.+\)! For help', line):
                    ready = True
                    ready_at = ready_at or time.monotonic()
                if "Saved the game" in line:
                    save_observed = True
            except queue.Empty:
                pass
            if args.qa_mode and qa_result.is_file() and qa_data is None:
                try:
                    qa_data = json.loads(qa_result.read_text(encoding="utf-8"))
                except (json.JSONDecodeError, OSError):
                    pass  # Atomic writer may still be replacing the result.
            if args.qa_mode in ("generate", "reload") and ready_at and qa_data is None and not dump_captured and time.monotonic() - ready_at >= 20:
                dump_captured = True
                dump_file = run_dir / "thread-dump-after-ready-20s.txt"
                jcmd = java.parent / ("jcmd.exe" if os.name == "nt" else "jcmd")
                try:
                    dump = subprocess.run([str(jcmd), str(process.pid), "Thread.print"], capture_output=True, text=True, encoding="utf-8", errors="replace", timeout=15, creationflags=NO_WINDOW)
                    dump_file.write_text(dump.stdout + dump.stderr, encoding="utf-8")
                    report["threadDumps"] = [{"path": relative(dump_file), "secondsAfterLaunch": round(time.monotonic() - launched_at, 2), "secondsAfterReady": round(time.monotonic() - ready_at, 2), "exitCode": dump.returncode}]
                    print("SERVER_THREAD_DUMP=" + relative(dump_file), flush=True)
                except (OSError, subprocess.TimeoutExpired) as error:
                    report["threadDumpError"] = str(error)
            if ready and not commands_sent and (not args.qa_mode or qa_data is not None):
                send("list")
                send("datapack list enabled")
                send("save-all flush")
                commands_sent = True
            if ready and save_observed and not stop_sent:
                send("stop")
                stop_sent = True
                stop_deadline = time.monotonic() + 30
            if time.monotonic() > deadline or (stop_deadline is not None and time.monotonic() > stop_deadline):
                report["timedOut"] = True
                if not stop_sent:
                    send("stop")
                    stop_sent = True
                try:
                    process.wait(timeout=15)
                except subprocess.TimeoutExpired:
                    process.terminate()  # Only the process started by this invocation.
                    forced_stop = True
                    try:
                        process.wait(timeout=10)
                    except subprocess.TimeoutExpired:
                        process.kill()
                        process.wait(timeout=10)
                break
    finally:
        if process.poll() is None:
            try:
                send("stop")
                process.wait(timeout=15)
            except (OSError, subprocess.TimeoutExpired):
                process.kill()
                process.wait(timeout=10)
                forced_stop = True
        thread.join(timeout=5)
        while not events.empty():
            lines.append(events.get_nowait())
        if process.stdin:
            process.stdin.close()
    log = "\n".join(lines)
    if qa_data is None and args.qa_mode and qa_result.is_file():
        qa_data = json.loads(qa_result.read_text(encoding="utf-8"))
    origins = []
    if class_log.is_file():
        for line in class_log.read_text(encoding="utf-8", errors="replace").splitlines():
            if "raccoonman.reterraforged." in line and " source: " in line:
                match = re.search(r'\] ([\w.$]+) source: (.+)$', line)
                if match:
                    origins.append({"class": match.group(1), "source": match.group(2).replace(ROOT.as_uri(), "<workspace>")})
    artifact_origin = any(installed.name in entry["source"] for entry in origins)
    target_game = bool(re.search(r"Loading Minecraft 26\.2 with Fabric Loader 0\.20\.1", log))
    reported_version = bool(re.search(r"Starting minecraft server version 26\.2", log, re.I))
    unchanged = digest(artifact) == artifact_info["sha256"] == digest(installed)
    qa_passed = not args.qa_mode or (qa_data is not None and qa_data.get("status") == "passed" and qa_data.get("mode") == args.qa_mode)
    companion_started = bool(qa_data and qa_data.get("serverStarted") and qa_data.get("mode") == args.qa_mode)
    companion_saved = bool(qa_data and "All chunks are saved" in log and (args.qa_mode == "export" or qa_data.get("saveCompleted")))
    companion_stopped = bool(args.qa_mode and "Stopping server" in log and process.returncode == 0)
    startup_reached = ready or companion_started
    saved = save_observed or companion_saved
    clean_stop = stop_sent or companion_stopped
    passed = startup_reached and saved and clean_stop and process.returncode == 0 and not forced_stop and artifact_origin and target_game and reported_version and unchanged and qa_passed and not report.get("timedOut")
    custom_gate = bool(qa_data and any(node.get("name") == "custom-generator-active" and node.get("status") == "passed" for node in qa_data.get("nodes", [])))
    generation = qa_data.get("generation", {}) if qa_data else {}
    custom_validated = bool(passed and args.qa_mode in ("generate", "reload") and custom_gate and generation.get("samples") and generation.get("chunks"))
    report.update({
        "result": "passed" if passed else "failed-runtime",
        "serverReady": ready, "saveObserved": save_observed, "stopSent": stop_sent,
        "companionServerStartedObserved": companion_started, "companionSaveCompleted": companion_saved,
        "companionCleanStopObserved": companion_stopped,
        "processExitCode": process.returncode, "forcedStop": forced_stop,
        "targetGameAndLoaderObserved": target_game, "serverVersionObserved": reported_version,
        "artifactUnchanged": unchanged, "packagedArtifactOriginObserved": artifact_origin,
        "loadedModClassCount": len(origins), "loadedModClassOrigins": origins[:40],
        "qaResult": qa_data,
        "customTerrainValidated": custom_validated,
        "customTerrainSampleCount": len(generation.get("samples", [])) if custom_validated else 0,
        "customTerrainChunkCount": len(generation.get("chunks", [])) if custom_validated else 0,
        "worldLevelDatExists": (instance / "qa-world/level.dat").is_file(),
        "regionFileCount": len(list((instance / "qa-world/region").glob("*.mca"))),
        "errorLines": [line.replace(str(ROOT), "<workspace>") for line in lines if re.search(r"/ERROR\]|/FATAL\]|Caused by:|Exception in thread|Mixin.*failed|InvalidInjection", line)][-50:],
        "tail": [line.replace(str(ROOT), "<workspace>")[:2048] for line in lines[-35:]],
        "limitations": ["Default vanilla world boot unless a generated preset datapack and companion QA mode were supplied", "No client rendering or visual terrain parity tested", "Only isolated loopback/offline QA server state was used"],
    })


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--java-home", type=Path, required=True)
    parser.add_argument("--jar", type=Path)
    parser.add_argument("--extra-mod", type=Path, action="append", default=[])
    parser.add_argument("--jvm-arg", action="append", default=[])
    parser.add_argument("--qa-mode", choices=("export", "generate", "reload"))
    parser.add_argument("--datapack", type=Path)
    parser.add_argument("--reuse-instance", type=Path)
    parser.add_argument("--timeout", type=int, default=300)
    parser.add_argument("--max-tick-time", type=int, default=60000, help="Server watchdog limit in milliseconds; use 0 only for bounded diagnostic runs.")
    parser.add_argument("--prepare-only", action="store_true")
    parser.add_argument("--report", type=Path, required=True)
    args = parser.parse_args()
    if not args.prepare_only and not args.jar:
        parser.error("--jar is required unless --prepare-only is set")
    if args.qa_mode and not args.extra_mod:
        parser.error("--qa-mode requires the separate companion QA JAR via --extra-mod")
    if args.timeout < 1 or args.timeout > 300:
        parser.error("--timeout must be between 1 and 300 seconds")
    if args.max_tick_time < 0:
        parser.error("--max-tick-time must be nonnegative")
    if any(value.startswith("-Dfabric.development=") and value != "-Dfabric.development=false" for value in args.jvm_arg):
        parser.error("Production QA cannot enable Fabric development mode")
    report_path = (ROOT / args.report).resolve()
    if not report_path.is_relative_to(ROOT / "docs/validation") or not report_path.name.startswith("server-") or report_path.suffix != ".json":
        parser.error("Reports must be docs/validation/server-*.json")
    report = {"schemaVersion": 1, "startedAt": dt.datetime.now(dt.timezone.utc).isoformat(), "result": "pending", "stage": "prepare", "runtimeExecuted": False}
    try:
        java = args.java_home.resolve() / "bin" / ("java.exe" if os.name == "nt" else "java")
        version = subprocess.run([str(java), "-version"], capture_output=True, text=True, timeout=15, creationflags=NO_WINDOW)
        report["javaVersionOutput"] = (version.stdout + version.stderr).strip()
        if version.returncode or not re.search(r'version "25(?:\.|\")', report["javaVersionOutput"]):
            raise ValueError("A Java 25 runtime is required")
        prepared = prepare(java)
        report["installation"] = prepared
        if args.prepare_only:
            report["result"] = "prepared-not-run"
        else:
            report["stage"] = "runtime"
            smoke(args, java, prepared, report)
    except Exception as error:
        report["result"] = "failed-" + report["stage"]
        report["exception"] = f"{type(error).__name__}: {error}".replace(str(ROOT), "<workspace>")
    report["finishedAt"] = dt.datetime.now(dt.timezone.utc).isoformat()
    report_path.parent.mkdir(parents=True, exist_ok=True)
    report_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    report_path.with_suffix(".md").write_text(
        "# Isolated production server QA\n\n"
        + f"Result: **{report['result']}**. Runtime executed: {report['runtimeExecuted']}.\n\n"
        + f"See the [machine-readable evidence]({report_path.name}) for artifact hashes, production installation, process exit, class origins and any companion checks.\n\n"
        + "A vanilla server boot establishes loading only. Custom terrain and visual parity require separate evidence.\n",
        encoding="utf-8",
    )
    summary = {key: report.get(key) for key in ("result", "stage", "instance", "artifact", "serverReady", "processExitCode", "packagedArtifactOriginObserved", "exception")}
    if report.get("qaResult"):
        result = report["qaResult"]
        summary["qaSummary"] = {"status": result.get("status"), "mode": result.get("mode"), "nodes": [node.get("name") for node in result.get("nodes", [])], "exportedFiles": result.get("export", {}).get("fileCount")}
    print(json.dumps(summary, indent=2))
    print("REPORT=" + relative(report_path))
    return 0 if report["result"] in ("passed", "prepared-not-run") else 1


if __name__ == "__main__":
    raise SystemExit(main())
