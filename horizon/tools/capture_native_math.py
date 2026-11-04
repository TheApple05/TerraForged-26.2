"""Freeze genuine Java NoiseUtil lookup constants and trig/sqrt values twice."""
from __future__ import annotations
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[2]


def sha(path): return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--java-home',type=Path,required=True)
    parser.add_argument('--jar',type=Path,required=True)
    parser.add_argument('--output',type=Path,required=True)
    args = parser.parse_args()
    if args.output.exists(): parser.error('Refusing to replace a math reference directory')
    source = ROOT/'horizon/tests/java/NativeMathProbe.java'
    workroot = ROOT/'build/qa/horizon/math-capture'
    workroot.mkdir(parents=True,exist_ok=True)
    extension = '.exe' if os.name=='nt' else ''
    snapshots = []
    with tempfile.TemporaryDirectory(dir=workroot) as temporary:
        work = Path(temporary)
        compile = subprocess.run([str(args.java_home/'bin'/('javac'+extension)),'--release','25','-proc:none',
            '-cp',str(args.jar.resolve()),'-d',str(work),str(source)],cwd=ROOT,capture_output=True,text=True,timeout=90)
        if compile.returncode: raise RuntimeError(compile.stdout+compile.stderr)
        for index in range(2):
            table,values = work/f'{index}.bin',work/f'{index}.tsv'
            run = subprocess.run([str(args.java_home/'bin'/('java'+extension)),'-cp',str(work)+os.pathsep+str(args.jar.resolve()),
                'NativeMathProbe',str(table),str(values)],cwd=ROOT,capture_output=True,text=True,timeout=90)
            if run.returncode: raise RuntimeError(run.stderr)
            properties = dict(line.split('=',1) for line in run.stdout.splitlines())
            if args.jar.name not in properties['ORIGIN']: raise ValueError('Original NoiseUtil package origin missing')
            snapshots.append((table.read_bytes(),values.read_bytes(),properties))
    if snapshots[0]!=snapshots[1]: raise ValueError('Independent Java math captures differ')
    binary,values,properties = snapshots[0]
    count = int(properties['SIN_COUNT'])
    if len(binary)!=count*4: raise ValueError('Sine table byte count differs')
    args.output.mkdir(parents=True)
    (args.output/'sine.bin').write_bytes(binary)
    (args.output/'values.tsv').write_bytes(values)
    noise = ROOT/'common/src/main/java/raccoonman/reterraforged/world/worldgen/noise/NoiseUtil.java'
    record = {'schemaVersion':1,'status':'frozen-independent-java-math-reference','freshRunsAgree':True,
        'productionJarSha256':sha(args.jar),'noiseUtilSourceSha256Lf':hashlib.sha256(noise.read_bytes().replace(b'\r\n',b'\n')).hexdigest(),
        'oracleSourceSha256Lf':hashlib.sha256(source.read_bytes().replace(b'\r\n',b'\n')).hexdigest(),
        'sineEntries':count,'sineTableSha256':sha(args.output/'sine.bin'),'valuesSha256':sha(args.output/'values.tsv'),
        'recordCount':len(values.splitlines()),'properties':properties,
        'scope':'Exact source-owned sine lookup constants and trig/sqrt observations; no terrain/game assets or native expected values.',
        'notCovered':['Every float/nonfinite input','Fractional power','Terrain or native activation']}
    (args.output/'metadata.json').write_text(json.dumps(record,indent=2)+'\n',encoding='utf-8')
    print(json.dumps(record))


if __name__=='__main__': main()
