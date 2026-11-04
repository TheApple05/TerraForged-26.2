"""Freeze two fresh observations of the exact published Java biome policy."""
from __future__ import annotations
import argparse, datetime as dt, gzip, hashlib, json, os, re, subprocess
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
PRODUCTION='1c9e5e7e8d286c145a647f466b2f908cda721d10672c40949864f1015570e886'
def sha(data):return hashlib.sha256(data).hexdigest()
def main():
    p=argparse.ArgumentParser(description=__doc__)
    for n in ('java-home','production-jar','classpath-file','work','output'):p.add_argument('--'+n,type=Path,required=True)
    a=p.parse_args()
    if a.work.exists() or a.output.exists():p.error('Choose fresh work and fixture directories; no replacement')
    if not a.work.resolve().is_relative_to((ROOT/'build/qa/horizon').resolve()):p.error('QA work must remain below build/qa/horizon')
    jar=a.production_jar.resolve();digest=sha(jar.read_bytes())
    if digest!=PRODUCTION:raise ValueError('The independently qualified dev23 production artifact is required')
    a.work.mkdir(parents=True);classes=a.work/'classes';classes.mkdir()
    inputs=ROOT/'horizon/tests/inputs/classic-biomes-v1.json';probe=ROOT/'horizon/tests/java/NativeClassicBiomeProbe.java'
    cp=str(jar)+os.pathsep+a.classpath_file.read_text(encoding='utf-8').strip()
    suffix='.exe' if os.name=='nt' else ''
    cmd=[str(a.java_home/'bin'/('javac'+suffix)),'--release','25','-encoding','UTF-8','-proc:none','-cp',cp,'-d',str(classes),str(probe)]
    compile_run=subprocess.run(cmd,capture_output=True,text=True,timeout=90)
    (a.work/'compile.log').write_text(compile_run.stdout+compile_run.stderr,encoding='utf-8')
    if compile_run.returncode:raise RuntimeError('Java oracle compile failed; see '+str(a.work/'compile.log'))
    captures=[];producers=[]
    for index in (0,1):
        values=a.work/f'run-{index}.tsv'
        run=subprocess.run([str(a.java_home/'bin'/('java'+suffix)),'-cp',str(classes)+os.pathsep+cp,'NativeClassicBiomeProbe',str(inputs.resolve()),str(jar),str(values.resolve())],capture_output=True,text=True,timeout=120)
        log=run.stdout+run.stderr;(a.work/f'run-{index}.log').write_text(log,encoding='utf-8')
        matched=re.search(r'CLASSIC_BIOME_CAPTURE_OK records=(\d+) variants=17 pid=(\d+)',log)
        # Minecraft Bootstrap redirects System.err through its logger, which
        # may prefix these markers and send them to stdout.
        origins=['PACKAGED_CLASS_ORIGIN='+line.split('PACKAGED_CLASS_ORIGIN=',1)[1] for line in log.splitlines() if 'PACKAGED_CLASS_ORIGIN=' in line]
        if run.returncode or not matched or len(origins)!=3:raise RuntimeError('Java oracle failed: '+log[-3000:])
        raw=values.read_bytes();rows=dict(line.split('\t',1) for line in raw.decode().splitlines())
        if len(rows)!=int(matched[1]) or len(rows)!=len(raw.splitlines()):raise ValueError('Incomplete or duplicate Java records')
        captures.append(raw);producers.append({'pid':int(matched[2]),'exitCode':run.returncode,'records':len(rows),'observationsSha256':sha(raw),'logSha256':sha(log.encode()),'verifiedPackagedOrigins':origins})
    if captures[0]!=captures[1] or producers[0]['pid']==producers[1]['pid']:raise ValueError('Fresh independent Java runs differ')
    config=json.loads(inputs.read_text(encoding='utf-8'))
    count=len(config['hashBits'])+len(config['parents'])*len(config['terrains'])*len(config['regionBits'])*len(config['temperatureBits'])
    if count!=producers[0]['records']:raise ValueError('Missing predefined cases')
    compressed=gzip.compress(captures[0],mtime=0)
    metadata={'schemaVersion':1,'status':'frozen-published-java-classic-biome-policy','freshRunsAgree':True,'capturedAtUtc':dt.datetime.now(dt.timezone.utc).isoformat(),
        'productionJarSha256':digest,'recordCount':count,'fixtureSha256':sha(compressed),'uncompressedValuesSha256':sha(captures[0]),'inputsSha256':sha(inputs.read_bytes()),
        'oracleSourceSha256Lf':sha(probe.read_bytes().replace(b'\r\n',b'\n')),'producerReports':producers,
        'scope':'Deliberate dev23 modern biome selection and raw-float region hash. Eligible/ineligible families, temperature/hash thresholds, nonfinite values and inherited terrain identities.',
        'nativeExpectedValuesUsed':False,'notCovered':['Native biome registry/feature implementation','Natural native generation','Native wrapper cave passthrough','Full old Engine parity','Client/save/launcher/modpack acceptance']}
    a.output.mkdir(parents=True);(a.output/'inputs.json').write_bytes(inputs.read_bytes());(a.output/'values.tsv.gz').write_bytes(compressed)
    (a.output/'metadata.json').write_text(json.dumps(metadata,indent=2)+'\n',encoding='utf-8',newline='\n')
    print(json.dumps({'status':metadata['status'],'recordCount':count,'freshRunsAgree':True}))
if __name__=='__main__':main()
