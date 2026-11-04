"""Compare recorded native router climate fields with the frozen genuine Java capture.

Reuses captured native execution; never feeds expected values to the native code.
"""
from __future__ import annotations
import argparse,datetime as dt,gzip,hashlib,json,math,struct
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
FIELDS={'temperature':'temperature','vegetation':'moisture','continents':'continent','erosion':'erosion','ridges':'weirdness'}
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def rows(data):
    lines=data.splitlines();r=dict(line.split('\t',1) for line in lines)
    if len(r)!=len(lines):raise ValueError('Duplicate keys')
    return r
def f32(bits):return struct.unpack('>f',bytes.fromhex(bits))[0]
def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--integration',type=Path,required=True);p.add_argument('--outputs',type=Path,required=True);p.add_argument('--report',type=Path,required=True)
    a=p.parse_args()
    if a.report.exists():p.error('Refusing to replace climate qualification evidence')
    integration=json.loads(a.integration.read_text(encoding='utf-8'))
    if integration['status']!='passed-eight-field-router-integration-guards':raise ValueError('Completed router integration required')
    fixture=ROOT/'horizon/tests/fixtures/host-density-v1';meta=json.loads((fixture/'metadata.json').read_text(encoding='utf-8'))
    compressed=fixture/'values.tsv.gz';data=gzip.decompress(compressed.read_bytes());inputs=fixture/'inputs.json'
    if sha(compressed)!=meta['fixtureSha256'] or hashlib.sha256(data).hexdigest()!=meta['uncompressedValuesSha256'] or sha(inputs)!=meta['inputsSha256'] or not meta['freshRunsAgree']:raise ValueError('Independent reference changed')
    expected=rows(data.decode());config=json.loads(inputs.read_text(encoding='utf-8'));side=len(config['coordinates']);profiles=[]
    for item in integration['profiles']:
        file=a.outputs/(item['profile']+'.tsv')
        if sha(file)!=item['outputSha256']:raise ValueError('Native router output changed')
        actual=rows(file.read_text(encoding='utf-8'));differences=[];count=0;stats={}
        for key,value in actual.items():
            kind,seed,field,x,z,y=key.split('/')
            if kind!='router':raise ValueError('Unknown record kind')
            numeric=f32(value)
            if not math.isfinite(numeric):raise ValueError('Nonfinite router output')
            summary=stats.setdefault(field,{'min':numeric,'max':numeric,'unique':set(),'negative':0,'positive':0})
            summary['min']=min(summary['min'],numeric);summary['max']=max(summary['max'],numeric);summary['unique'].add(value)
            summary['negative']+=numeric<0;summary['positive']+=numeric>0
            if field in FIELDS:
                original=f'scalar/{seed}/{FIELDS[field]}/{3*(int(x)*side+int(z))}'
                count+=1
                if expected.get(original)!=value:differences.append({'key':key,'expectedJava':expected.get(original),'native':value})
        for summary in stats.values():summary['unique']=len(summary['unique'])
        varied=stats['final_density']['negative']>0 and stats['final_density']['positive']>0 and stats['chunk_surface_level']['unique']>1
        profiles.append({'profile':item['profile'],'comparedJavaFields':count,'differenceCount':len(differences),'firstDifferences':differences[:30],
                         'routerStats':stats,'nonconstantSurfaceAndDensitySigns':varied,'passed':count==14040 and not differences and varied})
    report={'schemaVersion':1,'status':'passed-router-java-climate-anchors' if all(row['passed'] for row in profiles) else 'failed',
        'completedAtUtc':dt.datetime.now(dt.timezone.utc).isoformat(),'integrationReportSha256':sha(a.integration),
        'referenceMetadataSha256':sha(fixture/'metadata.json'),'profiles':profiles,
        'scope':'Exact five climate-root f32 values against genuine independent Java mapped cell functions at all 13 native Y samples; nonconstant surface and both density signs.',
        'notCovered':['Independent final-density/depth/surface parity','Real chunks','Native client/save/modpack/launcher acceptance']}
    a.report.parent.mkdir(parents=True,exist_ok=True);a.report.write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8',newline='\n');print(json.dumps(report,indent=2))
    return 0 if report['status']=='passed-router-java-climate-anchors' else 1
if __name__=='__main__':raise SystemExit(main())
