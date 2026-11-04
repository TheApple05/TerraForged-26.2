"""Install and exercise a packaged mod on an isolated production NeoForge server."""
from __future__ import annotations

import argparse
import datetime as dt
import json
import os
from pathlib import Path
import queue
import re
import shutil
import subprocess
import threading
import time
import uuid

from production_server_smoke import ROOT, NO_WINDOW, digest, download, free_port, relative

BASE = ROOT / "build/qa/neoforge-server"
NEO = "26.2.0.93"
INSTALLER_SHA = "7618883b9d87186281da2bd2a5e9a11af042ed15112bb63ccfc62bb0230ca699"


def prepare(java: Path) -> dict:
    import urllib.request
    from production_server_smoke import MC_METADATA
    url = f"https://maven.neoforged.net/releases/net/neoforged/neoforge/{NEO}/neoforge-{NEO}-installer.jar"
    installer = download(url, BASE / "downloads" / f"neoforge-{NEO}-installer.jar", INSTALLER_SHA)
    template = BASE / "template"
    template.mkdir(parents=True, exist_ok=True)
    args_file = template / f"libraries/net/neoforged/neoforge/{NEO}/win_args.txt"
    if not args_file.exists():
        result = subprocess.run([str(java), "-jar", str(installer), "--installServer", str(template)],
            cwd=template, capture_output=True, text=True, encoding="utf-8", errors="replace",
            timeout=240, creationflags=NO_WINDOW)
        (template / "installer.log").write_text(result.stdout + result.stderr, encoding="utf-8")
        if result.returncode or not args_file.is_file():
            raise RuntimeError(f"NeoForge installer failed: exit {result.returncode}")
    with urllib.request.urlopen(MC_METADATA, timeout=30) as response:
        metadata = json.load(response)
    server = template / "libraries/net/minecraft/server/26.2/server-26.2.jar"
    if metadata["id"] != "26.2" or digest(server, "sha1") != metadata["downloads"]["server"]["sha1"]:
            raise ValueError("Installed Minecraft does not match Mojang's published 26.2 artifact")
    return {"template": relative(template), "neoforge": NEO, "minecraft": "26.2",
        "installerUrl": url, "installerSha256": digest(installer), "matchesPublishedInstaller": True,
        "minecraftServerSha1": digest(server, "sha1"), "argsFile": args_file.relative_to(template).as_posix()}


