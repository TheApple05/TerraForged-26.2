"""Build explicit host-density oracle configuration, without expected outputs."""
import json
from pathlib import Path
ROOT = Path(__file__).resolve().parents[2]
folder = ROOT / "horizon/tests/fixtures/kernel-v1"
registry = json.loads((folder / "noise-registry.json").read_text(encoding="utf-8"))
names = ["reterraforged:terrain/ground", "reterraforged:climate/biome_edge_shape"]
data = {"preset": json.loads((folder / "preset.json").read_text(encoding="utf-8")),
        "noiseRegistry": {n: registry[n] for n in names},
        "seeds": [0, -1, -2147483648, 2147483647, 8675309, 4303642605],
        "coordinates": [-129, -128, -1, 0, 127, 128], "heights": [-64, 80, 320],
        "resolutions": [1, 16, 256],
        "volumes": [{"minimum": [-130, -64, -130], "sizes": [5, 3, 5], "steps": [1, 192, 1]},
                    {"minimum": [126, -64, 126], "sizes": [5, 3, 5], "steps": [1, 192, 1]}]}
target = ROOT / "horizon/tests/inputs/host-density-v1.json"
encoded = (json.dumps(data, indent=2) + "\n").encode()
if target.exists() and target.read_bytes() != encoded:
    raise ValueError("Refusing to replace differing predefined inputs")
target.write_bytes(encoded)
