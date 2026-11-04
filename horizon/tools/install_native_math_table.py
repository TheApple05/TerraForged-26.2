"""Install/check source-owned lookup data from the immutable Java math reference."""
from __future__ import annotations
import argparse
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--write',action='store_true')
    args = parser.parse_args()
    reference = ROOT/'horizon/tests/fixtures/math-v1'
    metadata = json.loads((reference/'metadata.json').read_text())
    data = (reference/'sine.bin').read_bytes()
    original = json.loads((ROOT/'tests/core/reference-metadata.json').read_text())
    pinned = next(value for key,value in original['source_sha256_lf'].items() if key.endswith('/noise/NoiseUtil.java'))
    if metadata['noiseUtilSourceSha256Lf']!=pinned or hashlib.sha256(data).hexdigest()!=metadata['sineTableSha256']:
        raise ValueError('Lookup source/reference identity changed')
    properties = metadata['properties']
    count,mask = int(properties['SIN_COUNT']),int(properties['SIN_MASK'])
    if count!=4096 or mask!=count-1 or len(data)!=count*4:
        raise ValueError('Unsupported sine table layout')
    target = ROOT/'horizon/native-core/src/data/noiseutil-sine.bin'
    if args.write:
        target.parent.mkdir(parents=True,exist_ok=True)
        target.write_bytes(data)
    elif target.read_bytes()!=data:
        raise ValueError('Native lookup differs from its exact Java reference')
    print('NATIVE_MATH_TABLE_MATCH=4096')


if __name__=='__main__': main()
