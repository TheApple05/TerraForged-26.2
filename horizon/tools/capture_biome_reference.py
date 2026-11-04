"""Freeze source-owned biome palette data and genuine Java lookup observations twice."""
from __future__ import annotations
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile
import zipfile

ROOT=Path(__file__).resolve().parents[2]
def sha(data):return hashlib.sha256(data).hexdigest()

def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--java-home',type=Path,required=True);p.add_argument('--jar',type=Path,required=True);p.add_argument('--output',type=Path,required=True);args=p.parse_args()
    if args.output.exists():p.error('Refusing to replace biome reference directory')
    source=ROOT/'horizon/tests/java/NativeBiomeProbe.java';jar=args.jar.resolve()
    cp=str(jar)+os.pathsep+(ROOT/'build/qa/target/compile-classpath.txt').read_text(encoding='utf-8').strip()
    suffix='.exe' if os.name=='nt' else '';workroot=ROOT/'build/qa/horizon/biome-capture';workroot.mkdir(parents=True,exist_ok=True)
    originals={}
    with zipfile.ZipFile(jar) as z:
        for name in ('biomes.png','biomes.txt'):
            packaged=z.read(name);local=(ROOT/'common/src/main/resources'/name).read_bytes()
            if name.endswith('.txt'):packaged=packaged.replace(b'\r\n',b'\n');local=local.replace(b'\r\n',b'\n')
            if packaged!=local:raise ValueError('Source-owned palette resource differs from actual package: '+name)
            originals[name]=sha(packaged)
    snapshots=[]
    with tempfile.TemporaryDirectory(dir=workroot) as temporary:
        work=Path(temporary)
        compile=subprocess.run([str(args.java_home/'bin'/('javac'+suffix)),'--release','25','-proc:none','-cp',cp,'-d',str(work),str(source)],cwd=ROOT,capture_output=True,text=True,timeout=90)
        if compile.returncode:raise RuntimeError(compile.stderr)
        for i in range(2):
            data=work/f'{i}.bin';values=work/f'{i}.tsv';log=work/f'{i}.log'
            run=subprocess.run([str(args.java_home/'bin'/('java'+suffix)),'-Xlog:class+load=info:file='+log.relative_to(ROOT).as_posix(),'-cp',str(work)+os.pathsep+cp,'NativeBiomeProbe',str(data),str(values)],cwd=ROOT,capture_output=True,text=True,timeout=90)
            if run.returncode:raise RuntimeError(run.stderr)
            lines=log.read_text(encoding='utf-8').splitlines()
            for name in ['cell.biome.type.BiomeType','cell.biome.type.BiomeTypeLoader','biome.Temperature','biome.Humidity','noise.NoiseUtil']:
                matches=[line for line in lines if 'raccoonman.reterraforged.world.worldgen.'+name+' source:' in line]
                if len(matches)!=1 or jar.name not in matches[0]:raise ValueError('Genuine biome source origin missing: '+name)
            snapshots.append((data.read_bytes(),values.read_bytes()))
    if snapshots[0]!=snapshots[1]:raise ValueError('Fresh Java biome observations differ')
    data,values=snapshots[0]
    if len(data)!=65536 or max(data)>10:raise ValueError('Unsupported palette layout')
    sources=['cell/biome/type/BiomeType.java','cell/biome/type/BiomeTypeLoader.java','biome/Temperature.java','biome/Humidity.java','biome/BiomeParameter.java','noise/NoiseUtil.java']
    result={'schemaVersion':1,'status':'frozen-genuine-java-biome-reference','freshRunsAgree':True,'productionJarSha256':sha(jar.read_bytes()),
        'tableSha256':sha(data),'fixtureSha256':sha(values),'recordCount':len(values.splitlines()),'sourceOwnedResourcesSha256':originals,
        'sourceSha256Lf':{name:sha((ROOT/'common/src/main/java/raccoonman/reterraforged/world/worldgen'/name).read_bytes().replace(b'\r\n',b'\n')) for name in sources},
        'oracleSourceSha256Lf':sha(source.read_bytes().replace(b'\r\n',b'\n')),
        'scope':'Finite source-owned climate category palette and Java curve/linear lookup, normalization and identity pair observations; not terrain snapshots or registered biome IDs.',
        'captureJava':subprocess.check_output([str(args.java_home/'bin'/('java'+suffix)),'-version'],stderr=subprocess.STDOUT,text=True),
        'notCovered':['Continent/river/climate cell generation','17 classic registered biome IDs','Native chunks/client/save activation']}
    args.output.mkdir(parents=True);(args.output/'table.bin').write_bytes(data);(args.output/'values.tsv').write_bytes(values)
    (args.output/'metadata.json').write_text(json.dumps(result,indent=2)+'\n',encoding='utf-8')
    print(json.dumps({k:result[k] for k in ('status','recordCount','tableSha256')}))

if __name__=='__main__':main()
