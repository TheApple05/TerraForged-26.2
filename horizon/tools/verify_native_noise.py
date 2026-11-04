"""Compare native noise modules with the immutable independently captured Java oracle."""
from __future__ import annotations
import argparse
import datetime as dt
import hashlib
import json
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[2]


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def rows(text):
    result = {}
    for line in text.splitlines():
        key, value = line.split('\t', 1)
        if key in result:
            raise ValueError('Duplicate record: '+key)
        result[key] = value.split(',')
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--report', type=Path, required=True)
    args = parser.parse_args()
    if args.report.exists():
        parser.error('Refusing to replace native parity evidence')
    fixture = ROOT/'horizon/tests/fixtures/noise-reference-v1.tsv'
    metadata = json.loads(fixture.with_suffix('.json').read_text())
    if digest(fixture) != metadata['fixtureSha256'] or not metadata['freshRunsAgree']:
        raise ValueError('Independent Java oracle changed or lacks fresh-run agreement')
    expected = rows(fixture.read_text())
    if len(expected) != metadata['recordCount']:
        raise ValueError('Oracle record count changed')
    manifest = ROOT/'horizon/native-core/Cargo.toml'
    logroot = ROOT/'build/qa/horizon/native-noise'
    logroot.mkdir(parents=True, exist_ok=True)
    report = {'schemaVersion':1,'status':'failed','startedAtUtc':dt.datetime.now(dt.timezone.utc).isoformat(),
        'fixtureSha256':digest(fixture),'javaOracle':metadata,'profiles':[],
        'recordsPerProfile':len(expected),'valuesPerProfile':sum(map(len,expected.values())),
        'rustc':subprocess.check_output(['rustc','+stable','-Vv'],text=True),
        'scope':'296 vectors, Perlin and Simplex sample/fractal values, raw float bits; no native terrain activation.',
        'notCovered':['Other noise modules or arbitrary graphs','Complete nonfinite/parameter domain','Cell/river/tile pipeline',
                      'Native density/surface/caves/decoration','Infinite generation','Native game save/rendering']}
    for profile in ('debug','release'):
        command = ['cargo','+stable','run','--quiet','--offline','--locked','--manifest-path',str(manifest),'--bin','noise_probe']
        if profile == 'release':
            command.append('--release')
        run = subprocess.run(command,cwd=ROOT,capture_output=True,text=True,timeout=90)
        output = logroot/(profile+'.tsv')
        output.write_text(run.stdout,encoding='utf-8',newline='\n')
        (logroot/(profile+'.stderr.log')).write_text(run.stderr,encoding='utf-8')
        actual = rows(run.stdout) if run.returncode == 0 else {}
        differences = [{'key':k,'expected':expected.get(k),'actual':actual.get(k)}
            for k in sorted(set(actual)|set(expected)) if actual.get(k)!=expected.get(k)]
        report['profiles'].append({'profile':profile,'exitCode':run.returncode,'outputSha256':digest(output),
            'differenceCount':len(differences),'firstDifferences':differences[:50],
            'allJavaValuesMatch':run.returncode==0 and not differences})
    report['status'] = 'passed-noise-foundation-only' if all(p['allJavaValuesMatch'] for p in report['profiles']) else 'failed'
    sources = sorted((ROOT/'horizon/native-core/src').rglob('*.rs'))+[manifest,ROOT/'horizon/native-core/Cargo.lock']
    report['sourceSha256Lf'] = {p.relative_to(ROOT).as_posix():hashlib.sha256(p.read_bytes().replace(b'\r\n',b'\n')).hexdigest() for p in sources}
    report['finishedAtUtc'] = dt.datetime.now(dt.timezone.utc).isoformat()
    report['verifierSha256'] = digest(Path(__file__))
    args.report.parent.mkdir(parents=True,exist_ok=True)
    args.report.write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8')
    print(json.dumps({k:report[k] for k in ('status','recordsPerProfile','valuesPerProfile')}))
    return 0 if report['status']=='passed-noise-foundation-only' else 1


if __name__ == '__main__':
    raise SystemExit(main())
