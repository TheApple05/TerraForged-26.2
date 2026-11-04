"""Capture packaged Java spawn centers in real, isolated Fabric servers using local files."""
from __future__ import annotations
import argparse,datetime as dt,hashlib,json,os,re,shutil,socket,subprocess,zipfile
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
PRODUCTION='1c9e5e7e8d286c145a647f466b2f908cda721d10672c40949864f1015570e886'
def sha(path):return hashlib.sha256(path.read_bytes()).hexdigest()
def main():
    parser=argparse.ArgumentParser(description=__doc__)
    for name in ('java-home','production-jar','classpath-file','server-instance','work','output'):parser.add_argument('--'+name,required=True,type=Path)
    a=parser.parse_args();work=a.work.resolve();output=a.output.resolve();jar=a.production_jar.resolve();template=a.server_instance.resolve()
    if work.exists() or output.exists() or not work.is_relative_to((ROOT/'build/qa/horizon').resolve()):parser.error('Fresh QA/fixture directories required')
    if sha(jar)!=PRODUCTION:raise ValueError('Exact dev23 Fabric artifact required')
    checked={'server.jar':'cdacdfb25898de5e4b4b0e5ddcc2722f77067e46605709c2d886c000ebb63ec5','fabric-server-launch.jar':'121ae339b9b903aaf4fb7b3903fbb5c1e37246ebe14786c2109a2a1eb29e32bc','mods/fabric-api-0.161.0+26.2.jar':'e5b858ceb13290c274e31cb888ff5a1a40cb91067e7ffab0b5b775aec51e216a'}
    if any(sha(template/name)!=digest for name,digest in checked.items()):raise ValueError('Existing local server inputs differ')
    work.mkdir(parents=True);shutil.copy2(Path(__file__),work/'producer-source.py');classes=work/'classes';classes.mkdir();probe=ROOT/'horizon/tests/java/NativeSpawnCaptureMod.java';inputs=ROOT/'horizon/tests/fixtures/host-density-v1/inputs.json'
    configuration=json.loads(inputs.read_text(encoding='utf-8'))
    cp=str(jar)+os.pathsep+a.classpath_file.read_text(encoding='utf-8').strip();suffix='.exe' if os.name=='nt' else ''
    compiled=subprocess.run([str(a.java_home/'bin'/('javac'+suffix)),'--release','25','-encoding','UTF-8','-proc:none','-cp',cp,'-d',str(classes),str(probe)],capture_output=True,text=True,timeout=90);(work/'compile.log').write_text(compiled.stdout+compiled.stderr,encoding='utf-8')
    if compiled.returncode:raise RuntimeError('Reference compile failed; preserved compile.log')
    qa=work/'spawn-capture-qa.jar'
    with zipfile.ZipFile(qa,'w',compression=zipfile.ZIP_DEFLATED) as archive:
        archive.writestr('fabric.mod.json',json.dumps({'schemaVersion':1,'id':'terraforged_spawn_capture_qa','version':'0.1.0','environment':'server','license':'MIT','entrypoints':{'main':['terraforged.qa.NativeSpawnCaptureMod']},'depends':{'reterraforged':'*','fabricloader':'>=0.20.1','fabric-lifecycle-events-v1':'*','minecraft':'26.2','java':'>=25'}}))
        for file in sorted(classes.rglob('*.class')):archive.write(file,file.relative_to(classes).as_posix())
    runs=[];captures=[]
    for index in range(2):
        instance=work/f'server-{index}';instance.mkdir();mods=instance/'mods';mods.mkdir()
        for directory in ('.fabric','libraries','versions'):shutil.copytree(template/directory,instance/directory)
        for name in ('server.jar','fabric-server-launch.jar','fabric-server-launcher.properties','eula.txt'):shutil.copy2(template/name,instance/name)
        shutil.copy2(template/'mods/fabric-api-0.161.0+26.2.jar',mods/'fabric-api-0.161.0+26.2.jar');installed=mods/jar.name;shutil.copy2(jar,installed);shutil.copy2(qa,mods/qa.name)
        pack=instance/'qa-world/datapacks/spawn-reference';pack.mkdir(parents=True)
        (pack/'pack.mcmeta').write_text(json.dumps({'pack':{'description':'Independent TerraForged spawn reference inputs','min_format':[107,1],'max_format':107}}),encoding='utf-8')
        for name,value in configuration['noiseRegistry'].items():
            namespace,location=name.split(':',1);destination=pack/'data'/namespace/'reterraforged/worldgen/noise'/Path(location+'.json');destination.parent.mkdir(parents=True,exist_ok=True)
            destination.write_text(value if isinstance(value,str) else json.dumps(value),encoding='utf-8')
        with socket.socket() as listener:listener.bind(('127.0.0.1',0));port=listener.getsockname()[1]
        properties={'server-ip':'127.0.0.1','server-port':port,'online-mode':'false','enable-query':'false','enable-rcon':'false','level-name':'qa-world','level-seed':8675309,'level-type':'minecraft:flat','spawn-protection':0,'view-distance':2,'simulation-distance':2,'max-players':1,'pause-when-empty-seconds':-1,'max-tick-time':60000}
        (instance/'server.properties').write_text('\n'.join(f'{key}={value}' for key,value in properties.items())+'\n',encoding='utf-8');env=dict(os.environ)
        for name in ('HORIZON_HOME','HORIZON_LAUNCHER_HOME','XDG_DATA_HOME','XDG_CONFIG_HOME','HORIZON_LAUNCHER_TRASH','HORIZON_LAUNCHER_CLIPBOARD','HORIZON_LAUNCHER_DESKTOP'):
            home=instance/'homes'/name.lower();home.mkdir(parents=True);env[name]=str(home)
        file=work/f'java-{index}.tsv';command=[str(a.java_home/'bin'/('java'+suffix)),'-Xms256M','-Xmx2G','-XX:ActiveProcessorCount=4','-Dfabric.development=false',f'-Duser.home={instance / "homes"}',f'-Dterraforged.qa.inputs={inputs}',f'-Dterraforged.qa.production={installed}',f'-Dterraforged.qa.output={file}','-jar','fabric-server-launch.jar','nogui']
        result=subprocess.run(command,cwd=instance,env=env,capture_output=True,text=True,timeout=300,creationflags=subprocess.CREATE_NO_WINDOW if os.name=='nt' else 0);log=result.stdout+result.stderr;(work/f'java-{index}.log').write_text(log,encoding='utf-8')
        marker=re.search(r'NATIVE_SPAWN_CAPTURE_OK records=(\d+) pid=(\d+)',log);origins=[line.split('SPAWN_PACKAGED_CLASS_ORIGIN=',1)[1] for line in log.splitlines() if 'SPAWN_PACKAGED_CLASS_ORIGIN=' in line]
        if result.returncode or not marker or int(marker[1])!=24 or len(origins)!=3:raise RuntimeError('Reference server capture failed; preserved log')
        captures.append(file.read_bytes());runs.append({'pid':int(marker[2]),'exitCode':result.returncode,'records':24,'outputSha256':sha(file),'logSha256':sha(work/f'java-{index}.log'),'packagedOrigins':origins})
    if captures[0]!=captures[1] or runs[0]['pid']==runs[1]['pid']:raise ValueError('Fresh Java observations differ')
    output.mkdir(parents=True);(output/'values.tsv').write_bytes(captures[0]);shutil.copy2(inputs,output/'inputs.json')
    metadata={'schemaVersion':1,'status':'frozen-packaged-java-spawn-centers','capturedAtUtc':dt.datetime.now(dt.timezone.utc).isoformat(),'productionJarSha256':PRODUCTION,'probeSourceSha256':sha(probe),'producerSourceSha256':sha(work/'producer-source.py'),'inputsSha256':sha(inputs),'valuesSha256':sha(output/'values.tsv'),'localServerInputsSha256':checked,'runs':runs,'freshJavaRunsAgree':True,'gameAssetsDownloaded':False,'nativeExpectedValuesUsed':False,'scope':'Default preset, eight long seeds (including int wrapping), all three packaged SpawnType.getSearchCenter methods. ISLANDS retains source origin fallback; no new island search is claimed.','notCovered':['Actual player placement or safe-spawn height selection','Other presets','Native controlled startup performance']}
    (output/'metadata.json').write_text(json.dumps(metadata,indent=2)+'\n',encoding='utf-8',newline='\n');print(json.dumps({'status':metadata['status'],'recordsPerRun':24}))
if __name__=='__main__':main()
