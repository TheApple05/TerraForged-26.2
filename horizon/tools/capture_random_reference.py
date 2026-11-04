"""Capture actual Java 25 Random/Collections output in two independent JVMs."""
from __future__ import annotations
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import tempfile

ROOT=Path(__file__).resolve().parents[2]
def sha(data):return hashlib.sha256(data).hexdigest()
def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--java-home',type=Path,required=True);p.add_argument('--output',type=Path,required=True);args=p.parse_args()
    if args.output.exists():p.error('Refusing to replace Java Random observations')
    source=ROOT/'horizon/tests/java/NativeJavaRandomProbe.java';captures=[];origins=[]
    workroot=ROOT/'build/qa/horizon/random-reference';workroot.mkdir(parents=True,exist_ok=True)
    with tempfile.TemporaryDirectory(dir=workroot) as temporary:
        java=args.java_home/'bin/java';javac=args.java_home/'bin/javac'
        compile=subprocess.run([str(javac),'--release','25','-encoding','UTF-8','-d',temporary,str(source)],capture_output=True,text=True,timeout=30)
        if compile.returncode:raise RuntimeError(compile.stderr)
        for index in range(2):
            log=Path(temporary)/f'classes-{index}.log'
            command=[str(java),'-Xlog:class+load=info:file='+log.relative_to(ROOT).as_posix(),'-cp',temporary,'NativeJavaRandomProbe']
            result=subprocess.run(command,capture_output=True,cwd=ROOT,timeout=30)
            if result.returncode:raise RuntimeError(result.stderr.decode('utf-8',errors='replace'))
            lines=log.read_text(encoding='utf-8').splitlines();observed=[]
            for name in ['java.util.Random','java.util.Collections']:
                matches=[line for line in lines if name+' source:' in line]
                if len(matches)!=1 or not any(origin in matches[0] for origin in ['jrt:/java.base','shared objects file']):raise ValueError('Actual JDK class origin absent: '+name)
                # Timestamps are process-specific, retain only class and origin.
                observed.append(matches[0][matches[0].index(name):])
            captures.append(result.stdout.replace(b'\r\n',b'\n'));origins.append(observed)
    if captures[0]!=captures[1]:raise ValueError('Independent JDK Random runs differ')
    values=captures[0];keys=[line.split(b'\t',1)[0] for line in values.splitlines()]
    if len(keys)!=2718 or len(set(keys))!=len(keys):raise ValueError('Random corpus incomplete/duplicate')
    metadata={'schemaVersion':1,'status':'frozen-java-random-reference','freshRunsAgree':True,'fixtureSha256':sha(values),
        'recordCount':len(keys),'oracleSourceSha256Lf':sha(source.read_bytes().replace(b'\r\n',b'\n')),'verifiedJdkClassOrigins':origins,
        'captureJava':subprocess.check_output([str(java),'-version'],stderr=subprocess.STDOUT,text=True),
        'nativeExpectedValuesUsed':False,'scope':'Nine signed64 seeds, four used Random methods, eight bounded domains, resets, Collections.shuffle and post-shuffle draw checkpoints.',
        'algorithmSpecification':'https://docs.oracle.com/en/java/javase/25/docs/api/java.base/java/util/Random.html',
        'notCovered':['Gaussian/double/long/byte APIs','Thread-shared mutable Random','Complete river draw order or terrain-provider construction']}
    args.output.mkdir(parents=True);(args.output/'values.tsv').write_bytes(values)
    (args.output/'metadata.json').write_text(json.dumps(metadata,indent=2)+'\n',encoding='utf-8')
    print(json.dumps({'status':metadata['status'],'recordCount':len(keys)}))
if __name__=='__main__':main()
