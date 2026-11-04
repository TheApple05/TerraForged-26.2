"""Prepare a publisher-hashed Linux JRE in ignored project storage, without global installation."""
from __future__ import annotations
import argparse
import datetime as dt
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import posixpath
import subprocess
import tarfile
import urllib.request

ROOT = Path(__file__).resolve().parents[2]
NAME = "OpenJDK25U-jre_x64_linux_hotspot_25.0.5_8.tar.gz"
URL = "https://github.com/adoptium/temurin25-binaries/releases/download/jdk-25.0.5%2B8/" + NAME
SHA256 = "6fac98f0c8202efc3135a9e9e42aa61963ec42d1c5cbb40caf231e7b14a24ef6"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--wsl-distro", default="Ubuntu")
    parser.add_argument("--report", type=Path, default=ROOT / "docs/validation/linux-jre25-runtime.json")
    args = parser.parse_args()
    tools = (ROOT / ".tools").resolve()
    archive = tools / "downloads" / NAME
    target = (tools / "java-linux25").resolve()
    if not target.is_relative_to(tools) or target == tools:
        raise ValueError("Linux runtime target escaped project tool storage")
    archive.parent.mkdir(parents=True, exist_ok=True)
    if not archive.exists():
        request = urllib.request.Request(URL, headers={"User-Agent": "TerraForged-26.2-validation/1.0"})
        with urllib.request.urlopen(request, timeout=60) as response:
            archive.write_bytes(response.read())
    if hashlib.sha256(archive.read_bytes()).hexdigest() != SHA256:
        raise ValueError("Linux JRE publisher SHA-256 mismatch")
    with tarfile.open(archive) as tar:
        roots = set()
        for member in tar.getmembers():
            path = PurePosixPath(member.name)
            if path.is_absolute() or ".." in path.parts or not path.parts:
                raise ValueError("Unsafe archive member: " + member.name)
            roots.add(path.parts[0])
            if member.issym() or member.islnk():
                linked = posixpath.normpath(posixpath.join(posixpath.dirname(member.name), member.linkname) if member.issym() else member.linkname)
                if linked.startswith("/") or not linked.startswith(path.parts[0] + "/"):
                    raise ValueError("Archive link leaves runtime root: " + member.name)
        if len(roots) != 1:
            raise ValueError("Runtime archive must have one top-level directory")
    def linux_path(path):
        if os.name != "nt": return str(path)
        return subprocess.check_output(["wsl", "-d", args.wsl_distro, "--exec", "wslpath", "-a", path.as_posix()], text=True).strip()
    def linux_call(command):
        prefix = ["wsl", "-d", args.wsl_distro, "--exec"] if os.name == "nt" else []
        return subprocess.run(prefix + command, capture_output=True, text=True, timeout=90, check=True)
    marker = target / ".terraforged-linux-jre.json"
    if not marker.exists():
        if target.exists() and any(target.iterdir()):
            raise ValueError("Refusing to extract over an unmarked nonempty runtime directory")
        target.mkdir(parents=True, exist_ok=True)
        linux_call(["tar", "--extract", "--gzip", "--file", linux_path(archive), "--directory", linux_path(target),
            "--strip-components=1", "--no-same-owner"])
        marker.write_text(json.dumps({"archiveSha256": SHA256, "createdBy": "prepare_linux_runtime.py"}) + "\n")
    if json.loads(marker.read_text())["archiveSha256"] != SHA256:
        raise ValueError("Existing project Linux runtime marker differs")
    version = linux_call([linux_path(target / "bin/java"), "-version"])
    system = linux_call(["uname", "-srmo"])
    report = {"schemaVersion": 1, "status": "prepared-runtime-executable", "checkedAtUtc": dt.datetime.now(dt.timezone.utc).isoformat(),
        "publisherReleaseUrl": "https://github.com/adoptium/temurin25-binaries/releases/tag/jdk-25.0.5%2B8",
        "sourceUrl": URL, "sha256": SHA256, "matchesPublisherGitHubReleaseDigest": True, "bytes": archive.stat().st_size,
        "runtimeDirectory": target.relative_to(ROOT).as_posix(), "linuxRuntimeDirectory": linux_path(target),
        "wslDistribution": args.wsl_distro if os.name == "nt" else None, "system": system.stdout.strip(),
        "javaVersionOutput": (version.stdout + version.stderr).strip(), "globalEnvironmentChanged": False,
        "notEvidenceOf": ["Minecraft loading", "Mod compatibility", "Linux performance"]}
    args.report.parent.mkdir(parents=True, exist_ok=True); args.report.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(report, indent=2))


if __name__ == "__main__": main()
