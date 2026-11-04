"""Install/check the original source-owned palette transformed by the Java loader."""
from __future__ import annotations
import argparse
import hashlib
import json
from pathlib import Path

ROOT=Path(__file__).resolve().parents[2]
def sha(data):return hashlib.sha256(data).hexdigest()
def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--write',action='store_true');args=p.parse_args()
    folder=ROOT/'horizon/tests/fixtures/biome-v1';metadata=json.loads((folder/'metadata.json').read_text(encoding='utf-8'));data=(folder/'table.bin').read_bytes()
    if not metadata['freshRunsAgree'] or sha(data)!=metadata['tableSha256'] or len(data)!=65536 or max(data)>10:raise ValueError('Palette identity/layout changed')
    for name,expected in metadata['sourceSha256Lf'].items():
        if sha((ROOT/'common/src/main/java/raccoonman/reterraforged/world/worldgen'/name).read_bytes().replace(b'\r\n',b'\n'))!=expected:raise ValueError('Original palette algorithm changed: '+name)
    for name,expected in metadata['sourceOwnedResourcesSha256'].items():
        content=(ROOT/'common/src/main/resources'/name).read_bytes()
        if name.endswith('.txt'):content=content.replace(b'\r\n',b'\n')
        if sha(content)!=expected:raise ValueError('Original palette resource changed: '+name)
    target=ROOT/'horizon/native-core/src/data/biome-types.bin'
    if args.write:
        if target.exists() and target.read_bytes()!=data:raise ValueError('Refusing to replace different native palette bytes')
        target.parent.mkdir(parents=True,exist_ok=True);target.write_bytes(data)
    elif target.read_bytes()!=data:raise ValueError('Native palette differs from independent original reference')
    print('NATIVE_BIOME_TABLE_MATCH=65536')
if __name__=='__main__':main()