def run(args, java: Path, prepared: dict, report: dict) -> None:
    if args.reuse_instance:
        instance = args.reuse_instance.resolve()
        if not instance.is_relative_to((BASE / "instances").resolve()) or not (instance / ".neo-qa-instance").is_file():
            raise ValueError("Only marked instances created by this harness can be reused")
    else:
        label = dt.datetime.now(dt.timezone.utc).strftime("%Y%m%dT%H%M%SZ") + "-" + uuid.uuid4().hex[:6]
        instance = BASE / "instances" / label
        shutil.copytree(ROOT / prepared["template"], instance)
        (instance / ".neo-qa-instance").write_text("production_neoforge_server.py\n")
    artifact = args.jar.resolve()
    mods = instance / "mods"
    mods.mkdir(exist_ok=True)
    installed = mods / artifact.name
    shutil.copy2(artifact, installed)
    extras = []
    for path in args.extra_mod:
        shutil.copy2(path, mods / path.name)
        extras.append({"path": relative(path), "sha256": digest(path)})
    output = instance / "qa-output"
    output.mkdir(exist_ok=True)
    result_file = output / "result.json"
    if result_file.exists():
        result_file.replace(output / ("previous-result-" + uuid.uuid4().hex[:6] + ".json"))
    if args.datapack:
        destination = instance / "qa-world/datapacks/terraforged-qa"
        destination.parent.mkdir(parents=True, exist_ok=True)
        shutil.copytree(args.datapack.resolve(), destination)
    settings = {"server-ip": "127.0.0.1", "server-port": free_port(), "online-mode": "false",
        "enable-query": "false", "enable-rcon": "false", "level-name": "qa-world", "level-seed": 8675309,
        "level-type": "minecraft:normal", "view-distance": 2, "simulation-distance": 2, "max-players": 1,
        "pause-when-empty-seconds": -1, "max-tick-time": 60000, "sync-chunk-writes": "true"}
    if args.datapack:
        settings["initial-enabled-packs"] = "vanilla,mod_data,file/terraforged-qa"
    (instance / "server.properties").write_text("\n".join(f"{key}={value}" for key, value in settings.items()) + "\n")
    (instance / "eula.txt").write_text("# Disposable isolated QA authorized for the port\neula=true\n")
    run_dir = instance / "qa-runs" / uuid.uuid4().hex[:10]
    run_dir.mkdir(parents=True)
    command = [str(java), "-Xms512M", "-Xmx3G", "-Xlog:class+load=info:file=" +
        (run_dir / "class-load.log").relative_to(instance).as_posix() + ":level,tags"]
    if args.qa_mode:
        command += [f"-Dterraforged.qa.mode={args.qa_mode}", f"-Dterraforged.qa.output={output}",
            "-Dterraforged.qa.expectedModSha256=" + digest(artifact)]
    command += ["@" + prepared["argsFile"], "nogui"]
    environment = os.environ.copy()
    for key in ("JAVA_TOOL_OPTIONS", "JDK_JAVA_OPTIONS", "_JAVA_OPTIONS"):
        environment.pop(key, None)
    process = subprocess.Popen(command, cwd=instance, stdin=subprocess.PIPE, stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT, text=True, encoding="utf-8", errors="replace", bufsize=1,
        env=environment, creationflags=NO_WINDOW)
    report.update({"instance": relative(instance), "pid": process.pid, "artifact": relative(artifact),
        "artifactSha256": digest(artifact), "extraMods": extras, "serverProperties": settings,
        "developmentMode": False, "qaMode": args.qa_mode, "rawLog": relative(run_dir / "stdout.log")})
    print("NEOFORGE_SERVER_INSTANCE=" + relative(instance), flush=True)
    events = queue.Queue()
    lines = []
    def reader():
        with (run_dir / "stdout.log").open("w", encoding="utf-8") as log:
            for line in process.stdout:
                log.write(line)
                log.flush()
                events.put(line.rstrip())
    thread = threading.Thread(target=reader, daemon=True)
    thread.start()
    ready = saved = stopping = forced = False
    started = time.monotonic()
    def send(text):
        if process.poll() is None:
            try:
                process.stdin.write(text + "\n")
                process.stdin.flush()
            except (BrokenPipeError, OSError):
                report["stdinClosedDuringShutdown"] = True
    try:
        while process.poll() is None:
            try:
                line = events.get(timeout=.2)
                lines.append(line)
                if re.search(r"Done \(.+\)! For help", line):
                    ready = True
                    if not args.qa_mode:
                        send("datapack list enabled")
                        send("save-all flush")
                if "Saved the game" in line:
                    saved = True
                    if not stopping:
                        send("stop")
                        stopping = True
            except queue.Empty:
                pass
            if time.monotonic() - started > args.timeout:
                report["timedOut"] = True
                break
    finally:
        # This handle belongs only to this invocation. Also clean up on an
        # unexpected harness exception or interruption after the launch.
        if process.poll() is None:
            send("stop")
            try:
                process.wait(timeout=15)
            except subprocess.TimeoutExpired:
                process.kill()
                process.wait(timeout=10)
                forced = True
        report["forcedStop"] = forced
        thread.join(timeout=5)
        report["logReaderCompleted"] = not thread.is_alive()
        try:
            process.stdin.close()
        except OSError:
            report["stdinClosedDuringShutdown"] = True
    while not events.empty():
        lines.append(events.get_nowait())
    log = "\n".join(lines)
    qa = json.loads(result_file.read_text()) if result_file.exists() else None
    class_log = (run_dir / "class-load.log").read_text(errors="replace")
    origin = any("raccoonman.reterraforged.RTFCommon source:" in line and installed.name in line
        for line in class_log.splitlines())
    stop_index = max((index for index, line in enumerate(lines) if "Stopping server" in line), default=-1)
    saved_after_stop = stop_index >= 0 and any("All chunks are saved" in line for line in lines[stop_index + 1:])
    clean_stop = process.returncode == 0 and saved_after_stop and report["logReaderCompleted"]
    target_game = bool(re.search(r"Starting minecraft server version 26\.2\b", log, re.I))
    unchanged = digest(artifact) == digest(installed)
    startup_reached = ready or bool(qa and qa.get("serverStarted"))
    passed = startup_reached and clean_stop and target_game and origin and unchanged and not forced and not report.get("timedOut")
    if args.qa_mode:
        passed = passed and bool(qa and qa.get("status") == "passed" and qa.get("runtime", {}).get("developmentEnvironment") is False)
    else:
        passed = passed and saved
    report.update({"status": "passed" if passed else "failed", "processExit": process.returncode,
        "ready": ready, "startupReached": startup_reached,
        "saveObserved": saved or bool(qa and qa.get("saveCompleted")), "cleanStop": clean_stop,
        "artifactOriginObserved": origin, "targetGameObserved": target_game, "artifactUnchanged": unchanged,
        "forcedStop": forced, "runtimeQa": qa, "customTerrainValidated": bool(qa and any(
            node.get("name") == "custom-generator-active" and node.get("status") == "passed" for node in qa.get("nodes", []))),
        "logTail": lines[-50:], "notTested": ["Visual parity", "Third-party modpacks", "Multiplayer clients", "All presets"]})


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--java-home", required=True, type=Path)
    parser.add_argument("--jar", type=Path)
    parser.add_argument("--prepare-only", action="store_true")
    parser.add_argument("--extra-mod", action="append", type=Path, default=[])
    parser.add_argument("--qa-mode", choices=["export", "generate", "reload"])
    parser.add_argument("--datapack", type=Path)
    parser.add_argument("--reuse-instance", type=Path)
    parser.add_argument("--timeout", type=int, default=150)
    parser.add_argument("--report", required=True, type=Path)
    args = parser.parse_args()
    java = args.java_home.resolve() / "bin/java.exe"
    report = {"schemaVersion": 1, "minecraft": "26.2", "loader": "neoforge", "neoforge": NEO}
    try:
        prepared = prepare(java)
        report["preparation"] = prepared
        if args.prepare_only:
            report["status"] = "prepared"
        elif args.jar:
            run(args, java, prepared, report)
        else:
            raise ValueError("--jar is required unless --prepare-only")
    except Exception as error:
        report.update({"status": "failed", "error": repr(error)})
    args.report.parent.mkdir(parents=True, exist_ok=True)
    args.report.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    print("NEOFORGE_SERVER_RESULT=" + report["status"], flush=True)
    return 0 if report["status"] in ("passed", "prepared") else 1


if __name__ == "__main__":
    raise SystemExit(main())
