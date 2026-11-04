"""Qualify the staged real Horizon graph against independently captured Java."""
from __future__ import annotations
import argparse
import datetime as dt
import gzip
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import os

ROOT=Path(__file__).resolve().parents[2]
MARKER='realHorizonGraph scalar bulk conservativeBounds parallelContexts releasedWorldOwner'
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def rows(data):
    lines=data.splitlines();result=dict(line.split('\t',1) for line in lines)
    if len(result)!=len(lines):raise ValueError('Duplicate result keys')
    return result

def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--workspace',required=True,type=Path);p.add_argument('--report',required=True,type=Path)
    p.add_argument('--registry',action='store_true',help='Exercise actual Horizon JSON registry decoding with explicit scoped bindings')
    a=p.parse_args();workspace=a.workspace.resolve()
    if not workspace.is_relative_to((ROOT/'build/qa/horizon').resolve()) or workspace.name!='staged':p.error('Use a freshly staged workspace below build/qa/horizon')
    if a.report.exists():p.error('Refusing to replace host graph qualification evidence')
    stage_result=workspace.parent/'stage-result.json';staging=json.loads(stage_result.read_text(encoding='utf-8'))
    if staging['horizonRevision']!='e4d7699a8d4ff5b18a966601b9c3e8346f4571e5' or staging['worldActivationIncluded']:raise ValueError('Wrong host stage')
    if bool(staging.get('registryDecoderIncluded',False))!=a.registry:raise ValueError('Wrong registry staging mode')
    for name,digest in staging['inputFiles'].items():
        if sha(ROOT/name)!=digest:raise ValueError('Restage changed input '+name)
    fixture=ROOT/'horizon/tests/fixtures/host-density-v1';meta=json.loads((fixture/'metadata.json').read_text(encoding='utf-8'))
    compressed=fixture/'values.tsv.gz';inputs=fixture/'inputs.json';data=gzip.decompress(compressed.read_bytes())
    if meta['status']!='frozen-genuine-java-host-density-reference' or not meta['freshRunsAgree'] or sha(compressed)!=meta['fixtureSha256'] or sha(inputs)!=meta['inputsSha256'] or hashlib.sha256(data).hexdigest()!=meta['uncompressedValuesSha256']:raise ValueError('Independent reference changed')
    expected=rows(data.decode())
    package='horizon-world' if a.registry else 'horizon-core'
    binary='terraforged_registry_host_probe' if a.registry else 'terraforged_density_host_probe'
    source=ROOT/('horizon/tests/rust/host_registry_probe.rs' if a.registry else 'horizon/tests/rust/host_density_probe.rs')
    destination=workspace/f'crates/{package}/src/bin/{binary}.rs'
    if destination.exists() and destination.read_bytes()!=source.read_bytes():raise ValueError('Existing probe differs')
    if not destination.exists():shutil.copy2(source,destination)
    work=workspace.parent/a.report.stem
    work.mkdir(exist_ok=False)
    report={'schemaVersion':1,'status':'failed','registryDecoderIncluded':a.registry,'startedAtUtc':dt.datetime.now(dt.timezone.utc).isoformat(),
        'horizonRevision':staging['horizonRevision'],'stageResultSha256':sha(stage_result),
        'referenceMetadataSha256':sha(fixture/'metadata.json'),'probeSourceSha256':sha(source),
        'inputSha256':sha(inputs),'recordsPerProfile':len(expected),'profiles':[],
        'scope':('Actual scoped Horizon JSON registry decoding and compiled graph evaluation. ' if a.registry else '')+'Compiled Horizon graph scalar/bulk f32 outputs using the assembled procedural kernel, ten cell fields, three clamp resolutions, two noise bindings and one addition graph.',
        'notCovered':['World activation','Full router conversion','f64 vanilla chunk parity','Biomes/surfaces/caves/features integration','Native client/save/modpack/launcher acceptance']}
    env=dict(os.environ,CARGO_BUILD_JOBS='4',CARGO_NET_OFFLINE='true')
    for profile in ('debug','release'):
        common=['cargo','+stable'];suffix=['--release'] if profile=='release' else []
        test=subprocess.run(common+['test','--quiet','--offline','--locked','--manifest-path',str(workspace/'Cargo.toml'),'-p','horizon-core','--lib']+suffix+['terraforged_'],capture_output=True,text=True,env=env,timeout=300)
        (work/(profile+'-lifecycle.log')).write_text(test.stdout+test.stderr,encoding='utf-8')
        lifecycle=test.returncode==0 and '2 passed; 0 failed' in test.stdout
        command=common+['run','--quiet','--offline','--locked','--manifest-path',str(workspace/'Cargo.toml'),'-p',package,'--bin',binary]+suffix+['--',str(inputs)]
        run=subprocess.run(command,capture_output=True,text=True,env=env,timeout=300)
        log=work/(profile+'.tsv');log.write_text(run.stdout,encoding='utf-8',newline='\n')
        (work/(profile+'.stderr.log')).write_text(run.stderr,encoding='utf-8')
        actual=rows(run.stdout) if run.returncode==0 else {}
        differences=[{'key':k,'expected':expected.get(k),'actual':actual.get(k)} for k in sorted(set(expected)|set(actual)) if expected.get(k)!=actual.get(k)]
        marker=('NATIVE_HOST_REGISTRY_OK '+str(len(expected))+' records realHorizonRegistry scalar bulk isolatedScopes boundedNesting threadIsolation panicCleanup errorsRecover vanillaUnchanged releasedWorldOwner') if a.registry else f'NATIVE_HOST_DENSITY_OK {len(expected)} records '+MARKER
        guarded=marker in run.stderr
        report['profiles'].append({'profile':profile,'exitCode':run.returncode,'lifecycleTestsPassed':lifecycle,
            'lifecycleLogSha256':sha(work/(profile+'-lifecycle.log')),'guardsObserved':guarded,
            'differenceCount':len(differences),'firstDifferences':differences[:30],
            'outputSha256':sha(log),'allJavaValuesMatch':not differences and run.returncode==0 and guarded and lifecycle})
    passed='passed-real-horizon-density-registry' if a.registry else 'passed-real-horizon-density-graph'
    if all(item['allJavaValuesMatch'] for item in report['profiles']):report['status']=passed
    report['completedAtUtc']=dt.datetime.now(dt.timezone.utc).isoformat()
    a.report.parent.mkdir(parents=True,exist_ok=True)
    a.report.write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8',newline='\n')
    print(json.dumps(report,indent=2))
    return 0 if report['status']==passed else 1

if __name__=='__main__':raise SystemExit(main())
