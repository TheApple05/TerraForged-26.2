"""Freeze independent packaged Java terrain-provider observations; never use native output."""
from __future__ import annotations
import argparse
import hashlib
import gzip
import json
from pathlib import Path

ROOT=Path(__file__).resolve().parents[2]
def sha(data):return hashlib.sha256(data).hexdigest()
SOURCES=['world/worldgen/'+n+'.java' for n in [
    'cell/terrain/provider/TerrainProvider','cell/terrain/populator/TerrainPopulator','cell/terrain/populator/VolcanoPopulator',
    'cell/terrain/region/RegionSelector','cell/terrain/Populators','cell/heightmap/Levels','cell/heightmap/RegionConfig',
    'util/Seed','noise/NoiseUtil','noise/module/Noises','biome/Erosion','biome/Weirdness','biome/BiomeParameter']]+[
    'data/worldgen/preset/PresetNoiseData.java','data/worldgen/preset/settings/TerrainSettings.java']
ORIGINS=['world.worldgen.'+n for n in [
    'cell.terrain.provider.TerrainProvider','cell.terrain.populator.TerrainPopulator','cell.terrain.populator.VolcanoPopulator',
    'cell.terrain.region.RegionSelector','cell.terrain.Populators','cell.heightmap.Levels','cell.heightmap.RegionConfig',
    'util.Seed','noise.module.Noises$HolderHolder','noise.module.Noises','noise.module.Cache2d']]

def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--first',type=Path,required=True);p.add_argument('--second',type=Path,required=True);p.add_argument('--output',type=Path,required=True);args=p.parse_args()
    if args.output.exists():p.error('Refusing to replace terrain-provider reference directory')
    captures=[];producers=[]
    for path in (args.first,args.second):
        raw=path.read_bytes();r=json.loads(raw)
        if r['result']!='passed' or r['processExitCode']!=0 or r['developmentMode'] is not False or not r['packagedArtifactOriginObserved']:raise ValueError('Production terrain-provider oracle did not pass')
        c=r['qaResult']['terrainProvider'];values=Path(c['file']).read_bytes();inputs=Path(c['inputs']).read_bytes()
        if c['status']!='captured-genuine-java-terrain-provider' or sha(values)!=c['sha256'] or sha(inputs)!=c['inputsSha256']:raise ValueError('Populator bytes changed')
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
    if inputs!=(ROOT/'horizon/tests/inputs/terrain-provider-v1.json').read_bytes():raise ValueError('Captured inputs differ from predefined cases')
    rows=dict(line.split('\t',1) for line in values.decode().splitlines())
    if len(rows)!=len(values.splitlines()) or len(rows)!=230292 or sum(k.startswith('node-cell/') for k in rows)!=77682 or sum(k.startswith('selector/') for k in rows)!=49368 or sum(k.startswith('volcano/') for k in rows)!=101400 or sum(k.startswith('modify/') for k in rows)!=1152 or sum(k.startswith('node/') for k in rows)!=642:raise ValueError('Incomplete terrain-provider corpus')
    compressed=gzip.compress(values,mtime=0)
    source_root=ROOT/'common/src/main/java/raccoonman/reterraforged'
    report={'schemaVersion':1,'status':'frozen-genuine-java-terrain-provider-reference','freshRunsAgree':True,
        'productionJarSha256':producers[0]['productionJarSha256'],'fixtureSha256':sha(compressed),'uncompressedValuesSha256':sha(values),'inputsSha256':sha(inputs),
        'recordCount':len(rows),'settingCases':4,'signed32Seeds':6,'cellSamples':229602,'providerNodes':642,'volcanoGridSamples':101400,'allCellFields':24,'producerReports':producers,
        'sourceSha256Lf':{name:sha((source_root/name).read_bytes().replace(b'\r\n',b'\n')) for name in SOURCES},
        'oracleSourceSha256Lf':sha((ROOT/'horizon/tests/java/NativeTerrainProviderProbe.java').read_bytes().replace(b'\r\n',b'\n')),
        'scope':'Actual provider construction/composite/shuffle order, weighted selection and volcano geometry/type mutation; all 24 Cell fields.',
        'nativeExpectedValuesUsed':False,'notCovered':['Arbitrary prior global Java registry history','All preset/configuration domains','River networks/cache','Whole terrain/climate/filtered tiles','Native chunks/client/save']}
    args.output.mkdir(parents=True);(args.output/'values.tsv.gz').write_bytes(compressed);(args.output/'inputs.json').write_bytes(inputs)
    (args.output/'metadata.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8')
    print(json.dumps({k:report[k] for k in ('status','recordCount','cellSamples')}))
if __name__=='__main__':main()
