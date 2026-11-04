"""Run the exact source-built native client against a fresh save, then reopen it.

The fixture explicitly supplies a native saved stem, spawn and no structures.
It does not qualify UI creation, natural spawn, native features or performance.
"""
from __future__ import annotations
import argparse, datetime as dt, hashlib, json, os, shutil, subprocess, time
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]

def sha(path):return hashlib.sha256(path.read_bytes()).hexdigest()
def main():
    p=argparse.ArgumentParser(description=__doc__)
    for n in ('workspace','resources','report'):p.add_argument('--'+n,required=True,type=Path)
    a=p.parse_args();workspace=a.workspace.resolve();source_resources=a.resources.resolve()
    if not workspace.is_relative_to((ROOT/'build/qa/horizon').resolve()) or workspace.name!='staged':p.error('Fresh full-source stage below build/qa/horizon required')
    if a.report.exists():p.error('Refusing to replace client evidence')
    stage_file=workspace.parent/'stage-result.json';stage=json.loads(stage_file.read_text(encoding='utf-8'))
    if not stage.get('fullEngineSourceIncluded') or not stage['automaticWorldSelectionIncluded']:p.error('Full entry stage required')
    for name,digest in stage['inputFiles'].items():
        if sha(ROOT/name)!=digest:raise ValueError('Restage changed source '+name)
    work=workspace.parent/a.report.stem;work.mkdir(exist_ok=False)
    verifier_snapshot=work/'verifier-source.py';shutil.copy2(Path(__file__),verifier_snapshot)
    env=dict(os.environ,CARGO_BUILD_JOBS='4',CARGO_NET_OFFLINE='true',HORIZON_NO_AUDIO='1',HORIZON_NO_DISCORD='1',HORIZON_LAUNCHER_NO_SELF_UPDATE='1',HORIZON_TELEMETRY_URL='off',HORIZON_PLAYER_NAME='TerraForgedQA')
    for name in ('HORIZON_HOME','HORIZON_LAUNCHER_HOME','XDG_DATA_HOME','XDG_CONFIG_HOME','HORIZON_LAUNCHER_TRASH','HORIZON_LAUNCHER_CLIPBOARD','HORIZON_LAUNCHER_DESKTOP'):
        path=work/'homes'/name.lower();path.mkdir(parents=True);env[name]=str(path.resolve())
    game=Path(env['HORIZON_HOME']);world=game/'saves/NativeQA';socket=game/'control.sock'
    (game/'options.txt').write_text('onboardAccessibility:false\nrenderDistance:6\nsimulationDistance:5\nmaxFps:30\ntutorialStep:none\n',encoding='utf-8')
    checked_files=[source_resources/'horizon-prepare.json',source_resources/'horizon-resources.json',source_resources/'facts.json']+sorted((source_resources/'worldgen').rglob('*.json'))
    before={path.relative_to(source_resources).as_posix():sha(path) for path in checked_files}
    resources=work/'prepared-resources'
    report={'schemaVersion':1,'status':'failed','startedAtUtc':dt.datetime.now(dt.timezone.utc).isoformat(),
        'horizonRevision':stage['horizonRevision'],'stageResultSha256':sha(stage_file),'verifierSourceSha256':sha(verifier_snapshot),'runs':[],
        'qaSpawnSupplied':True,'structuresEnabled':False,'renderDistanceChunks':6,'gameAssetsDownloaded':False,
        'sourceResourceRoot':str(source_resources),'isolatedResourceRoot':str(resources),
        'notCovered':['UI world creation','Natural spawn','Structures','17 native classic biomes/features','Independent post-decoration block parity','Standard launcher installation','Controlled native performance and modpack qualification']}
    process=None;log_file=None
    def record_run(command,label,timeout=120):
        result=subprocess.run(command,cwd=game,env=env,capture_output=True,text=True,timeout=timeout)
        (work/(label+'.stdout.log')).write_text(result.stdout,encoding='utf-8',newline='\n')
        (work/(label+'.stderr.log')).write_text(result.stderr,encoding='utf-8')
        if result.returncode:raise RuntimeError(f'{label}: process exit {result.returncode}; see preserved logs')
        return result
    try:
        # Ordinary copies avoid sharing writable cache files with the player's
        # prepared resources. This never invokes a downloader or Java exporter.
        shutil.copytree(source_resources,resources)
        if not all(sha(resources/name)==digest for name,digest in before.items()):raise ValueError('Prepared resource copy differs from the recorded inputs')
        probe=ROOT/'horizon/tests/rust/save_fixture_probe.rs';dest=workspace/'crates/horizon-client-lib/qa/terraforged_save_fixture.rs'
        if dest.exists() and dest.read_bytes()!=probe.read_bytes():raise ValueError('Existing QA probe differs')
        dest.parent.mkdir(parents=True,exist_ok=True)
        if not dest.exists():shutil.copy2(probe,dest)
        manifest=workspace/'crates/horizon-client-lib/Cargo.toml';text=manifest.read_text(encoding='utf-8')
        addition='\n[[bin]]\nname = "terraforged_save_fixture"\npath = "qa/terraforged_save_fixture.rs"\n'
        report['qaManifestBeforeSha256']=sha(manifest)
        if 'name = "terraforged_save_fixture"' not in text:manifest.write_text(text+addition,encoding='utf-8',newline='\n')
        elif addition.strip() not in text:raise ValueError('Conflicting fixture target')
        report['qaManifestAfterSha256']=sha(manifest);report['fixtureProbeSha256']=sha(probe)
        record_run(['cargo','+stable','build','--offline','--locked','--manifest-path',str(workspace/'Cargo.toml'),'--profile','fast','-p','horizon-client','-p','horizon-client-lib','--bin','horizon-client','--bin','horizon-ctl','--bin','terraforged_save_fixture'],'build',1200)
        suffix='.exe' if os.name=='nt' else '';target=workspace/'target/fast'
        client=target/('horizon-client'+suffix);ctl=target/('horizon-ctl'+suffix);fixture=target/('terraforged_save_fixture'+suffix)
        report['artifactsSha256']={path.name:sha(path) for path in (client,ctl,fixture)}
        preset=ROOT/'horizon/native-content/data/terraforged/worldgen/world_preset/default.json'
        def fixture_run(action,label):
            result=record_run([str(fixture),action,str(world),str(preset),str(resources)],label,180)
            return json.loads(result.stdout)
        report['fixtureInitialization']=fixture_run('init','fixture-init')
        def control(words,label,timeout=30):
            result=record_run([str(ctl),'--socket',str(socket)]+words,label,timeout)
            lines=result.stdout.strip().splitlines();reply=json.loads(lines[-1]) if lines else {}
            if 'error' in reply:raise RuntimeError(f'{label}: game rejected command')
            return reply
        def loaded_view(index,x,y,z):
            # wait-ready alone can observe an empty client queue between a
            # server teleport and its first new chunk packet. Require the
            # requested chunk and actual resident/drawn terrain as well.
            deadline=time.monotonic()+420
            for attempt in range(1000):
                snapshot=control(['status'],f'game-{index}-view-{attempt}',10)
                block=control(['block',str(x),str(y),str(z)],f'game-{index}-block-{attempt}',10)
                renderer=snapshot.get('renderer') or {}
                if (block.get('loaded') and snapshot.get('chunks',0)>=81
                    and not snapshot.get('pending_chunks',0) and not snapshot.get('pending_uploads',0)
                    and not any((snapshot.get('streaming') or {}).values())
                    and renderer.get('resident_sections',0)>0 and renderer.get('drawn_sections',0)>0):
                    control(['wait','20'],f'game-{index}-settle',15)
                    return snapshot
                if time.monotonic()>deadline:raise RuntimeError('Requested chunk/visible terrain did not become ready')
                control(['wait','10'],f'game-{index}-wait-{attempt}',15)
            raise RuntimeError('Requested chunk/visible terrain wait exhausted')
        for index in range(2):
            log_file=(work/f'game-{index}.log').open('w',encoding='utf-8')
            smoke=work/f'game-{index}-smoke.json'
            cmd=[str(client),'--resources',str(resources),'--game-dir',str(game),'--home',str(game),'--world',str(world),'--name','TerraForgedQA','--offscreen','--control','--control-socket',str(socket),'--render-distance','6','--width','1280','--height','720','--timeout-seconds','900','--smoke-report',str(smoke)]
            started=time.monotonic();creationflags=subprocess.CREATE_NO_WINDOW if os.name=='nt' else 0
            process=subprocess.Popen(cmd,cwd=game,env=env,stdout=log_file,stderr=subprocess.STDOUT,creationflags=creationflags)
            # Each ctl attempt is bounded; failed startup attempts remain in logs.
            for attempt in range(150):
                if process.poll() is not None:raise RuntimeError('Game exited before its control endpoint was ready')
                try:
                    state=control(['state'],f'game-{index}-startup-{attempt}',5);break
                except (RuntimeError,subprocess.TimeoutExpired):time.sleep(1)
            else:raise RuntimeError('Control endpoint did not become ready')
            control(['wait-ready','180'],f'game-{index}-near-ready',210)
            if index==0:
                control(['command','tp @s 8.5 90 8.5'],f'game-{index}-camera')
                control(['turn','0','-45'],f'game-{index}-look')
                visible=loaded_view(index,8,75,8)
            else:
                control(['command','tp @s 16392 80 -32760'],f'game-{index}-teleport')
                visible=loaded_view(index,16392,65,-32760)
            state=control(['state'],f'game-{index}-state')
            image=work/f'game-{index}.png';control(['screenshot',str(image)],f'game-{index}-screenshot',60)
            if not image.is_file() or image.stat().st_size<1024:raise RuntimeError('Game did not produce a usable PNG')
            # The stable quit command saves the real world before exiting.
            control(['quit'],f'game-{index}-quit',120)
            code=process.wait(timeout=120);process=None;log_file.close();log_file=None
            if code:raise RuntimeError(f'Game exit code {code}')
            saved=fixture_run('check_all' if index else 'check',f'game-{index}-saved-biomes')
            report['runs'].append({'index':index,'exitCode':code,'elapsedSeconds':time.monotonic()-started,'state':state,
                'visibleTerrainState':visible,'savedBiomeChecks':saved,'screenshot':str(image),'screenshotSha256':sha(image),'smokeReportSha256':sha(smoke) if smoke.exists() else None,
                'gameLogSha256':sha(work/f'game-{index}.log')})
        report['status']='passed-native-client-save-smoke-awaiting-image-review';report['renderedImagesNeedReview']=True
    except Exception as error:
        report['error']=str(error)
    finally:
        if process is not None and process.poll() is None:
            try:
                subprocess.run([str(ctl),'--socket',str(socket),'quit'],cwd=game,env=env,capture_output=True,text=True,timeout=15)
                process.wait(timeout=15)
            except Exception:
                process.terminate()
                try:process.wait(timeout=10)
                except subprocess.TimeoutExpired:process.kill();process.wait()
        if log_file is not None:log_file.close()
        report['checkedResourcesUnchanged']=all(sha(source_resources/name)==digest for name,digest in before.items())
        report['checkedResourceCopyUnchanged']=resources.exists() and all(sha(resources/name)==digest for name,digest in before.items())
        if not report['checkedResourcesUnchanged']:report['status']='failed';report['error']='Checked resource bytes changed'
        report['checkedResourceFilesSha256']=before;report['completedAtUtc']=dt.datetime.now(dt.timezone.utc).isoformat()
        a.report.parent.mkdir(parents=True,exist_ok=True);a.report.write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8',newline='\n')
    print(json.dumps({k:v for k,v in report.items() if k not in ('checkedResourceFilesSha256','fixtureInitialization','runs')},indent=2))
    return 0 if report['status'].startswith('passed-') else 1
if __name__=='__main__':raise SystemExit(main())
