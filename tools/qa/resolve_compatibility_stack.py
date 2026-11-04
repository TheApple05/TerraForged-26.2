"""Fetch pinned publisher files supporting 26.2 and verify every published file hash.

This deliberately records upstream ModernFix's lack of a 26.2 build separately
from the explicitly named ModernFix-mVUS community port. No third-party binary
is committed or included in TerraForged's distribution.
"""
from __future__ import annotations

import argparse
import concurrent.futures
import datetime as dt
import hashlib
import json
from pathlib import Path
import urllib.request

ROOT = Path(__file__).resolve().parents[2]
PINS = {
    "sodium": {"fabric": "xJZxADzI", "neoforge": "DmnNKsfS"},
    "iris": {"fabric": "gxZWWnKH", "neoforge": "k55HdONq"},
    "distanthorizons": {"fabric": "aAow4a9f", "neoforge": "aAow4a9f"},
    "c2me-fabric": {"fabric": "4wjXxrDQ"},
    "c2me-neoforge": {"neoforge": "XpuI7aLS"},
    "lithium": {"fabric": "f7vZ0VWU", "neoforge": "J9CowDXK"},
    "ferrite-core": {"fabric": "d5ddUdiB", "neoforge": "LtVvw4uS"},
    "repurposed-structures-fabric": {"fabric": "CrmgMIJp"},
    "repurposed-structures-forge": {"neoforge": "h9w0EGMo"},
    "midnightlib": {"fabric": "FYtROxFj"},
    "modernfix-mvus": {"fabric": "50uUra83", "neoforge": "dyZhmjnd"},
    "complementary-reimagined": {"iris": "Bqen1mJX"},
}


def get(url):
    request = urllib.request.Request(url, headers={"User-Agent": "TerraForged-26.2-compatibility-validation/1.0"})
    with urllib.request.urlopen(request, timeout=60) as response:
        return response.read()


def resolve(item, download: bool):
    slug, loader, version_id = item
    project = json.loads(get("https://api.modrinth.com/v2/project/" + slug))
    version = json.loads(get("https://api.modrinth.com/v2/version/" + version_id))
    if version["project_id"] != project["id"] or loader not in version["loaders"] or "26.2" not in version["game_versions"]:
        raise ValueError("Pinned publisher metadata does not match requested target: " + str(item))
    files = [file for file in version["files"] if file["primary"]]
    if len(files) != 1:
        raise ValueError("Expected exactly one primary publisher file: " + version_id)
    file = files[0]
    target = ROOT / "build/qa/modpack-compat-expanded" / loader / file["filename"]
    actual = None
    if download:
        target.parent.mkdir(parents=True, exist_ok=True)
        if not target.is_file():
            target.write_bytes(get(file["url"]))
        content = target.read_bytes()
        for algorithm, expected in file["hashes"].items():
            if hashlib.new(algorithm, content).hexdigest() != expected:
                raise ValueError("Downloaded publisher hash mismatch: " + str(target))
        actual = hashlib.sha256(content).hexdigest()
    return {"slug": slug, "loader": loader, "projectId": project["id"], "title": project["title"],
        "publisherSourceUrl": project["source_url"], "license": project["license"],
        "versionId": version_id, "versionNumber": version["version_number"], "versionType": version["version_type"],
        "datePublished": version["date_published"], "gameVersions": version["game_versions"],
        "dependencies": version["dependencies"], "filename": file["filename"], "sourceUrl": file["url"],
        "versionUrl": "https://modrinth.com/mod/" + slug + "/version/" + version_id,
        "publishedHashes": file["hashes"], "sha256": actual, "publishedHashesMatch": True if download else None,
        "file": target.relative_to(ROOT).as_posix(), "bytes": target.stat().st_size if download else None,
        "communityPort": slug == "modernfix-mvus", "category": "shaderpack" if loader == "iris" else "mod"}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--download", action="store_true")
    parser.add_argument("--report", type=Path, default=ROOT / "docs/validation/compatibility-26.2-artifacts.json")
    args = parser.parse_args()
    jobs = [(slug, loader, version) for slug, loaders in PINS.items() for loader, version in loaders.items()]
    with concurrent.futures.ThreadPoolExecutor(max_workers=4) as pool:
        artifacts = list(pool.map(lambda item: resolve(item, args.download), jobs))
    modernfix = json.loads(get("https://api.modrinth.com/v2/project/modernfix/version?game_versions=%5B%2226.2%22%5D"))
    report = {"schemaVersion": 1, "checkedAtUtc": dt.datetime.now(dt.timezone.utc).isoformat(),
        "minecraft": "26.2", "selection": "Explicit immutable publisher version IDs, exact declared target and loader checked",
        "artifacts": artifacts, "officialModernFix26.2": {"available": bool(modernfix),
            "query": "https://api.modrinth.com/v2/project/modernfix/version?game_versions=%5B%2226.2%22%5D",
            "matchingVersionIds": [version["id"] for version in modernfix]},
        "notEvidenceOf": ["Runtime compatibility", "Performance improvement", "Every modpack combination"]}
    args.report.parent.mkdir(parents=True, exist_ok=True)
    args.report.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({"artifacts": len(artifacts), "downloaded": args.download, "report": str(args.report)}))


if __name__ == "__main__":
    main()
