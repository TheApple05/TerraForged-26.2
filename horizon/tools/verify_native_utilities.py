"""Check native coordinate/seed/RNG output against the frozen original Java fixture."""
from __future__ import annotations
import argparse
import datetime as dt
import hashlib
import json
from pathlib import Path
import subprocess
from verify_native_noise import rows

ROOT = Path(__file__).resolve().parents[2]


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--report',type=Path,required=True)
    args = parser.parse_args()
    if args.report.exists():
        parser.error('Refusing to replace utility evidence')
    fixture = ROOT/'tests/core/reference-output.tsv'
    metadata = json.loads((ROOT/'tests/core/reference-metadata.json').read_text())
    if digest(fixture)!=metadata['fixture_sha256']:
        raise ValueError('Original independent Java fixture changed')
    expected = {k:v for k,v in rows(fixture.read_text()).items() if k.startswith(('pack/','packf/','seed/','random/'))}
    if len(expected)!=422:
        raise ValueError('Unexpected reference subset')
    report = {'schemaVersion':1,'status':'failed','startedAtUtc':dt.datetime.now(dt.timezone.utc).isoformat(),
        'fixtureSha256':digest(fixture),'upstreamCommit':metadata['upstream_commit'],
        'recordsPerProfile':len(expected),'profiles':[],
        'scope':'Frozen coordinate packing, signed-zero float packing, seed sequences and seeded FastRandom streams; no terrain activation.',
        'notCovered':['Every public utility API/input','Arbitrary 64-bit seeds/gammas','Dedicated RNG rejection-loop corpus','Cell/tile/rivers','Native infinite generation']}
    logs = ROOT/'build/qa/horizon/native-utilities'
    logs.mkdir(parents=True,exist_ok=True)
    for profile in ('debug','release'):
        command = ['cargo','+stable','run','--quiet','--offline','--locked','--manifest-path',str(ROOT/'horizon/native-core/Cargo.toml'),'--bin','utility_probe']
        if profile=='release': command.append('--release')
        run = subprocess.run(command,cwd=ROOT,capture_output=True,text=True,timeout=90)
        output = logs/(profile+'.tsv')
        output.write_text(run.stdout,encoding='utf-8',newline='\n')
        (logs/(profile+'.stderr.log')).write_text(run.stderr,encoding='utf-8')
        actual = rows(run.stdout) if run.returncode==0 else {}
        differences = [{'key':k,'expected':expected.get(k),'actual':actual.get(k)} for k in sorted(set(expected)|set(actual)) if expected.get(k)!=actual.get(k)]
        report['profiles'].append({'profile':profile,'exitCode':run.returncode,'outputSha256':digest(output),
            'differenceCount':len(differences),'firstDifferences':differences[:50],
            'allJavaValuesMatch':run.returncode==0 and not differences})
    report['status'] = 'passed-utilities-only' if all(p['allJavaValuesMatch'] for p in report['profiles']) else 'failed'
    paths = [ROOT/'horizon/native-core/src'/name for name in ('seed.rs','pos_util.rs','fast_random.rs','bin/utility_probe.rs')]
    report['sourceSha256Lf'] = {p.relative_to(ROOT).as_posix():hashlib.sha256(p.read_bytes().replace(b'\r\n',b'\n')).hexdigest() for p in paths}
    report['finishedAtUtc']=dt.datetime.now(dt.timezone.utc).isoformat()
    report['verifierSha256']=digest(Path(__file__))
    args.report.parent.mkdir(parents=True,exist_ok=True)
    args.report.write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8')
    print(json.dumps({'status':report['status'],'differences':[p['differenceCount'] for p in report['profiles']]}))
    return 0 if report['status']=='passed-utilities-only' else 1


if __name__=='__main__': main()
