"""Compare native terrain with the preexisting independent full Java kernel corpus.

Native code receives only preset/registry/coordinate/layout inputs; no expected
cells or snapshots. Only the explicitly selected stage set is compared.
"""
from __future__ import annotations
import argparse
import datetime as dt
import gzip
import hashlib
import json
from pathlib import Path
import struct
import subprocess
from verify_native_noise import rows

ROOT=Path(__file__).resolve().parents[2]
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def terrain_rows(path,stages=('terrain',)):
    data=gzip.decompress(path.read_bytes());offset=0
    def take(length):
        nonlocal offset
        chunk=data[offset:offset+length]
        if len(chunk)!=length:raise ValueError('Truncated original kernel capture')
        offset+=length;return chunk
    def number(format):return struct.unpack('>'+format,take(struct.calcsize('>'+format)))[0]
    def text():return take(number('H')).decode('ascii')
    if text()!='TerraForged kernel oracle v1':raise ValueError('Unexpected original kernel schema')
    seed=number('q');narrowed=number('i');fields=[(text(),text()) for _ in range(number('i'))]
    if narrowed!=(seed+2**31)%2**32-2**31 or len(fields)!=24:raise ValueError('Original kernel header changed')
    result={};count=0;tile=None
    while True:
        stage=text()
        if stage=='complete':
            if number('q')!=count or offset!=len(data):raise ValueError('Original kernel count/trailing bytes changed')
            break
        if stage=='tile-layout':
            tile=tuple(number('i') for _ in range(5))
            continue
        if stage not in ('terrain','rivers','climate-true','climate-false','filtered-tile'):raise ValueError('Unknown original stage')
        x=number('i');z=number('i');values=[]
        for name,kind in fields:
            if kind=='float':values.append(f'{number("I"):08x}')
            elif kind=='int':values.append(str(number('i')))
            elif kind=='boolean':
                b=number('B')
                if b not in (0,1):raise ValueError('Invalid original boolean')
                values.append('true' if b else 'false')
            else:values.append(text())
        if stage in stages:
            tile_key=f'{tile[0]}/{tile[1]}/' if stage=='filtered-tile' else ''
            key=f'{stage}/{seed}/{tile_key}{x}/{z}'
            if key in result:raise ValueError('Duplicate original terrain point')
            result[key]=values
        count+=1
    if len(result)!=sum(51200 if stage=='filtered-tile' else 121 for stage in stages):raise ValueError('Original selected stages incomplete')
    return seed,result

def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--report',type=Path,required=True)
    p.add_argument('--stages',choices=['terrain','unfiltered','full'],default='terrain');args=p.parse_args()
    stages=('terrain',) if args.stages=='terrain' else ('terrain','rivers','climate-true','climate-false')
    if args.stages=='full':stages+=('filtered-tile',)
    passed_status={'terrain':'passed-original-kernel-terrain-only','unfiltered':'passed-original-kernel-unfiltered-stages','full':'passed-original-kernel-all-stages'}[args.stages]
    if args.report.exists():p.error('Refusing to replace original-kernel terrain evidence')
    manifest=ROOT/'docs/validation/horizon-kernel-java-oracle-v1.json';metadata=json.loads(manifest.read_text(encoding='utf-8'))
    if metadata['status']!='frozen-independent-java-kernel-reference' or not metadata['freshJvmsAgree'] or not metadata['freshContextsAgree']:raise ValueError('Original independent kernel evidence absent')
    folder=ROOT/'horizon/tests/fixtures/kernel-v1'
    for name,digest in metadata['files'].items():
        if sha(folder/name)!=digest:raise ValueError('Original frozen input/capture changed: '+name)
    expected={};seeds=[]
    for capture in metadata['captures']:
        seed,values=terrain_rows(folder/capture['file'],stages);seeds.append(seed);expected.update(values)
    if len(expected)!=6*sum(51200 if stage=='filtered-tile' else 121 for stage in stages) or len(seeds)!=6:raise ValueError('Six original seed cases required')
    points=sorted({tuple(map(int,key.split('/')[-2:])) for key in expected if not key.startswith('filtered-tile/')})
    work=ROOT/'build/qa/horizon/kernel-terrain'/args.report.stem;work.mkdir(parents=True,exist_ok=True)
    layouts=metadata['captures'][0]['layouts'] if args.stages=='full' else []
    for capture in metadata['captures']:
        if args.stages=='full' and (capture['layouts']!=layouts or capture['tileSizeFactor']!=3):raise ValueError('Original tile input layout changed')
    queries=work/'queries.json';queries.write_text(json.dumps({'seeds':seeds,'points':points,'layouts':layouts})+'\n',encoding='utf-8',newline='\n')
    report={'schemaVersion':1,'status':'failed','startedAtUtc':dt.datetime.now(dt.timezone.utc).isoformat(),'originalKernelManifestSha256':sha(manifest),
        'inputSha256':{name:sha(folder/name) for name in ['preset.json','noise-registry.json']},'queriesSha256':sha(queries),
        'recordsPerProfile':len(expected),'fieldsPerProfile':sum(map(len,expected.values())),'profiles':[],
        'scope':'Native '+args.stages+' stages against original full-kernel Java observations; no expected cell data is read by native code.',
        'notCovered':(['River execution','Climate acceptance from this corpus'] if args.stages=='terrain' else [])+(['Filtered tiles and halos'] if args.stages!='full' else [])+['Other presets or tile layouts from this corpus','Native chunks/client/save/launcher']}
    for profile in ('debug','release'):
        command=['cargo','+stable','run','--quiet','--offline','--locked','--manifest-path',str(ROOT/'horizon/native-adapter/Cargo.toml'),'--bin','kernel_terrain_probe' if args.stages=='terrain' else 'kernel_cell_probe']
        if profile=='release':command.append('--release')
        command+=['--',str(folder/'preset.json'),str(folder/'noise-registry.json'),str(queries)]
        run=subprocess.run(command,cwd=ROOT,capture_output=True,text=True,timeout=120)
        log=work/(profile+'.tsv');log.write_text(run.stdout,encoding='utf-8',newline='\n');(work/(profile+'.stderr.log')).write_text(run.stderr,encoding='utf-8')
        actual=rows(run.stdout) if run.returncode==0 else {}
        differences=[{'key':k,'expected':expected.get(k),'actual':actual.get(k)} for k in sorted(set(actual)|set(expected)) if actual.get(k)!=expected.get(k)]
        report['profiles'].append({'profile':profile,'exitCode':run.returncode,'differenceCount':len(differences),'firstDifferences':differences[:30],
            'outputSha256':sha(log),'allJavaValuesMatch':run.returncode==0 and not differences})
    report['status']=passed_status if all(p['allJavaValuesMatch'] for p in report['profiles']) else 'failed'
    report['sourceSha256Lf']={p.relative_to(ROOT).as_posix():hashlib.sha256(p.read_bytes() if p.suffix=='.bin' else p.read_bytes().replace(b'\r\n',b'\n')).hexdigest()
        for crate in ['native-core','native-adapter'] for p in sorted((ROOT/'horizon'/crate/'src').rglob('*')) if p.is_file() and p.suffix in ('.rs','.bin')}
    report['verifierSha256']=sha(Path(__file__));report['finishedAtUtc']=dt.datetime.now(dt.timezone.utc).isoformat()
    args.report.parent.mkdir(parents=True,exist_ok=True);args.report.write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8')
    print(json.dumps({k:report[k] for k in ('status','recordsPerProfile','fieldsPerProfile')}))
    return 0 if report['status']==passed_status else 1
if __name__=='__main__':raise SystemExit(main())
