"""Compare the bounded native adapter against immutable genuine Java registry observations."""
from __future__ import annotations
import argparse
import datetime as dt
import hashlib
import json
from pathlib import Path
import subprocess
from verify_native_noise import rows

ROOT=Path(__file__).resolve().parents[2]
FAMILIES={
    'registry':('registry-v1','registry_probe','Bounded registry adapter, raw finite bits and explicit nonfinite classifications',
                'NATIVE_REGISTRY_GUARDS_OK missing cycle type field enum identifier numeric octave power depth lookup','passed-registry-adapter-only'),
    'preset':('preset-v1','preset_probe','Five kernel setting groups and 49 source factory graphs',
              'NATIVE_PRESET_GUARDS_OK missingSections unknownEnums sourceOctaves','passed-preset-adapter-only'),
    'continent':('continent-v1','continent_probe','Advanced continent geometry, cells, seed cursor and edge/ocean searches',
                 'NATIVE_CONTINENT_GUARDS_OK finiteQueries scaleBounds queryOrder parallelOwnership','passed-continent-geometry-only'),
    'populators':('populators-v1','populator_probe','RegionModule and 27 terrain/ocean/selector/region/continent blending graphs',
                  'NATIVE_POPULATOR_GUARDS_OK weights selectorBounds depthBudget nodeBudget parallelOwnership','passed-populator-composition-only'),
    'terrain-factories':('terrain-factories-v1','terrain_factory_probe','Twenty original terrain factory branches, raw noise outputs, bounds, cells and constructor cursors',
                         'NATIVE_TERRAIN_FACTORY_GUARDS_OK originalCursors ownedGraphs replayOrder parallelReaders','passed-terrain-factories-only'),
}
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()

def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--report',type=Path,required=True);p.add_argument('--family',choices=FAMILIES,default='registry');args=p.parse_args()
    if args.report.exists():p.error('Refusing to replace registry parity evidence')
    directory,binary,scope,marker,passed_status=FAMILIES[args.family]
    folder=ROOT/'horizon/tests/fixtures'/directory;fixture=folder/'values.tsv';inputs=folder/'inputs.json'
    metadata=json.loads((folder/'metadata.json').read_text(encoding='utf-8'))
    if sha(fixture)!=metadata['fixtureSha256'] or sha(inputs)!=metadata['inputsSha256'] or not metadata['freshRunsAgree']:
        raise ValueError('Independent registry inputs/observations changed')
    expected=rows(fixture.read_text(encoding='utf-8'))
    if len(expected)!=metadata['recordCount']:raise ValueError('Registry record count changed')
    manifest=ROOT/'horizon/native-adapter/Cargo.toml';logroot=ROOT/'build/qa/horizon/registry'/args.report.stem
    logroot.mkdir(parents=True,exist_ok=True)
    report={'schemaVersion':1,'status':'failed','startedAtUtc':dt.datetime.now(dt.timezone.utc).isoformat(),
        'javaOracle':metadata,'fixtureSha256':sha(fixture),'inputsSha256':sha(inputs),'profiles':[],
        'recordsPerProfile':len(expected),'fieldsPerProfile':sum(map(len,expected.values())),
        'rustc':subprocess.check_output(['rustc','+stable','-Vv'],text=True),
        'family':args.family,
        'scope':scope+'; no native terrain activation.',
        'notCovered':['Every invalid/codec parameter domain','Preset or cell/river/filtered-tile pipeline','Native density/surface/chunks','Client/save/launcher integration']}
    for profile in ('debug','release'):
        cmd=['cargo','+stable','run','--quiet','--offline','--locked','--manifest-path',str(manifest),'--bin',binary]
        if profile=='release':cmd.append('--release')
        cmd+=['--',str(inputs)]
        result=subprocess.run(cmd,cwd=ROOT,capture_output=True,text=True,timeout=120)
        output=logroot/(profile+'.tsv');output.write_text(result.stdout,encoding='utf-8',newline='\n')
        (logroot/(profile+'.stderr.log')).write_text(result.stderr,encoding='utf-8')
        actual=rows(result.stdout) if result.returncode==0 else {}
        differences=[{'key':k,'expected':expected.get(k),'actual':actual.get(k)} for k in sorted(set(actual)|set(expected)) if actual.get(k)!=expected.get(k)]
        guards=marker in result.stderr
        report['profiles'].append({'profile':profile,'exitCode':result.returncode,'outputSha256':sha(output),
            'differenceCount':len(differences),'firstDifferences':differences[:50],'rejectionGuardsObserved':guards,
            'allJavaValuesMatch':result.returncode==0 and not differences and guards})
    report['status']=passed_status if all(r['allJavaValuesMatch'] for r in report['profiles']) else 'failed'
    sources=[]
    for name in ('native-core','native-adapter'):
        base=ROOT/'horizon'/name;sources+=list((base/'src').rglob('*.rs'))+list((base/'src/data').glob('*.bin'))+[base/'Cargo.toml',base/'Cargo.lock']
    report['sourceSha256Lf']={p.relative_to(ROOT).as_posix():hashlib.sha256(p.read_bytes() if p.suffix=='.bin' else p.read_bytes().replace(b'\r\n',b'\n')).hexdigest() for p in sorted(sources)}
    report['finishedAtUtc']=dt.datetime.now(dt.timezone.utc).isoformat();report['verifierSha256']=sha(Path(__file__))
    args.report.parent.mkdir(parents=True,exist_ok=True);args.report.write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8')
    print(json.dumps({k:report[k] for k in ('status','recordsPerProfile','fieldsPerProfile')}))
    return 0 if report['status']==passed_status else 1

if __name__=='__main__':raise SystemExit(main())
