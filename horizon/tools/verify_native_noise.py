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
    parser.add_argument('--family', choices=['base','second','composition','math','core','sources','power','shaping','cellular','final','visitors'], default='base')
    args = parser.parse_args()
    if args.report.exists():
        parser.error('Refusing to replace native parity evidence')
    fixture = ROOT/{'base':'horizon/tests/fixtures/noise-reference-v1.tsv','second':'horizon/tests/fixtures/noise-reference-v2.tsv',
                    'composition':'horizon/tests/fixtures/noise-composition-v1.tsv','math':'horizon/tests/fixtures/math-v1/values.tsv',
                    'core':'tests/core/reference-output.tsv','sources':'horizon/tests/fixtures/noise-sources-v1.tsv',
                    'power':'horizon/tests/fixtures/power-v1.tsv','shaping':'horizon/tests/fixtures/noise-shaping-v1.tsv',
                    'cellular':'horizon/tests/fixtures/noise-cellular-v1.tsv','final':'horizon/tests/fixtures/noise-final-v1.tsv','visitors':'horizon/tests/fixtures/noise-visitors-v1.tsv'}[args.family]
    metadata_file = (ROOT/'tests/core/reference-metadata.json' if args.family=='core' else
                     fixture.parent/'metadata.json' if args.family=='math' else fixture.with_suffix('.json'))
    metadata = json.loads(metadata_file.read_text())
    expected_sha = metadata['fixture_sha256'] if args.family=='core' else metadata['valuesSha256'] if args.family=='math' else metadata['fixtureSha256']
    if digest(fixture) != expected_sha or (args.family!='core' and not metadata['freshRunsAgree']):
        raise ValueError('Independent Java oracle changed or lacks fresh-run agreement')
    expected = rows(fixture.read_text())
    if len(expected) != metadata.get('recordCount',metadata.get('record_count')):
        raise ValueError('Oracle record count changed')
    manifest = ROOT/'horizon/native-core/Cargo.toml'
    logroot = ROOT/'build/qa/horizon/native-noise'/args.report.stem
    logroot.mkdir(parents=True, exist_ok=True)
    report = {'schemaVersion':1,'status':'failed','startedAtUtc':dt.datetime.now(dt.timezone.utc).isoformat(),
        'fixtureSha256':digest(fixture),'javaOracle':metadata,'profiles':[],
        'recordsPerProfile':len(expected),'valuesPerProfile':sum(map(len,expected.values())),
        'rustc':subprocess.check_output(['rustc','+stable','-Vv'],text=True),
        'family':args.family,
        'scope':('144 visited Java graphs with exact callback traces' if args.family=='visitors' else
                 '78 line/terrace/domain/erosion graphs, domain coordinates, line helpers and explicit NaN classifications' if args.family=='final' else
                 '158 genuine cellular/legacy/spline/cache Java graphs and thread cache sequences' if args.family=='cellular' else
                 '85 genuine Java shaping constructor graphs and legacy min/max' if args.family=='shaping' else
                 'White/Cubic/ridge/billow static/fractal and metadata' if args.family=='sources' else
                 'Float power and spectral weight observations' if args.family=='power' else
                 'Complete original frozen helper/core corpus' if args.family=='core' else
                 'Source-owned lookup data and Java trig/sqrt observations' if args.family=='math' else
                 '40 genuine Java constructor graphs and legacy min/max' if args.family=='composition' else
                 '296 vectors and '+('Perlin/Simplex' if args.family=='base' else 'Perlin2/Simplex2')+' sample/fractal values')+'; raw float bits, no native terrain activation.',
        'notCovered':['Other noise modules or arbitrary graphs','Complete nonfinite/parameter domain','Cell/river/tile pipeline',
                      'Native density/surface/caves/decoration','Infinite generation','Native game save/rendering']}
    for profile in ('debug','release'):
        command = ['cargo','+stable','run','--quiet','--offline','--locked','--manifest-path',str(manifest),'--bin',
                   {'base':'noise_probe','second':'noise_family_probe','composition':'noise_composition_probe',
                    'math':'math_probe','core':'core_probe','sources':'noise_sources_probe','power':'power_probe',
                    'shaping':'noise_shaping_probe','cellular':'noise_cellular_probe','final':'noise_final_probe','visitors':'noise_visitor_probe'}[args.family]]
        if profile == 'release':
            command.append('--release')
        run = subprocess.run(command,cwd=ROOT,capture_output=True,text=True,timeout=90)
        output = logroot/(profile+'.tsv')
        output.write_text(run.stdout,encoding='utf-8',newline='\n')
        (logroot/(profile+'.stderr.log')).write_text(run.stderr,encoding='utf-8')
        actual = rows(run.stdout) if run.returncode == 0 else {}
        differences = [{'key':k,'expected':expected.get(k),'actual':actual.get(k)}
            for k in sorted(set(actual)|set(expected)) if actual.get(k)!=expected.get(k)]
        guards = args.family!='composition' or 'NATIVE_GRAPH_GUARDS_OK depth=256 expandedBudget=1000000' in run.stderr
        guards = guards and (args.family!='shaping' or 'NATIVE_SHAPING_ITERATION_GUARD_OK max=1024' in run.stderr)
        guards = guards and (args.family!='cellular' or 'NATIVE_CELLULAR_GUARDS_OK orderedNonemptySpline legacyPowerLimit=1024' in run.stderr)
        guards = guards and (args.family!='final' or 'NATIVE_FINAL_GUARDS_OK terrace=2..4096 octaves=0..32 combinedDomainDepth=256' in run.stderr)
        report['profiles'].append({'profile':profile,'exitCode':run.returncode,'outputSha256':digest(output),
            'differenceCount':len(differences),'firstDifferences':differences[:50],
            'constructorGuardsObserved':guards,
            'allJavaValuesMatch':run.returncode==0 and not differences and guards})
    report['status'] = 'passed-noise-foundation-only' if all(p['allJavaValuesMatch'] for p in report['profiles']) else 'failed'
    sources = sorted((ROOT/'horizon/native-core/src').rglob('*.rs'))+sorted((ROOT/'horizon/native-core/src/data').glob('*.bin'))+[manifest,ROOT/'horizon/native-core/Cargo.lock']
    report['sourceSha256Lf'] = {p.relative_to(ROOT).as_posix():hashlib.sha256(
        p.read_bytes() if p.suffix=='.bin' else p.read_bytes().replace(b'\r\n',b'\n')).hexdigest() for p in sources}
    report['sourceNormalization'] = 'LF for text source/configuration; binary lookup bytes exact'
    report['finishedAtUtc'] = dt.datetime.now(dt.timezone.utc).isoformat()
    report['verifierSha256'] = digest(Path(__file__))
    args.report.parent.mkdir(parents=True,exist_ok=True)
    args.report.write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8')
    print(json.dumps({k:report[k] for k in ('status','recordsPerProfile','valuesPerProfile')}))
    return 0 if report['status']=='passed-noise-foundation-only' else 1


if __name__ == '__main__':
    raise SystemExit(main())
