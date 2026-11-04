"""Validate and freeze genuine Java kernel captures from two independent JVMs."""
from __future__ import annotations
import argparse
from collections import Counter
import gzip
import hashlib
import json
import math
from pathlib import Path
import shutil
import struct

ROOT = Path(__file__).resolve().parents[2]


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def inspect(path):
    data = gzip.decompress(path.read_bytes())
    offset = 0
    def take(length):
        nonlocal offset
        value = data[offset:offset+length]
        if len(value)!=length: raise ValueError('Truncated kernel corpus')
        offset += length
        return value
    def number(format):
        return struct.unpack('>'+format,take(struct.calcsize('>'+format)))[0]
    def text():
        # All schema, terrain and enum names in v1 are ASCII. This deliberately
        # rejects unsupported modified-UTF strings instead of misdecoding them.
        return take(number('H')).decode('ascii')
    if text()!='TerraForged kernel oracle v1': raise ValueError('Wrong oracle format')
    world_seed, kernel_seed = number('q'), number('i')
    if kernel_seed!=(world_seed+2**31)%2**32-2**31: raise ValueError('Java seed narrowing mismatch')
    fields = [(text(),text()) for _ in range(number('i'))]
    if len(fields)!=24 or len({name for name,_ in fields})!=24: raise ValueError('Incomplete Cell schema')
    body_start = offset
    stages, terrains, biomes = Counter(), Counter(), Counter()
    layouts = []
    while True:
        stage = text()
        if stage=='complete':
            expected = number('q')
            if expected!=sum(stages.values()) or offset!=len(data): raise ValueError('Count/trailing-data mismatch')
            break
        if stage=='tile-layout':
            x,z,size,border,total = [number('i') for _ in range(5)]
            if size!=128 or total!=size+2*border or border not in (16,32): raise ValueError('Unexpected actual tile layout')
            layouts.append({'tileX':x,'tileZ':z,'size':size,'border':border,'total':total})
            continue
        if stage not in {'terrain','rivers','climate-true','climate-false','filtered-tile'}:
            raise ValueError('Unexpected stage '+stage)
        number('i'); number('i')
        for name,kind in fields:
            if kind=='float':
                if not math.isfinite(number('f')): raise ValueError('Nonfinite field '+name)
            elif kind=='int': number('i')
            elif kind=='boolean':
                if number('B') not in (0,1): raise ValueError('Invalid boolean')
            else:
                value = text()
                if not value: raise ValueError('Empty terrain/biome identity')
                if name=='terrain': terrains[value]+=1
                elif name=='biome': biomes[value]+=1
                else: raise ValueError('Unexpected Cell reference field')
        stages[stage]+=1
    if len(layouts)!=2 or stages['filtered-tile']!=sum(p['total']**2 for p in layouts):
        raise ValueError('Full actual tile backings/halos incomplete')
    return {'worldSeed':world_seed,'kernelSeed':kernel_seed,'cellRecords':sum(stages.values()),
        'stages':dict(stages),'layouts':layouts,'fields':fields,
        'terrainObservations':dict(terrains),'biomeObservations':dict(biomes),
        'cellBodySha256':hashlib.sha256(data[body_start:]).hexdigest()}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--first',type=Path,required=True)
    parser.add_argument('--second',type=Path,required=True)
    parser.add_argument('--output',type=Path,required=True)
    parser.add_argument('--report',type=Path,required=True)
    args = parser.parse_args()
    if args.output.exists() or args.report.exists(): parser.error('Refusing to replace a frozen kernel oracle')
    reports = [json.loads(p.read_text()) for p in (args.first,args.second)]
    snapshots = []
    for report in reports:
        qa = report.get('qaResult',report.get('runtimeQa'))
        if report.get('result',report.get('status'))!='passed' or qa['kernelOracle']['status']!='passed-java-reference-capture-only':
            raise ValueError('Java reference process failed')
        snapshots.append((ROOT/report['instance']/'qa-output/kernel-oracle-v1',qa['kernelOracle']))
    (first,a),(second,b) = snapshots
    if a!=b or a['productionJarSha256']!=qa['runtime']['modSha256']:
        raise ValueError('Fresh Java JVM captures/input bindings differ')
    for name,key in [('preset.json','presetSha256'),('noise-registry.json','noiseRegistrySha256')]:
        if sha(first/name)!=a[key] or sha(second/name)!=a[key]: raise ValueError('Reference inputs changed')
    records = []
    for row in a['captures']:
        if sha(first/row['file'])!=row['sha256'] or sha(second/row['file'])!=row['sha256']:
            raise ValueError('Captured reference bytes changed')
        if row['repeat']==0:
            parsed = inspect(first/row['file'])
            if parsed['cellRecords']!=row['cellRecords'] or parsed['worldSeed']!=row['worldSeed']:
                raise ValueError('Binary content differs from Java manifest')
            records.append({**row,**parsed})
    if len(records)!=6: raise ValueError('Six seed cases required')
    by_seed = {row['worldSeed']:row for row in records}
    if by_seed[8675309]['cellBodySha256']!=by_seed[8675309+2**32]['cellBodySha256']:
        raise ValueError('High32bit seed narrowing pair differs in actual kernel cells')
    args.output.mkdir(parents=True)
    for name in ['preset.json','noise-registry.json']+[row['file'] for row in records]:
        shutil.copyfile(first/name,args.output/name)
    result = {'schemaVersion':1,'status':'frozen-independent-java-kernel-reference','nativeImplementationPassed':False,
        'productionJarSha256':a['productionJarSha256'],'freshJvmsAgree':True,'freshContextsAgree':True,
        'uniqueSeedCases':6,'recordsPerCaptureSet':sum(r['cellRecords'] for r in records),
        'recordsAcrossBothContextRepeats':sum(r['cellRecords'] for r in a['captures']),
        'originalReports':{str(p):sha(p) for p in (args.first,args.second)},'captures':records,
        'fieldNames':a['fieldNames'],'files':{p.name:sha(p) for p in sorted(args.output.iterdir())},
        'scope':a['scope'],'notCovered':a['notCovered'],'archiveVerifierSha256':sha(Path(__file__))}
    args.report.parent.mkdir(parents=True,exist_ok=True)
    args.report.write_text(json.dumps(result,indent=2)+'\n',encoding='utf-8')
    print(json.dumps({'status':result['status'],'recordsPerCaptureSet':result['recordsPerCaptureSet']}))


if __name__=='__main__': main()
