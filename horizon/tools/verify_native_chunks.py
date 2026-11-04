"""Run the exact native procedural chunk executable in isolated fresh processes.

Uses only configuration plus the player's prepared resources. This prototype
has no automatic world-selection hook or snapshot importer.
"""
from __future__ import annotations
import argparse,datetime as dt,hashlib,json,os,shutil,subprocess,time
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
MARKER='NATIVE_PROCEDURAL_CHUNKS_OK distantCoordinates realNoiseAquifersMaterialsCarvers actualDecoration coldRegeneration releasedKernelOwner'
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def main():
    p=argparse.ArgumentParser(description=__doc__)
    for n in ('workspace','resources','report'):p.add_argument('--'+n,required=True,type=Path)
    p.add_argument('--profile',choices=['debug','release'],default='release');p.add_argument('--repeat',type=int,choices=[1,2,3],default=2)
    a=p.parse_args();workspace=a.workspace.resolve();resources=a.resources.resolve()
    if not workspace.is_relative_to((ROOT/'build/qa/horizon').resolve()) or workspace.name!='staged':p.error('Fresh world stage below build/qa/horizon required')
    if a.report.exists():p.error('Refusing to replace native chunk evidence')
    stage_path=workspace.parent/'stage-result.json';stage=json.loads(stage_path.read_text(encoding='utf-8'))
    if not stage['worldActivationIncluded'] or stage['automaticWorldSelectionIncluded']:raise ValueError('Explicit world-factory stage required')
    for name,digest in stage['inputFiles'].items():
        if sha(ROOT/name)!=digest:raise ValueError('Restage changed source '+name)
    source=ROOT/'horizon/tests/rust/native_chunk_probe.rs';dest=workspace/'crates/horizon-world/src/bin/terraforged_native_chunk_probe.rs'
    if dest.exists() and dest.read_bytes()!=source.read_bytes():raise ValueError('Existing probe differs')
    if not dest.exists():shutil.copy2(source,dest)
    work=workspace.parent/a.report.stem;work.mkdir(exist_ok=False)
    files=[resources/'facts.json']+sorted((resources/'worldgen').rglob('*.json'))
    before={f.relative_to(resources).as_posix():sha(f) for f in files}
    env=dict(os.environ,CARGO_BUILD_JOBS='4',CARGO_NET_OFFLINE='true',HORIZON_TELEMETRY_URL='off',HORIZON_LAUNCHER_NO_SELF_UPDATE='1',HORIZON_NO_AUDIO='1')
    build=['cargo','+stable','build','--quiet','--offline','--locked','--manifest-path',str(workspace/'Cargo.toml'),'-p','horizon-world','--bin','terraforged_native_chunk_probe']
    if a.profile=='release':build.append('--release')
    compiled=subprocess.run(build,capture_output=True,text=True,env=env,timeout=300)
    (work/'build.log').write_text(compiled.stdout+compiled.stderr,encoding='utf-8')
    report={'schemaVersion':1,'status':'failed','startedAtUtc':dt.datetime.now(dt.timezone.utc).isoformat(),'horizonRevision':stage['horizonRevision'],
        'stageResultSha256':sha(stage_path),'probeSourceSha256':sha(source),'profile':a.profile,'compileExitCode':compiled.returncode,
        'resourceRoot':str(resources),'resourceFilesSha256':before,'runs':[],'gameAssetsDownloaded':False,
        'notCovered':['17 native classic biome variants','Source carver configuration','Independent whole-chunk Java parity','Structures/spawn/save/client/launcher/modpack acceptance']}
    executable=workspace/'target'/a.profile/('terraforged_native_chunk_probe'+('.exe' if os.name=='nt' else ''))
    inputs=ROOT/'horizon/tests/fixtures/host-density-v1/inputs.json'
    normalized=[]
    if compiled.returncode==0:
        report['executableSha256']=sha(executable);report['configurationSha256']=sha(inputs)
        for index in range(a.repeat):
            isolation=work/f'run-{index}-homes'
            for name in ('HORIZON_HOME','HORIZON_LAUNCHER_HOME','XDG_DATA_HOME','XDG_CONFIG_HOME','HORIZON_LAUNCHER_TRASH','HORIZON_LAUNCHER_CLIPBOARD','HORIZON_LAUNCHER_DESKTOP'):
                directory=isolation/name.lower();directory.mkdir(parents=True);env[name]=str(directory.resolve())
            start=time.monotonic()
            try:
                run=subprocess.run([str(executable),str(inputs),str(resources)],capture_output=True,text=True,env=env,timeout=600)
                stdout,stderr,code=run.stdout,run.stderr,run.returncode
            except subprocess.TimeoutExpired as error:
                stdout=(error.stdout or b'').decode() if isinstance(error.stdout,bytes) else error.stdout or ''
                stderr=(error.stderr or b'').decode() if isinstance(error.stderr,bytes) else error.stderr or ''
                code=-999;stderr+='\nQA_TIMEOUT_600_SECONDS\n'
            out=work/f'run-{index}.json';out.write_text(stdout,encoding='utf-8',newline='\n');(work/f'run-{index}.stderr.log').write_text(stderr,encoding='utf-8')
            result=json.loads(stdout) if code==0 else None
            okay=code==0 and MARKER in stderr and result['status']=='passed-native-procedural-terrain-prototype' and result['releasedKernelOwner']
            report['runs'].append({'index':index,'exitCode':code,'elapsedSeconds':time.monotonic()-start,'outputSha256':sha(out),'guardsObserved':MARKER in stderr,'passed':okay})
            if okay:
                result.pop('generationSeconds',None);normalized.append(result)
            else:break
    report['freshProcessesAgree']=len(normalized)==a.repeat and all(row==normalized[0] for row in normalized)
    report['resourcesUnchanged']=all(sha(resources/name)==digest for name,digest in before.items())
    if compiled.returncode==0 and report['freshProcessesAgree'] and report['resourcesUnchanged'] and all(row['passed'] for row in report['runs']):report['status']='passed-native-procedural-chunk-prototype'
    report['completedAtUtc']=dt.datetime.now(dt.timezone.utc).isoformat();a.report.parent.mkdir(parents=True,exist_ok=True)
    a.report.write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8',newline='\n');print(json.dumps({k:v for k,v in report.items() if k!='resourceFilesSha256'},indent=2))
    return 0 if report['status']=='passed-native-procedural-chunk-prototype' else 1
if __name__=='__main__':raise SystemExit(main())
