"""Compare native pillar graphs and palettes with frozen packaged Java observations."""
from __future__ import annotations
import argparse,datetime as dt,gzip,hashlib,json,os,shutil,subprocess
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
def sha(data):return hashlib.sha256(data).hexdigest()
def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--report',required=True,type=Path);a=p.parse_args()
    if a.report.exists():p.error('Refusing to replace evidence')
    fixture=ROOT/'horizon/tests/fixtures/pillar-sampler-v1';metadata=json.loads((fixture/'metadata.json').read_text(encoding='utf-8'))
    inputs=(fixture/'inputs.tsv').read_bytes();compressed=(fixture/'values.tsv.gz').read_bytes();expected=gzip.decompress(compressed)
    if sha(inputs)!=metadata['inputsSha256'] or sha(compressed)!=metadata['fixtureSha256'] or sha(expected)!=metadata['uncompressedValuesSha256']:raise ValueError('Frozen fixture bytes changed')
    for name,key in (('NativePillarSamplerProbe.java','probeSourceSha256Lf'),('NativePillarCaptureMod.java','companionSourceSha256Lf')):
        if sha((ROOT/'horizon/tests/java'/name).read_bytes().replace(b'\r\n',b'\n'))!=metadata[key]:raise ValueError('Reference producer source changed '+name)
    work=ROOT/'build/qa/horizon'/a.report.stem;work.mkdir(parents=True,exist_ok=False);shutil.copy2(Path(__file__),work/'verifier-source.py')
    env=dict(os.environ,CARGO_BUILD_JOBS='2',CARGO_NET_OFFLINE='true');env.pop('CARGO_TARGET_DIR',None);profiles=[]
    for profile in ('debug','release'):
        command=['cargo','+stable','build','--offline','--locked','--manifest-path',str(ROOT/'horizon/native-core/Cargo.toml'),'--bin','pillar_sampler_probe']+(['--release'] if profile=='release' else [])
        result=subprocess.run(command,cwd=ROOT,env=env,capture_output=True,text=True,timeout=300);(work/(profile+'-build.log')).write_text(result.stdout+result.stderr,encoding='utf-8')
        if result.returncode:raise RuntimeError('Native sampler probe failed to build')
        exe=ROOT/'horizon/native-core/target'/profile/('pillar_sampler_probe'+('.exe' if os.name=='nt' else ''))
        result=subprocess.run([str(exe),str(fixture/'inputs.tsv')],cwd=work,env=env,capture_output=True,timeout=60);(work/(profile+'-values.tsv')).write_bytes(result.stdout);(work/(profile+'-stderr.log')).write_bytes(result.stderr)
        if result.returncode:raise RuntimeError('Native sampler probe exited '+str(result.returncode))
        wanted=expected.splitlines();observed=result.stdout.splitlines();differences=[{'record':i,'java':x.decode(),'native':y.decode()} for i,(x,y) in enumerate(zip(wanted,observed)) if x!=y]
        if len(wanted)!=len(observed):differences.insert(0,{'expectedCount':len(wanted),'actualCount':len(observed)})
        profiles.append({'profile':profile,'exitCode':result.returncode,'recordCount':len(observed),'differenceCount':len(differences),'firstDifferences':differences[:5],'executableSha256':sha(exe.read_bytes()),'outputSha256':sha(result.stdout),'allJavaObservationsMatch':not differences})
    report={'schemaVersion':1,'status':'passed-independent-pillar-sampler' if all(r['allJavaObservationsMatch'] for r in profiles) else 'failed','completedAtUtc':dt.datetime.now(dt.timezone.utc).isoformat(),'recordsPerProfile':metadata['recordCount'],'referenceMetadataSha256':sha((fixture/'metadata.json').read_bytes()),'nativeSourceSha256':sha((ROOT/'horizon/native-core/src/pillar_sampler.rs').read_bytes()),'probeSourceSha256':sha((ROOT/'horizon/native-core/src/bin/pillar_sampler_probe.rs').read_bytes()),'verifierSourceSha256':sha((work/'verifier-source.py').read_bytes()),'profiles':profiles,'scope':metadata['scope'],'notCovered':metadata['notCovered']}
    a.report.parent.mkdir(parents=True,exist_ok=True);a.report.write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8',newline='\n');print(json.dumps(report,indent=2));return 0 if report['status'].startswith('passed') else 1
if __name__=='__main__':raise SystemExit(main())
