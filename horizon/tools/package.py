"""Package only native adapter sources; never include game resources or snapshots."""
from __future__ import annotations
import argparse
import hashlib
import json
from pathlib import Path
import zipfile

ROOT = Path(__file__).resolve().parents[2]


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path)
    parser.add_argument("--biomes", type=Path, help="Include all 17 exported classic biome definitions as a combined native/content .hmod")
    args = parser.parse_args()
    native = json.loads((ROOT / "horizon/horizon-native.json").read_text())
    if args.output is None:
        suffix = ".hmod" if args.biomes else ".hnmod"
        args.output = ROOT / ("build/distributions/terraforged-prepared-0.1.0-horizon26.3" + suffix)
    bundle = ROOT / "horizon/mixins" / native["id"]
    entries = {"horizon-native.json": (ROOT / "horizon/horizon-native.json").read_bytes()}
    for path in sorted(bundle.iterdir()):
        if path.is_file():
            entries[f"mixins/{native['id']}/{path.name}"] = path.read_bytes()
    entries[f"mixins/{native['id']}/LICENSE"] = (ROOT / "horizon/LICENSE").read_bytes()
    entries[f"mixins/{native['id']}/LICENSE-RETERRAFORGED"] = (ROOT / "licenses/ReTerraForged-MIT.txt").read_bytes()
    entries[f"mixins/{native['id']}/LICENSE-TERRAFORGED"] = (ROOT / "licenses/TerraForged-MIT.txt").read_bytes()
    entries[f"mixins/{native['id']}/README.md"] = (ROOT / "horizon/README.md").read_bytes()
    included_biomes = []
    if args.biomes:
        required = set("bryce cold_steppe cold_marshland fir_forest flower_plains frozen_lake frozen_marsh lake marshland savanna_scrub shattered_savanna_scrub snowy_fir_forest snowy_taiga_scrub steppe stone_forest taiga_scrub warm_beach".split())
        actual = {path.stem for path in args.biomes.glob("*.json")}
        if actual != required:
            parser.error("Combined content requires exactly the 17 restored classic biome JSON entries")
        entries["horizon-mod.toml"] = b'''id = "terraforged_prepared"
name = "TerraForged prepared terrain prototype"
version = "0.1.0"
description = "Finite prepared TerraForged terrain; native infinite generation is unfinished."
authors = ["TerraForged Revival contributors"]
api = "0.2"
side = "both"
capabilities = ["registries"]
namespaces = ["reterraforged"]
'''
        for name in sorted(required):
            path = args.biomes / (name + ".json")
            json.loads(path.read_text(encoding="utf-8"))
            entries[f"data/reterraforged/worldgen/biome/{name}.json"] = path.read_bytes()
            included_biomes.append("reterraforged:" + name)
        entries["assets/reterraforged/lang/en_us.json"] = (json.dumps({
            "biome.reterraforged." + name: name.replace("_", " ").title()
            for name in sorted(required)
        }, indent=2) + "\n").encode("utf-8")
    args.output.parent.mkdir(parents=True, exist_ok=True)
    with zipfile.ZipFile(args.output, "w", compression=zipfile.ZIP_DEFLATED) as archive:
        for name, data in sorted(entries.items()):
            info = zipfile.ZipInfo(name, (1980, 1, 1, 0, 0, 0))
            info.compress_type = zipfile.ZIP_DEFLATED
            info.external_attr = 0o100644 << 16
            archive.writestr(info, data)
    with zipfile.ZipFile(args.output) as archive:
        assert archive.namelist() == sorted(entries)
        assert all(archive.read(name) == data for name, data in entries.items())
    report = {"schema": 1, "status": "packaged-native-prototype", "artifact": str(args.output.resolve()),
              "sha256": hashlib.sha256(args.output.read_bytes()).hexdigest(), "manifest": native,
              "entrySha256": {name: hashlib.sha256(data).hexdigest() for name, data in sorted(entries.items())},
              "includedPreparedTerrainSnapshots": False,
              "includedMinecraftBinariesOrPlayerPreparedResources": False,
              "includedClassicBiomeIds": included_biomes,
              "notEvidenceOf": ["Normal launcher installation", "Rendered native game", "Infinite native generation", "Horizon modpack readiness"]}
    (args.output.parent / (args.output.name + ".json")).write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(report, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
