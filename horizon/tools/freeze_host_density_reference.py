"""Freeze independently captured production Java density function observations."""
from __future__ import annotations
import argparse
from collections import Counter
import gzip
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
CLASSES = ["densityfunction/CellSampler", "densityfunction/CellSampler$Field",
           "densityfunction/ClampToNearestUnit", "densityfunction/NoiseFunction",
           "densityfunction/tile/generation/TileGenerator", "cell/heightmap/WorldLookup",
           "GeneratorContext"] + ["densityfunction/CellSampler$Field$"+str(i) for i in range(1,11)]

def sha(data): return hashlib.sha256(data).hexdigest()

def main():
    p=argparse.ArgumentParser(description=__doc__)
    for n in ("first","second","output"): p.add_argument("--"+n, type=Path, required=True)
    a=p.parse_args()
    if a.output.exists(): p.error("Choose a new immutable reference directory")
    captures=[]; producers=[]
    for path in (a.first,a.second):
        raw=path.read_bytes();r=json.loads(raw)
        if r['result']!='passed' or r['processExitCode']!=0 or r['developmentMode'] is not False or not r['packagedArtifactOriginObserved']:
            raise ValueError('Production Java oracle did not pass')
        c=r['qaResult']['hostDensity'];values=Path(c['file']).read_bytes();inputs=Path(c['inputs']).read_bytes()
        if c['status']!='captured-genuine-java-host-density' or sha(values)!=c['sha256'] or sha(inputs)!=c['inputsSha256']:
            raise ValueError('Capture bytes changed')
        log=Path(r['classLoadLog'])
        if not log.is_absolute(): log=path.resolve().parents[2]/log
        log_bytes=log.read_bytes();lines=log_bytes.decode().splitlines();origins=[]
        for name in CLASSES:
            matches=[line for line in lines if 'raccoonman.reterraforged.world.worldgen.'+name.replace('/','.')+' source:' in line]
            if len(matches)!=1 or r['artifact']['name'] not in matches[0]:raise ValueError('Original class origin absent: '+name)
            origins.append(matches[0])
        captures.append((values,inputs));producers.append({'report':path.name,'sha256':sha(raw),'pid':r['pid'],
            'instance':r['instance'],'productionJarSha256':r['artifact']['sha256'],
            'classLoadLogSha256':sha(log_bytes),'verifiedOriginalClassOrigins':origins})
    if captures[0]!=captures[1] or producers[0]['pid']==producers[1]['pid'] or producers[0]['productionJarSha256']!=producers[1]['productionJarSha256']:
        raise ValueError('Independent captures disagree')
    values,inputs=captures[0]
    if inputs!=(ROOT/'horizon/tests/inputs/host-density-v1.json').read_bytes():raise ValueError('Configuration differs from predefined inputs')
    rows=dict(line.split('\t',1) for line in values.decode().splitlines());kinds=Counter(k.split('/')[0] for k in rows)
    if len(rows)!=len(values.splitlines()) or kinds!={'scalar':10368,'volume':14400}:raise ValueError('Incomplete scalar/bulk capture')
    compressed=gzip.compress(values,mtime=0)
    report={'schemaVersion':1,'status':'frozen-genuine-java-host-density-reference','freshRunsAgree':True,
        'recordCount':len(rows),'recordKinds':kinds,'fixtureSha256':sha(compressed),
        'uncompressedValuesSha256':sha(values),'inputsSha256':sha(inputs),'producerReports':producers,
        'oracleSourceSha256Lf':sha((ROOT/'horizon/tests/java/NativeHostDensityProbe.java').read_bytes().replace(b'\r\n',b'\n')),
        'nativeOutputsRead':False,'scope':'Actual mapped cell densities over filtered tiles, original clamp and noise wrappers, one addition graph; scalar and bulk results narrowed to the Horizon f32 interface.',
        'notCovered':['Full density router conversion','Original f64 vanilla chunk execution','Every preset or numeric domain','Native chunks/client/save/packaging']}
    a.output.mkdir(parents=True)
    (a.output/'values.tsv.gz').write_bytes(compressed);(a.output/'inputs.json').write_bytes(inputs)
    (a.output/'metadata.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8',newline='\n')
    print(json.dumps({k:report[k] for k in ('status','recordCount','recordKinds')}))

if __name__=='__main__':main()
