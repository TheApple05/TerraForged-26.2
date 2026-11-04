"""Check the actual staged generator's spawn entry against independent Java centers."""
from __future__ import annotations
import argparse,datetime as dt,hashlib,json,os,re,shutil,subprocess,time
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
def sha(path):return hashlib.sha256(path.read_bytes()).hexdigest()
def main():
    parser=argparse.ArgumentParser(description=__doc__)
    for name in ('workspace','resources','report'):parser.add_argument('--'+name,required=True,type=Path)
    a=parser.parse_args();workspace=a.workspace.resolve();resources=a.resources.resolve()
    if a.report.exists() or not workspace.is_relative_to((ROOT/'build/qa/horizon').resolve()) or workspace.name!='staged':parser.error('Fresh report and owned stage required')
    stage_path=workspace.parent/'stage-result.json';stage=json.loads(stage_path.read_text(encoding='utf-8'))
    if not stage['worldActivationIncluded']:raise ValueError('World stage required')
    for name,digest in stage['inputFiles'].items():
        if sha(ROOT/name)!=digest:raise ValueError('Source no longer matches stage '+name)
    source=ROOT/'horizon/tests/rust/spawn_search_probe.rs';destination=workspace/'crates/horizon-world/src/bin/terraforged_spawn_search_probe.rs'
    if destination.exists() and destination.read_bytes()!=source.read_bytes():raise ValueError('Staged spawn probe differs')
    if not destination.exists():shutil.copy2(source,destination)
    work=workspace.parent/a.report.stem;work.mkdir(exist_ok=False);shutil.copy2(Path(__file__),work/'verifier-source.py')
    fixture=ROOT/'horizon/tests/fixtures/native-spawn-v1/values.tsv';centers=dict(line.split('\t') for line in fixture.read_text(encoding='utf-8').splitlines());expected=[int(n) for n in centers['8675309/CONTINENT_CENTER'].split(',')]
    files=[resources/'facts.json']+sorted((resources/'worldgen').rglob('*.json'));before={p.relative_to(resources).as_posix():sha(p) for p in files}
    env=dict(os.environ,CARGO_BUILD_JOBS='4',CARGO_NET_OFFLINE='true',HORIZON_NO_AUDIO='1',HORIZON_NO_DISCORD='1',HORIZON_LAUNCHER_NO_SELF_UPDATE='1',HORIZON_TELEMETRY_URL='off');env.pop('CARGO_TARGET_DIR',None)
    build=subprocess.run(['cargo','+stable','build','--offline','--locked','--manifest-path',str(workspace/'Cargo.toml'),'--profile','fast','-p','horizon-world','--bin','terraforged_spawn_search_probe'],capture_output=True,text=True,env=env,timeout=300);(work/'build.log').write_text(build.stdout+build.stderr,encoding='utf-8')
    exe=workspace/'target/fast'/('terraforged_spawn_search_probe'+('.exe' if os.name=='nt' else ''));runs=[]
    if build.returncode==0:
        for index in range(2):
            for name in ('HORIZON_HOME','HORIZON_LAUNCHER_HOME','XDG_DATA_HOME','XDG_CONFIG_HOME','HORIZON_LAUNCHER_TRASH','HORIZON_LAUNCHER_CLIPBOARD','HORIZON_LAUNCHER_DESKTOP'):
                home=work/f'run-{index}'/name.lower();home.mkdir(parents=True);env[name]=str(home)
            start=time.monotonic();run=subprocess.run([str(exe),str(resources)],cwd=work,env=env,capture_output=True,text=True,timeout=180);out=work/f'run-{index}.log';out.write_text(run.stdout+run.stderr,encoding='utf-8')
            kernel=re.search(r'CORE_CONTINENT_CENTER (-?\d+) (-?\d+)',run.stdout);actual=re.search(r'NATIVE_SPAWN_RESULT (-?\d+) (-?\d+) elapsedSeconds=(\S+)',run.stdout)
            runs.append({'exitCode':run.returncode,'processSeconds':time.monotonic()-start,'kernelCenter':[int(kernel[1]),int(kernel[2])] if kernel else None,'generatorCenter':[int(actual[1]),int(actual[2])] if actual else None,'spawnCallSeconds':float(actual[3]) if actual else None,'logSha256':sha(out)})
    unchanged=all(sha(resources/name)==digest for name,digest in before.items());passed=build.returncode==0 and unchanged and len(runs)==2 and all(r['exitCode']==0 and r['kernelCenter']==expected and r['generatorCenter']==expected for r in runs)
    report={'schemaVersion':1,'status':'passed-staged-native-spawn-factory' if passed else 'failed','completedAtUtc':dt.datetime.now(dt.timezone.utc).isoformat(),'horizonRevision':stage['horizonRevision'],'stageResultSha256':sha(stage_path),'probeSourceSha256':sha(source),'verifierSourceSha256':sha(work/'verifier-source.py'),'referenceSha256':sha(fixture),'expectedJavaConfiguredCenter':expected,'compileExitCode':build.returncode,'executableSha256':sha(exe) if build.returncode==0 else None,'runs':runs,'resourcesUnchanged':unchanged,'resourceFilesSha256':before,'gameAssetsDownloaded':False,'timingsAreControlledPerformanceEvidence':False,'notCovered':['Actual player spawn and height selection','Other seeds in this factory smoke (separate adapter fixture covers eight)','Source carvers, native biome features, standard launcher, native modpacks and performance']}
    a.report.parent.mkdir(parents=True,exist_ok=True);a.report.write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8',newline='\n');print(json.dumps({k:v for k,v in report.items() if k!='resourceFilesSha256'},indent=2));return 0 if passed else 1
if __name__=='__main__':raise SystemExit(main())
