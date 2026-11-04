"""Freeze real Java preset fields/helpers and factory output from two game JVMs."""
from __future__ import annotations
import argparse
import hashlib
import json
from pathlib import Path

ROOT=Path(__file__).resolve().parents[2]
def sha(data):return hashlib.sha256(data).hexdigest()
SOURCES=['data/worldgen/preset/settings/'+n+'.java' for n in ['Preset','WorldSettings','TerrainSettings','ClimateSettings','RiverSettings','FilterSettings','Presets','ContinentType','SpawnType']]+[
    'world/worldgen/cell/heightmap/ControlPoints.java','world/worldgen/cell/continent/MushroomIslandPopulator.java',
    'world/worldgen/noise/NoiseUtil.java','world/worldgen/noise/module/Noises.java']
ORIGINS=['data.worldgen.preset.settings.'+n for n in ['Preset','WorldSettings','TerrainSettings','ClimateSettings','RiverSettings','FilterSettings']]+[
    'world.worldgen.cell.heightmap.ControlPoints','world.worldgen.noise.module.Noises','world.worldgen.noise.module.Perlin','world.worldgen.noise.module.Perlin2',
    'world.worldgen.noise.module.Simplex','world.worldgen.noise.module.Simplex2','world.worldgen.noise.module.PerlinRidge','world.worldgen.noise.module.SimplexRidge','world.worldgen.noise.module.Billow','world.worldgen.noise.module.Cubic']
def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--first',type=Path,required=True);p.add_argument('--second',type=Path,required=True);p.add_argument('--output',type=Path,required=True);args=p.parse_args()
    if args.output.exists():p.error('Refusing to replace preset reference directory')
    captures=[];producers=[]
    for path in (args.first,args.second):
        raw=path.read_bytes();r=json.loads(raw)
        if r['result']!='passed' or r['processExitCode']!=0 or r['developmentMode'] is not False or not r['packagedArtifactOriginObserved']:raise ValueError('Production preset oracle did not pass')
        c=r['qaResult']['presetModel'];values=Path(c['file']).read_bytes();inputs=Path(c['inputs']).read_bytes()
        if c['status']!='captured-genuine-java-preset-factories' or sha(values)!=c['sha256'] or sha(inputs)!=c['inputsSha256']:raise ValueError('Preset capture bytes changed')
        log=Path(r['classLoadLog'])
        if not log.is_absolute():log=path.resolve().parents[2]/log
        raw_log=log.read_bytes();lines=raw_log.decode().splitlines();origins=[]
        for name in ORIGINS:
            matches=[line for line in lines if 'raccoonman.reterraforged.'+name+' source:' in line]
            if len(matches)!=1 or r['artifact']['name'] not in matches[0]:raise ValueError('Original preset/factory class absent: '+name)
            origins.append(matches[0])
        captures.append((values,inputs));producers.append({'report':path.name,'sha256':sha(raw),'pid':r['pid'],'instance':r['instance'],'productionJarSha256':r['artifact']['sha256'],'classLoadLogSha256':sha(raw_log),'verifiedOriginalClassOrigins':origins})
    if captures[0]!=captures[1] or producers[0]['pid']==producers[1]['pid'] or producers[0]['productionJarSha256']!=producers[1]['productionJarSha256']:raise ValueError('Independent preset captures differ')
    values,inputs=captures[0]
    if inputs!=(ROOT/'horizon/tests/inputs/preset-model-v1.json').read_bytes():raise ValueError('Captured inputs differ from predefined cases')
    rows=dict(line.split('\t',1) for line in values.decode().splitlines())
    if len(rows)!=len(values.splitlines()) or len(rows)!=85630 or sum(k.startswith('graph/') for k in rows)!=49 or len(json.loads(inputs))!=5:raise ValueError('Preset corpus incomplete')
    source_root=ROOT/'common/src/main/java/raccoonman/reterraforged'
    report={'schemaVersion':1,'status':'frozen-genuine-java-preset-reference','freshRunsAgree':True,'productionJarSha256':producers[0]['productionJarSha256'],
        'fixtureSha256':sha(values),'inputsSha256':sha(inputs),'recordCount':len(rows),'presetCases':5,'factoryGraphs':49,'producerReports':producers,
        'sourceSha256Lf':{name:sha((source_root/name).read_bytes().replace(b'\r\n',b'\n')) for name in SOURCES},
        'oracleSourceSha256Lf':sha((ROOT/'horizon/tests/java/NativePresetProbe.java').read_bytes().replace(b'\r\n',b'\n')),
        'scope':'Five cell-kernel setting groups, source optional defaults, control-point fallback, range helpers and 49 factory graphs; no full native terrain execution.',
        'nativeExpectedValuesUsed':False,'notCovered':['Cave/structure/miscellaneous execution','All preset/configuration domains','Continent/river/filtered-tile pipeline','Native client/save/launcher integration']}
    args.output.mkdir(parents=True);(args.output/'values.tsv').write_bytes(values);(args.output/'inputs.json').write_bytes(inputs)
    (args.output/'metadata.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8')
    print(json.dumps({k:report[k] for k in ('status','recordCount','factoryGraphs')}))
if __name__=='__main__':main()
