"""Record the Java source surface for the next infinite native terrain milestone.

This is a conservative package inventory, not a claim that every method is
reachable or that a Java-to-Rust port exists. Platform imports are listed as
adapter boundaries rather than silently included in a pure arithmetic kernel.
"""
from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import re

ROOT = Path(__file__).resolve().parents[2]
JAVA = ROOT / "common/src/main/java"
BASE = JAVA / "raccoonman/reterraforged"
GROUPS = {
    "noise": "world/worldgen/noise",
    "cell": "world/worldgen/cell",
    "tile": "world/worldgen/densityfunction/tile",
    "presetSettings": "data/worldgen/preset/settings",
    "utilities": "world/worldgen/util",
}
ADDITIONAL = ["world/worldgen/GeneratorContext.java", "world/worldgen/WorldFilters.java",
              "world/worldgen/WorldErosion.java"]


def inventory() -> dict:
    groups = {}
    files = set()
    for label, folder in GROUPS.items():
        selected = sorted((BASE / folder).rglob("*.java"))
        files.update(selected)
        groups[label] = {
            "directory": (BASE / folder).relative_to(ROOT).as_posix(),
            "files": len(selected),
            "nonemptyLines": sum(sum(bool(line.strip()) for line in p.read_text(encoding="utf-8").splitlines()) for p in selected),
        }
    files.update(BASE / path for path in ADDITIONAL)
    entries, imports = {}, {}
    for path in sorted(files):
        raw = path.read_bytes()
        source = raw.decode("utf-8")
        key = path.relative_to(ROOT).as_posix()
        entries[key] = {"sha256": hashlib.sha256(raw).hexdigest(),
                        "nonemptyLines": sum(bool(line.strip()) for line in source.splitlines())}
        for target in re.findall(r"^import\s+(?:static\s+)?([\w.*]+);", source, flags=re.MULTILINE):
            if target.startswith("java."):
                category = "javaLibrarySemantics"
            elif target.startswith("raccoonman.reterraforged."):
                # Nested classes and static members need a containing source.
                parts = target.split(".")
                local = None
                while parts:
                    candidate = JAVA.joinpath(*parts).with_suffix(".java")
                    if candidate.is_file():
                        local = candidate
                        break
                    parts.pop()
                if local in files:
                    continue
                category = "additionalProjectAdapters"
            else:
                category = "externalAdapters"
            imports.setdefault(category, {}).setdefault(target, []).append(key)
    return {
        "schema": 1, "status": "inventory-not-implemented-native-kernel",
        "scope": "Complete cell/tile pipeline for one pinned preset before native chunk activation",
        "groups": groups, "additionalSources": ADDITIONAL,
        "totalFiles": len(entries), "totalNonemptyLines": sum(e["nonemptyLines"] for e in entries.values()),
        "entries": entries, "importBoundaries": imports,
        "limitations": ["Conservative package inventory, not method reachability analysis",
                        "Preset/codec/bootstrap and Java concurrency APIs require explicit Rust adapters",
                        "Generated chunks also require native density, surface, biome and feature integration"],
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", required=True, type=Path)
    args = parser.parse_args()
    report = inventory()
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({"output": str(args.output), "totalFiles": report["totalFiles"],
                      "totalNonemptyLines": report["totalNonemptyLines"], "groups": report["groups"]}, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
