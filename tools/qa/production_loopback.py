"""Join a disposable, loopback-only dedicated server with the exact production client."""
from __future__ import annotations
import argparse
import datetime as dt
import json
import os
from pathlib import Path
import queue
import re
import signal as process_signal
import subprocess
import sys
import threading
import time
import uuid

ROOT = Path(__file__).resolve().parents[2]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--loader", choices=["fabric", "neoforge"], required=True)
    parser.add_argument("--java-home", type=Path, required=True)
    parser.add_argument("--jar", type=Path, required=True)
    parser.add_argument("--server-qa", type=Path, required=True)
    parser.add_argument("--client-qa", type=Path, required=True)
    parser.add_argument("--datapack", type=Path, required=True)
    parser.add_argument("--extra-mod", type=Path, action="append", default=[])
    parser.add_argument("--client-extra-mod", type=Path, action="append", default=[])
    parser.add_argument("--report", type=Path, required=True)
    args = parser.parse_args()
    if args.report.exists():
        raise ValueError("Refusing to replace prior evidence")
    base = ROOT / "build/qa/loopback" / (args.loader + "-" + uuid.uuid4().hex[:8])
    base.mkdir(parents=True)
    signal = base / "stop-requested"
    # The server harness retains its immutable report under docs/validation.
    server_report = args.report.resolve().with_name("server-" + args.report.name)
    if server_report.exists():
        raise ValueError("Refusing to replace prior server evidence")
    client_report = base / "client.json"
    server_tool = "production_server_smoke.py" if args.loader == "fabric" else "production_neoforge_server.py"
    command = [sys.executable, str(ROOT / "tools/qa" / server_tool), "--java-home", str(args.java_home),
               "--jar", str(args.jar), "--extra-mod", str(args.server_qa), "--datapack", str(args.datapack),
               "--qa-mode", "generate", "--timeout", "300", "--hold-until-file", str(signal),
               "--jvm-arg=-Dterraforged.qa.holdAfterGeneration=true", "--report", str(server_report)]
    for mod in args.extra_mod:
        command += ["--extra-mod", str(mod)]
    report = {"schemaVersion": 1, "loader": args.loader, "startedAtUtc": dt.datetime.now(dt.timezone.utc).isoformat(),
              "scope": "One offline QA identity, isolated loopback dedicated server; no public/authenticated-server claim.",
              "status": "failed", "serverCommand": command}
    process = subprocess.Popen(command, cwd=ROOT, stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
                               text=True, encoding="utf-8", errors="replace", start_new_session=os.name != "nt")
    events = queue.Queue()
    def read():
        with (base / "server-driver.log").open("w", encoding="utf-8") as log:
            for line in process.stdout:
                log.write(line)
                log.flush()
                events.put(line)
    reader = threading.Thread(target=read, daemon=True)
    reader.start()
    try:
        deadline = time.monotonic() + 150
        instance = None
        while time.monotonic() < deadline:
            if process.poll() is not None:
                raise RuntimeError("Dedicated server stopped before the client join")
            try:
                line = events.get(timeout=0.2)
                match = re.search(r"(?:NEOFORGE_)?SERVER_INSTANCE=(.+)", line)
                if match:
                    instance = ROOT / match.group(1).strip()
            except queue.Empty:
                pass
            if instance and (instance / "qa-output/result.json").is_file():
                qa = json.loads((instance / "qa-output/result.json").read_text())
                if qa.get("status") != "passed":
                    raise RuntimeError("Dedicated-server QA failed before the client join")
                report["serverGenerationBeforeJoin"] = qa
                break
        else:
            raise RuntimeError("Dedicated-server readiness exceeded the bound")
        properties = dict(line.split("=", 1) for line in (instance / "server.properties").read_text().splitlines() if "=" in line and not line.startswith("#"))
        if properties.get("server-ip") != "127.0.0.1" or properties.get("online-mode") != "false":
            raise ValueError("Disposable server is not restricted to the intended loopback QA mode")
        address = "127.0.0.1:" + properties["server-port"]
        print("LOOPBACK_READY=" + address, flush=True)
        client_tool = "production_client.py" if args.loader == "fabric" else "production_neoforge_client.py"
        client = [sys.executable, str(ROOT / "tools/qa" / client_tool), "--java-home", str(args.java_home),
                  "--jar", str(args.jar), "--extra-mod", str(args.client_qa), "--launch", "--client-qa",
                  "--wait-seconds", "130", "--render-distance", "4", "--report", str(client_report),
                  "--jvm-arg=-Dterraforged.clientqa.loopbackAddress=" + address]
        for mod in args.extra_mod + args.client_extra_mod:
            client += ["--extra-mod", str(mod)]
        report["clientCommand"] = client
        with (base / "client-driver.log").open("w", encoding="utf-8") as log:
            result = subprocess.run(client, cwd=ROOT, stdout=log, stderr=subprocess.STDOUT, timeout=160)
        data = json.loads(client_report.read_text())
        report["client"] = data
        qa = data.get("clientQa", {})
        network = qa.get("networkTerrainRendering", {})
        if result.returncode or qa.get("status") != "passed" or not network.get("connectionOpen") or len(network.get("synchronizedVariants", [])) != 17:
            raise RuntimeError("Actual loopback client registry/chunk/disconnect QA did not pass")
        report["clientJoinPassed"] = True
    except Exception as error:
        report["error"] = repr(error)
    finally:
        signal.touch()
        try:
            process.wait(timeout=60)
        except subprocess.TimeoutExpired:
            # The harness owns its Java child. Kill only this controller's process
            # tree if orderly shutdown fails, retaining an explicit failed result.
            if os.name == "nt":
                cleanup = subprocess.run(["taskkill", "/PID", str(process.pid), "/T", "/F"],
                                         capture_output=True, text=True, timeout=15)
                report["forcedCleanup"] = {"exitCode": cleanup.returncode, "output": cleanup.stdout + cleanup.stderr}
            else:
                os.killpg(process.pid, process_signal.SIGKILL)
            process.wait(timeout=15)
            report["controllerForcedStop"] = True
        reader.join(timeout=5)
        report["serverHarnessExit"] = process.returncode
        if server_report.exists():
            report["server"] = json.loads(server_report.read_text())
        server = report.get("server", {})
        passed = report.get("clientJoinPassed") and process.returncode == 0 and server.get("result", server.get("status")) == "passed" and not report.get("controllerForcedStop")
        report["status"] = "passed" if passed else "failed"
        report["finishedAtUtc"] = dt.datetime.now(dt.timezone.utc).isoformat()
        args.report.parent.mkdir(parents=True, exist_ok=True)
        args.report.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    print("LOOPBACK_RESULT=" + report["status"], flush=True)
    return 0 if report["status"] == "passed" else 1


if __name__ == "__main__":
    raise SystemExit(main())
