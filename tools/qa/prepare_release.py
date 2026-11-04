"""Assemble both unchanged loader JARs only after complete candidate acceptance."""
from __future__ import annotations
import argparse
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
import shutil

ROOT = Path(__file__).resolve().parents[2]


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--acceptance', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--notes', type=Path, required=True)
    args = parser.parse_args()
    acceptance = json.loads(args.acceptance.read_text())
    if acceptance['status']!='passed-scoped-modpack-candidate':
        parser.error('Complete functional and performance acceptance is required')
    if set(acceptance['artifacts'])!={'fabric','neoforge'} or acceptance['minecraft']!='26.2':
        parser.error('Expected both Minecraft26.2 loader artifacts')
    if args.output.exists():
        parser.error('Refusing to replace a release directory')
    for loader, row in acceptance['artifacts'].items():
        if sha(ROOT/row['path'])!=row['sha256']:
            raise ValueError('Accepted artifact bytes changed: '+loader)
    for path, expected in acceptance['evidenceSha256'].items():
        if sha(ROOT/path)!=expected:
            raise ValueError('Accepted evidence changed: '+path)
    notes = args.notes.read_bytes()
    short_version = acceptance['version'].removesuffix('+26.2')
    args.output.mkdir(parents=True)
    artifacts = []
    for loader, row in acceptance['artifacts'].items():
        filename = f'terraforged-revival-{short_version}-{loader}-mc26.2.jar'
        target = args.output/filename
        shutil.copyfile(ROOT/row['path'],target)
        if sha(target)!=row['sha256']:
            raise ValueError('Release copy differs from tested bytes')
        artifacts.append({'loader':loader,'file':filename,'sha256':row['sha256'],'bytes':target.stat().st_size})
    (args.output/'VALIDATION.json').write_bytes(args.acceptance.read_bytes())
    (args.output/'RELEASE-NOTES.md').write_bytes(notes)
    (args.output/'INSTALLATION.md').write_bytes((ROOT/'docs/INSTALLATION.md').read_bytes())
    record = {'schemaVersion':1,'status':'assembled-accepted-prerelease','createdAtUtc':datetime.now(timezone.utc).isoformat(),
        'minecraft':'26.2','version':acceptance['version'],'stableRelease':False,'artifacts':artifacts,
        'requirements':{'java':'25','fabricLoader':'0.20.1','fabricApi':'0.161.0+26.2','neoforge':'26.2.0.93'},
        'acceptanceSha256':sha(args.acceptance),'excluded':['QA companions','Third-party mods','Game resources','Horizon prototype']}
    (args.output/'distribution.json').write_text(json.dumps(record,indent=2)+'\n',encoding='utf-8')
    files = sorted(path for path in args.output.iterdir() if path.is_file())
    (args.output/'SHA256SUMS').write_text(''.join(sha(path)+'  '+path.name+'\n' for path in files),encoding='utf-8')
    print(json.dumps(record,indent=2))


if __name__=='__main__':
    main()
