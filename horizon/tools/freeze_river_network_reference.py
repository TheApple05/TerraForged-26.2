"""Freeze independent genuine Java network and complete unfiltered cell captures."""
from __future__ import annotations
import argparse,gzip,hashlib,json
from collections import Counter
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
def sha(data):return hashlib.sha256(data).hexdigest()
NAMES=['cell/continent/simple/SimpleRiverGenerator','cell/continent/advanced/AdvancedContinentGenerator',
       'cell/rivermap/river/BaseRiverGenerator','cell/rivermap/river/Network','cell/rivermap/river/RiverCarver',
       'cell/rivermap/river/River','cell/rivermap/river/RiverWarp','cell/rivermap/river/RiverConfig',
       'cell/rivermap/lake/Lake','cell/rivermap/wetland/Wetland','cell/rivermap/gen/GenWarp',
       'cell/rivermap/Rivermap','cell/rivermap/RiverCache','cell/heightmap/Heightmap','cell/climate/Climate']
def main():
    p=argparse.ArgumentParser(description=__doc__)
    for n in ('first','second','output'):p.add_argument('--'+n,type=Path,required=True)
    args=p.parse_args()
    if args.output.exists():p.error('Refusing to replace network reference directory')
    captures=[];producers=[]
    for path in (args.first,args.second):
        raw=path.read_bytes();r=json.loads(raw)
        if r['result']!='passed' or r['processExitCode']!=0 or r['developmentMode'] is not False or not r['packagedArtifactOriginObserved']:raise ValueError('Production network oracle did not pass')
        c=r['qaResult']['riverNetworks'];values=Path(c['file']).read_bytes();inputs=Path(c['inputs']).read_bytes()
        if c['status']!='captured-genuine-java-river-networks' or sha(values)!=c['sha256'] or sha(inputs)!=c['inputsSha256']:raise ValueError('Network capture bytes changed')
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
    if inputs!=(ROOT/'horizon/tests/inputs/river-networks-v1.json').read_bytes():raise ValueError('Captured inputs differ from predefined cases')
    rows=dict(line.split('\t',1) for line in values.decode().splitlines());counts=Counter(k.split('/')[0] for k in rows)
    expected={'map':62424,'full':3888,'node':1920,'river':1920,'bounds':1920,'config':1920,'warp':1920,'curve':1920,'wetland':562,'lake':434,'map-info':72,'rng':72}
    if len(rows)!=len(values.splitlines()) or counts!=expected or sum(len(v.split(',')) for v in rows.values())!=1687920:raise ValueError('Incomplete network corpus')
    compressed=gzip.compress(values,mtime=0);source_root=ROOT/'common/src/main/java/raccoonman/reterraforged/world/worldgen'
    report={'schemaVersion':1,'status':'frozen-genuine-java-river-network-reference','freshRunsAgree':True,
        'productionJarSha256':producers[0]['productionJarSha256'],'fixtureSha256':sha(compressed),'uncompressedValuesSha256':sha(values),'inputsSha256':sha(inputs),
        'recordCount':len(rows),'fieldsPerProfile':1687920,'recordKinds':counts,'producerReports':producers,
        'sourceSha256Lf':{n+'.java':sha((source_root/(n+'.java')).read_bytes().replace(b'\r\n',b'\n')) for n in NAMES},
        'oracleSourceSha256Lf':sha((ROOT/'horizon/tests/java/NativeRiverNetworkProbe.java').read_bytes().replace(b'\r\n',b'\n')),
        'scope':'Original cached river maps and construction tree/configuration/warp/curve/bounds, original staged random checkpoints, synthetic map carving and complete terrain-river-climate cells.',
        'nativeExpectedValuesUsed':False,'notCovered':['Every numeric/configuration domain','Java expiring-cache scheduling behavior','Filtered tiles/erosion/smoothing','Native density/chunks/client/save']}
    args.output.mkdir(parents=True);(args.output/'values.tsv.gz').write_bytes(compressed);(args.output/'inputs.json').write_bytes(inputs)
    (args.output/'metadata.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8');print(json.dumps({k:report[k] for k in ('status','recordCount','fieldsPerProfile')}))
if __name__=='__main__':main()
