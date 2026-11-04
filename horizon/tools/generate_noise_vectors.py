"""Translate the pinned MIT NoiseUtil vector literals without regenerating noise."""
from __future__ import annotations
import argparse
import hashlib
from pathlib import Path
import re

ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / 'common/src/main/java/raccoonman/reterraforged/world/worldgen/noise/NoiseUtil.java'
OUTPUT = ROOT / 'horizon/native-core/src/noise_vectors.rs'


def generated():
    source = SOURCE.read_bytes().replace(b'\r\n', b'\n')
    # This is also the independently frozen core-reference source.
    import json
    metadata = json.loads((ROOT / 'tests/core/reference-metadata.json').read_text())
    hashes = metadata['source_sha256_lf']
    # Reference metadata uses upstream-relative paths; validate against its map.
    matching = [value for name, value in hashes.items() if name.endswith('/noise/NoiseUtil.java')]
    if not matching:
        raise ValueError('Pinned NoiseUtil hash missing from reference metadata')
    if matching != [hashlib.sha256(source).hexdigest()]:
        raise ValueError('NoiseUtil differs from independently frozen reference')
    text = source.decode()
    lines = ['// Generated from MIT ReTerraForged NoiseUtil.java; see LICENSE-RETERRAFORGED.',
             '// Reproduce with python horizon/tools/generate_noise_vectors.py --write.',
             '// Literal order and duplicate gradient entries are intentional.', '']
    for name, count in [('GRAD_2D', 8), ('GRAD_2D_24', 32), ('CELL_2D', 256)]:
        declaration = re.search(r'\b' + name + r' = new Vec2f\[\] \{ (.*?) \};', text)
        if not declaration:
            raise ValueError('Vector declaration missing: ' + name)
        pairs = re.findall(r'new Vec2f\(([^,]+), ([^)]+)\)', declaration[1])
        if len(pairs) != count:
            raise ValueError('Wrong vector count: ' + name)
        lines.append(f'pub const {name}: [(f32, f32); {count}] = [')
        for x, y in pairs:
            lines.append(f'    ({x.rstrip("fF")}f32, {y.rstrip("fF")}f32),')
        lines += ['];', '']
    return '\n'.join(lines)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--write', action='store_true')
    args = parser.parse_args()
    expected = generated()
    if args.write:
        OUTPUT.write_text(expected, encoding='utf-8', newline='\n')
    elif OUTPUT.read_text() != expected:
        raise ValueError('Native vector source differs from pinned literals')
    print('NOISE_VECTORS_MATCH=296')


if __name__ == '__main__':
    main()
