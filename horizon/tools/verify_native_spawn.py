"""Compare configured native search centers with independently captured packaged Java."""
from __future__ import annotations
import argparse,datetime as dt,hashlib,json,os,shutil,subprocess
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
def sha(path):return hashlib.sha256(path.read_bytes()).hexdigest()
def main():
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--report',required=True,type=Path);a=parser.parse_args()
    if a.report.exists():parser.error('Refusing to replace evidence')
    fixture=ROOT/'horizon/tests/fixtures/native-spawn-v1';metadata=json.loads((fixture/'metadata.json').read_text(encoding='utf-8'));inputs=fixture/'inputs.json';values=fixture/'values.tsv'
    if sha(inputs)!=metadata['inputsSha256'] or sha(values)!=metadata['valuesSha256'] or sha(ROOT/'horizon/tests/java/NativeSpawnCaptureMod.java')!=metadata['probeSourceSha256']:raise ValueError('Frozen source/reference bytes changed')
    expected=values.read_bytes();work=ROOT/'build/qa/horizon'/a.report.stem;work.mkdir(parents=True,exist_ok=False);shutil.copy2(Path(__file__),work/'verifier-source.py')
    env=dict(os.environ,CARGO_BUILD_JOBS='2',CARGO_NET_OFFLINE='true');env.pop('CARGO_TARGET_DIR',None);profiles=[]
    for profile in ('debug','release'):
        command=['cargo','+stable','build','--offline','--locked','--manifest-path',str(ROOT/'horizon/native-adapter/Cargo.toml'),'--bin','spawn_center_probe']+(['--release'] if profile=='release' else [])
        result=subprocess.run(command,cwd=ROOT,env=env,capture_output=True,text=True,timeout=300);(work/(profile+'-build.log')).write_text(result.stdout+result.stderr,encoding='utf-8')
        if result.returncode:raise RuntimeError('Spawn probe build failed')
        exe=ROOT/'horizon/native-adapter/target'/profile/('spawn_center_probe'+('.exe' if os.name=='nt' else ''))
        result=subprocess.run([str(exe),str(inputs)],cwd=work,env=env,capture_output=True,timeout=60);actual=work/(profile+'-values.tsv');actual.write_bytes(result.stdout);(work/(profile+'-stderr.log')).write_bytes(result.stderr)
        guards=b'NATIVE_SPAWN_CENTERS_OK independentConfiguredCenters invalidModesRejected longSeedWrapping' in result.stderr
        profiles.append({'profile':profile,'exitCode':result.returncode,'records':len(result.stdout.splitlines()),'executableSha256':sha(exe),'outputSha256':sha(actual),'allJavaObservationsMatch':result.stdout==expected,'invalidModeGuardsObserved':guards})
    passed=all(x['exitCode']==0 and x['allJavaObservationsMatch'] and x['invalidModeGuardsObserved'] and x['records']==24 for x in profiles)
    report={'schemaVersion':1,'status':'passed-independent-configured-spawn-centers' if passed else 'failed','completedAtUtc':dt.datetime.now(dt.timezone.utc).isoformat(),'referenceMetadataSha256':sha(fixture/'metadata.json'),'nativeSourceSha256':sha(ROOT/'horizon/native-adapter/src/spawn.rs'),'probeSourceSha256':sha(ROOT/'horizon/native-adapter/src/bin/spawn_center_probe.rs'),'verifierSourceSha256':sha(work/'verifier-source.py'),'profiles':profiles,'scope':metadata['scope'],'notCovered':metadata['notCovered']}
    a.report.parent.mkdir(parents=True,exist_ok=True);a.report.write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8',newline='\n');print(json.dumps(report,indent=2));return 0 if passed else 1
if __name__=='__main__':raise SystemExit(main())
