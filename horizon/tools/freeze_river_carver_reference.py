"""Freeze two independent genuine packaged Java carver captures; no native values."""
from __future__ import annotations
import argparse, gzip, hashlib, json
from collections import Counter
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
def sha(data): return hashlib.sha256(data).hexdigest()
NAMES=['cell/rivermap/river/RiverCarver','cell/rivermap/river/River','cell/rivermap/river/RiverWarp',
       'cell/rivermap/river/RiverConfig','cell/rivermap/lake/Lake','cell/rivermap/lake/LakeConfig',
       'cell/rivermap/wetland/Wetland','cell/heightmap/Levels','util/Boundsf','util/Variance',
       'noise/function/CurveFunctions','noise/NoiseUtil']
def main():
    p=argparse.ArgumentParser(description=__doc__)
    for n in ('first','second','output'): p.add_argument('--'+n,type=Path,required=True)
    args=p.parse_args()
    if args.output.exists():p.error('Refusing to replace river-carver reference directory')
    captures=[];producers=[]
    for path in (args.first,args.second):
        raw=path.read_bytes();r=json.loads(raw)
        if r['result']!='passed' or r['processExitCode']!=0 or r['developmentMode'] is not False or not r['packagedArtifactOriginObserved']:raise ValueError('Production carver oracle did not pass')
        c=r['qaResult']['riverCarvers'];values=Path(c['file']).read_bytes();inputs=Path(c['inputs']).read_bytes()
        if c['status']!='captured-genuine-java-river-carvers' or sha(values)!=c['sha256'] or sha(inputs)!=c['inputsSha256']:raise ValueError('Captured carver bytes changed')
        log=Path(r['classLoadLog'])
        if not log.is_absolute():log=path.resolve().parents[2]/log
        raw_log=log.read_bytes();lines=raw_log.decode().splitlines();origins=[]
        for name in NAMES:
            matches=[line for line in lines if 'raccoonman.reterraforged.world.worldgen.'+name.replace('/','.')+' source:' in line]
            if len(matches)!=1 or r['artifact']['name'] not in matches[0]:raise ValueError('Original class absent: '+name)
            origins.append(matches[0])
        captures.append((values,inputs));producers.append({'report':path.name,'sha256':sha(raw),'pid':r['pid'],'instance':r['instance'],
            'productionJarSha256':r['artifact']['sha256'],'classLoadLogSha256':sha(raw_log),'verifiedOriginalClassOrigins':origins})
    if captures[0]!=captures[1] or producers[0]['pid']==producers[1]['pid'] or producers[0]['productionJarSha256']!=producers[1]['productionJarSha256']:raise ValueError('Independent captures differ')
    values,inputs=captures[0]
    if inputs!=(ROOT/'horizon/tests/inputs/river-carvers-v1.json').read_bytes():raise ValueError('Captured inputs differ from predefined cases')
    rows=dict(line.split('\t',1) for line in values.decode().splitlines())
    counts=Counter(k.split('/')[0] for k in rows)
    expected={'carve':48600,'lake':34992,'wetland':78732,'fork':810,'lake-geometry':5832,'lake-bounds':72,'wetland-bounds':162,
              'valley-rng':384,'create-rng':18,'round-double':24,'bounds-empty':1,'bounds-none':1,'bounds-negative':1}
    if len(rows)!=len(values.splitlines()) or counts!=expected or sum(len(v.split(',')) for v in rows.values())!=3919986:raise ValueError('Incomplete carver corpus')
    compressed=gzip.compress(values,mtime=0);source_root=ROOT/'common/src/main/java/raccoonman/reterraforged/world/worldgen'
    report={'schemaVersion':1,'status':'frozen-genuine-java-river-carvers-reference','freshRunsAgree':True,
        'productionJarSha256':producers[0]['productionJarSha256'],'fixtureSha256':sha(compressed),'uncompressedValuesSha256':sha(values),'inputsSha256':sha(inputs),
        'recordCount':len(rows),'fieldsPerProfile':3919986,'recordKinds':counts,'producerReports':producers,
        'sourceSha256Lf':{n+'.java':sha((source_root/(n+'.java')).read_bytes().replace(b'\r\n',b'\n')) for n in NAMES},
        'oracleSourceSha256Lf':sha((ROOT/'horizon/tests/java/NativeRiverCarverProbe.java').read_bytes().replace(b'\r\n',b'\n')),
        'scope':'Original river/lake/wetland cell mutations, bounds, geometry predicates, fork widths, valley curves and RNG checkpoints, double rounding. Finite raw bits and explicit NaN classifications.',
        'nativeExpectedValuesUsed':False,'notCovered':['Every numeric/configuration domain','River network construction/cache','Filtered tiles/erosion/smoothing','Native chunks/client/save']}
    args.output.mkdir(parents=True);(args.output/'values.tsv.gz').write_bytes(compressed);(args.output/'inputs.json').write_bytes(inputs)
    (args.output/'metadata.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8')
    print(json.dumps({k:report[k] for k in ('status','recordCount','fieldsPerProfile')}))
if __name__=='__main__':main()
