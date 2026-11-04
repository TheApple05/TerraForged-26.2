"""Exercise eight compiled router fields with the player's existing resources.

This is a router integration/guard check, not independent full Java chunk parity.
No game resources are downloaded or changed; source JSON bytes are only hashed.
"""
from __future__ import annotations
import argparse,datetime as dt,hashlib,json,os,shutil,subprocess,time
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
MARKER='NATIVE_HOST_ROUTER_OK 22464 records eightFields finiteValues surfaceHeightDomain surfaceIgnoresY worldLocalDefinitions vanillaUnchanged releasedWorldOwner'
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def main():
    p=argparse.ArgumentParser(description=__doc__)
    for name in ('workspace','resources','report'):p.add_argument('--'+name,type=Path,required=True)
    a=p.parse_args();workspace=a.workspace.resolve();resources=a.resources.resolve()
    if not workspace.is_relative_to((ROOT/'build/qa/horizon').resolve()) or workspace.name!='staged':p.error('Fresh staged workspace below build/qa/horizon required')
    if a.report.exists():p.error('Refusing to replace router integration evidence')
    staging_path=workspace.parent/'stage-result.json';staging=json.loads(staging_path.read_text(encoding='utf-8'))
    if not staging.get('routerDefinitionSupplyIncluded') or staging['worldActivationIncluded']:raise ValueError('Router-only stage required')
    for name,digest in staging['inputFiles'].items():
        if sha(ROOT/name)!=digest:raise ValueError('Restage changed source '+name)
    files=[file for kind in ('noise','density_function') for file in sorted((resources/'data/minecraft/worldgen'/kind).rglob('*.json'))]
    if not files:raise ValueError('Own prepared vanilla resources are absent; no downloads attempted')
    resources_before={f.relative_to(resources).as_posix():sha(f) for f in files}
    source=ROOT/'horizon/tests/rust/host_router_probe.rs';destination=workspace/'crates/horizon-world/src/bin/terraforged_router_host_probe.rs'
    if destination.exists() and destination.read_bytes()!=source.read_bytes():raise ValueError('Existing probe differs')
    if not destination.exists():shutil.copy2(source,destination)
    work=workspace.parent/a.report.stem;work.mkdir(exist_ok=False)
    inputs=ROOT/'horizon/tests/fixtures/host-density-v1/inputs.json'
    report={'schemaVersion':1,'status':'failed','startedAtUtc':dt.datetime.now(dt.timezone.utc).isoformat(),
        'horizonRevision':staging['horizonRevision'],'stageResultSha256':sha(staging_path),'probeSourceSha256':sha(source),
        'configurationSha256':sha(inputs),'resourceRoot':str(resources),'resourceFilesSha256':resources_before,
        'recordsPerProfile':22464,'profiles':[],'gameAssetsDownloaded':False,
        'scope':'Actual Horizon registry and all eight compiled router fields using own existing vanilla noise/density resources. Finite outputs, surface height/range, scoped definitions, vanilla preservation and owner release.',
        'notCovered':['Independent full Java density/router/chunk parity','World activation and real chunks','Complete caves/aquifers/materials/features integration','Native client/save/modpack/launcher acceptance']}
    env=dict(os.environ,CARGO_NET_OFFLINE='true',CARGO_BUILD_JOBS='4',HORIZON_TELEMETRY_URL='off',HORIZON_LAUNCHER_NO_SELF_UPDATE='1',HORIZON_NO_AUDIO='1')
    for name in ('HORIZON_HOME','HORIZON_LAUNCHER_HOME','XDG_DATA_HOME','XDG_CONFIG_HOME','HORIZON_LAUNCHER_TRASH','HORIZON_LAUNCHER_CLIPBOARD','HORIZON_LAUNCHER_DESKTOP'):
        directory=work/'isolation'/name.lower();directory.mkdir(parents=True);env[name]=str(directory.resolve())
    for profile in ('debug','release'):
        command=['cargo','+stable','run','--quiet','--offline','--locked','--manifest-path',str(workspace/'Cargo.toml'),'-p','horizon-world','--bin','terraforged_router_host_probe']
        if profile=='release':command.append('--release')
        command+=['--',str(inputs),str(resources)]
        started=time.monotonic();run=subprocess.run(command,capture_output=True,text=True,env=env,timeout=600)
        out=work/(profile+'.tsv');out.write_text(run.stdout,encoding='utf-8',newline='\n');(work/(profile+'.stderr.log')).write_text(run.stderr,encoding='utf-8')
        rows=run.stdout.splitlines();keys=[line.split('\t',1)[0] for line in rows]
        valid=run.returncode==0 and len(rows)==22464 and len(set(keys))==22464 and MARKER in run.stderr
        report['profiles'].append({'profile':profile,'exitCode':run.returncode,'recordCount':len(rows),'guardsObserved':MARKER in run.stderr,
            'outputSha256':sha(out),'integrationPassed':valid,'elapsedSeconds':time.monotonic()-started})
    report['resourcesUnchanged']=all(sha(resources/name)==digest for name,digest in resources_before.items())
    report['profilesAgree']=report['profiles'][0]['outputSha256']==report['profiles'][1]['outputSha256']
    if report['resourcesUnchanged'] and report['profilesAgree'] and all(row['integrationPassed'] for row in report['profiles']):report['status']='passed-eight-field-router-integration-guards'
    report['completedAtUtc']=dt.datetime.now(dt.timezone.utc).isoformat();a.report.parent.mkdir(parents=True,exist_ok=True)
    a.report.write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8',newline='\n');print(json.dumps({k:v for k,v in report.items() if k!='resourceFilesSha256'},indent=2))
    return 0 if report['status']=='passed-eight-field-router-integration-guards' else 1
if __name__=='__main__':raise SystemExit(main())
