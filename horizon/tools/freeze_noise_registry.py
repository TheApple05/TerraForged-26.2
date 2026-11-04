"""Freeze actual registry/codec observations from two genuine production game JVMs."""
from __future__ import annotations
import argparse
import hashlib
import json
from pathlib import Path
from freeze_noise_composition import VISITOR_NAMES

ROOT=Path(__file__).resolve().parents[2]
def sha(data):return hashlib.sha256(data).hexdigest()

def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--first',type=Path,required=True)
    p.add_argument('--second',type=Path,required=True)
    p.add_argument('--output',type=Path,required=True)
    args=p.parse_args()
    if args.output.exists():p.error('Refusing to replace registry reference directory')
    captures=[];producers=[]
    for path in (args.first,args.second):
        raw=path.read_bytes();r=json.loads(raw)
        if r['result']!='passed' or r['processExitCode']!=0 or r['developmentMode'] is not False or not r['packagedArtifactOriginObserved']:
            raise ValueError('Actual production registry run did not pass')
        c=r['qaResult']['noiseRegistry']
        values=Path(c['file']).read_bytes();inputs=Path(c['inputs']).read_bytes()
        if c['status']!='captured-genuine-java-registry' or sha(values)!=c['sha256'] or sha(inputs)!=c['inputsSha256']:
            raise ValueError('Registry capture identity changed')
        log=Path(r['classLoadLog'])
        if not log.is_absolute():log=path.resolve().parents[2]/log
        log_bytes=log.read_bytes();lines=log_bytes.decode().splitlines();origins=[]
        for name in VISITOR_NAMES+['function.Interpolation']:
            suffix=name if '.' in name or name=='NoiseUtil' else 'module.'+name
            matches=[s for s in lines if 'raccoonman.reterraforged.world.worldgen.noise.'+suffix+' source:' in s]
            if len(matches)!=1 or r['artifact']['name'] not in matches[0]:raise ValueError('Genuine class origin missing: '+name)
            origins.append(matches[0])
        captures.append((values,inputs))
        producers.append({'report':path.name,'sha256':sha(raw),'pid':r['pid'],'instance':r['instance'],
            'productionJarSha256':r['artifact']['sha256'],'classLoadLogSha256':sha(log_bytes),'verifiedOriginalClassOrigins':origins})
    if captures[0]!=captures[1] or producers[0]['pid']==producers[1]['pid'] or producers[0]['productionJarSha256']!=producers[1]['productionJarSha256']:
        raise ValueError('Fresh registry captures disagree')
    values,inputs=captures[0];rows=dict(line.split('\t',1) for line in values.decode().splitlines())
    documents=json.loads(inputs)
    if len(documents)!=78 or len(rows)!=75*1735+3 or len(rows)!=len(values.splitlines()):raise ValueError('Registry corpus incomplete')
    sources={}
    for name in VISITOR_NAMES+['function.Interpolation']:
        suffix=name if '.' in name or name=='NoiseUtil' else 'module.'+name
        source=ROOT/'common/src/main/java/raccoonman/reterraforged/world/worldgen/noise'/(suffix.replace('.','/').split('$')[0]+'.java')
        sources[name]=sha(source.read_bytes().replace(b'\r\n',b'\n'))
    probe=ROOT/'horizon/tests/java/NativeNoiseRegistryProbe.java'
    report={'schemaVersion':1,'status':'frozen-genuine-java-registry-reference','freshRunsAgree':True,
        'productionJarSha256':producers[0]['productionJarSha256'],'producerReports':producers,
        'fixtureSha256':sha(values),'inputsSha256':sha(inputs),'recordCount':len(rows),'rootCount':78,'successfulRoots':75,'expectedRejectedInputs':['qa:linear_spline','qa:linear_spline_numeric','qa:linear_spline_empty'],
        'actualRegistryEntries':26,'syntheticCodecInputs':52,'sourceSha256Lf':sources,
        'oracleSourceSha256Lf':sha(probe.read_bytes().replace(b'\r\n',b'\n')),
        'nonfiniteSampleRecords':sum(v in ('NaN','+Infinity','-Infinity') for k,v in rows.items() if k.startswith('sample/')),
        'scope':'Actual registry values plus genuine JSON codec parsing/rejection across all 41 types, high octaves, references and decimal rounding.',
        'nativeExpectedValuesUsed':False,'notCovered':['Every codec parameter/input domain','Native cell/river/tile pipeline','Native game activation']}
    args.output.mkdir(parents=True)
    (args.output/'values.tsv').write_bytes(values);(args.output/'inputs.json').write_bytes(inputs)
    (args.output/'metadata.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8')
    print(json.dumps({k:report[k] for k in ('status','rootCount','recordCount','nonfiniteSampleRecords')}))

if __name__=='__main__':main()
