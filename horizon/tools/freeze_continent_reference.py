"""Freeze independent packaged Java continent observations; never use native output."""
from __future__ import annotations
import argparse
import hashlib
import json
from pathlib import Path

ROOT=Path(__file__).resolve().parents[2]
def sha(data):return hashlib.sha256(data).hexdigest()
SOURCES=['world/worldgen/cell/continent/advanced/'+n+'.java' for n in ['AbstractContinent','AdvancedContinentGenerator']]+[
    'world/worldgen/cell/continent/simple/SimpleRiverGenerator.java','world/worldgen/cell/rivermap/river/BaseRiverGenerator.java',
    'world/worldgen/cell/heightmap/ControlPoints.java','world/worldgen/cell/Cell.java','world/worldgen/util/Seed.java',
    'world/worldgen/noise/NoiseUtil.java','world/worldgen/noise/module/Noises.java','world/worldgen/noise/domain/DomainWarp.java']
ORIGINS=['world.worldgen.cell.continent.advanced.'+n for n in ['AbstractContinent','AdvancedContinentGenerator']]+[
    'world.worldgen.GeneratorContext','world.worldgen.cell.continent.simple.SimpleRiverGenerator',
    'world.worldgen.cell.rivermap.river.BaseRiverGenerator','world.worldgen.cell.Cell','world.worldgen.util.Seed',
    'world.worldgen.noise.module.Perlin2','world.worldgen.noise.module.Simplex','world.worldgen.noise.module.Simplex2']

def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--first',type=Path,required=True);p.add_argument('--second',type=Path,required=True);p.add_argument('--output',type=Path,required=True);args=p.parse_args()
    if args.output.exists():p.error('Refusing to replace continent reference directory')
    captures=[];producers=[]
    for path in (args.first,args.second):
        raw=path.read_bytes();r=json.loads(raw)
        if r['result']!='passed' or r['processExitCode']!=0 or r['developmentMode'] is not False or not r['packagedArtifactOriginObserved']:raise ValueError('Production continent oracle did not pass')
        c=r['qaResult']['continent'];values=Path(c['file']).read_bytes();inputs=Path(c['inputs']).read_bytes()
        if c['status']!='captured-genuine-java-continent' or sha(values)!=c['sha256'] or sha(inputs)!=c['inputsSha256']:raise ValueError('Continent bytes changed')
        log=Path(r['classLoadLog'])
        if not log.is_absolute():log=path.resolve().parents[2]/log
        raw_log=log.read_bytes();lines=raw_log.decode().splitlines();origins=[]
        for name in ORIGINS:
            matches=[line for line in lines if 'raccoonman.reterraforged.'+name+' source:' in line]
            if len(matches)!=1 or r['artifact']['name'] not in matches[0]:raise ValueError('Original class absent: '+name)
            origins.append(matches[0])
        captures.append((values,inputs));producers.append({'report':path.name,'sha256':sha(raw),'pid':r['pid'],'instance':r['instance'],
            'productionJarSha256':r['artifact']['sha256'],'classLoadLogSha256':sha(raw_log),'verifiedOriginalClassOrigins':origins})
    if captures[0]!=captures[1] or producers[0]['pid']==producers[1]['pid'] or producers[0]['productionJarSha256']!=producers[1]['productionJarSha256']:raise ValueError('Independent captures differ')
    values,inputs=captures[0]
    if inputs!=(ROOT/'horizon/tests/inputs/continent-v1.json').read_bytes():raise ValueError('Captured inputs differ from predefined cases')
    rows=dict(line.split('\t',1) for line in values.decode().splitlines())
    if len(rows)!=len(values.splitlines()) or len(rows)!=26113 or sum(k.startswith('cell/') for k in rows)!=25392 or len(json.loads(inputs))!=4:raise ValueError('Incomplete continent corpus')
    source_root=ROOT/'common/src/main/java/raccoonman/reterraforged'
    report={'schemaVersion':1,'status':'frozen-genuine-java-continent-reference','freshRunsAgree':True,
        'productionJarSha256':producers[0]['productionJarSha256'],'fixtureSha256':sha(values),'inputsSha256':sha(inputs),
        'recordCount':len(rows),'presetCases':4,'signed32Seeds':6,'cellSamples':25392,'allCellFields':24,'producerReports':producers,
        'sourceSha256Lf':{name:sha((source_root/name).read_bytes().replace(b'\r\n',b'\n')) for name in SOURCES},
        'oracleSourceSha256Lf':sha((ROOT/'horizon/tests/java/NativeContinentProbe.java').read_bytes().replace(b'\r\n',b'\n')),
        'scope':'Original advanced continent, constructor seed cursor, default/dirty cells, corrected centers, edge/ocean searches and geometry helpers.',
        'nativeExpectedValuesUsed':False,'notCovered':['Other continent types','River networks/cache','Whole terrain/climate/filtered tiles','Native chunks/client/save']}
    args.output.mkdir(parents=True);(args.output/'values.tsv').write_bytes(values);(args.output/'inputs.json').write_bytes(inputs)
    (args.output/'metadata.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8')
    print(json.dumps({k:report[k] for k in ('status','recordCount','cellSamples')}))
if __name__=='__main__':main()
