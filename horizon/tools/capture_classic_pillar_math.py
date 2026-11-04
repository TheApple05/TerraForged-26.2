"""Freeze independent classic pillar arithmetic from unmodified Java source."""
from __future__ import annotations
import argparse,datetime as dt,gzip,hashlib,json,os,re,struct,subprocess
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
def sha(data):return hashlib.sha256(data).hexdigest()
def bits(value):return struct.pack('>f',value).hex()
def main():
    p=argparse.ArgumentParser(description=__doc__)
    for name in ('java-home','work','output'):p.add_argument('--'+name,required=True,type=Path)
    a=p.parse_args()
    if a.work.exists() or a.output.exists() or not a.work.resolve().is_relative_to((ROOT/'build/qa/horizon').resolve()):p.error('Fresh QA work and fixture paths required')
    a.work.mkdir(parents=True);classes=a.work/'classes';classes.mkdir();lines=[]
    for m,maximum in enumerate([1.0,0.999]):
        for i in range(-8,8201):lines.append(f'terrace/{m}/{i}\tterrace\t{bits(i/8192)}\t{bits(maximum)}')
    values=[0xff800000,0xbf800000,0xbdcccccd,0x80000000,0,1,0x3983126f,0x3a03126f,0x3dcccccd,0x3e800000,0x3f000000,0x3f400000,0x3f7fbe77,0x3f800000,0x40000000,0x7f800000,0x7fc00000,0xffc00000,0x7fc01234]
    for m,maximum in enumerate([1.0,0.999]):
        for i,value in enumerate(values):lines.append(f'terrace-edge/{m}/{i}\tterrace\t{value:08x}\t{bits(maximum)}')
    for i,edge in enumerate(values):
        for j,river in enumerate(values):
            for k,gradient in enumerate(values):lines.append(f'mask/{i}/{j}/{k}\tmask\t{edge:08x}\t{river:08x}\t{gradient:08x}')
    for i,shape in enumerate(values):
        for j,mask in enumerate(values):
            for stone in (0,1):lines.append(f'height/{i}/{j}/{stone}\theight\t{shape:08x}\t{mask:08x}\t{stone}')
    inputs=('\n'.join(lines)+'\n').encode();input_file=a.work/'inputs.tsv';input_file.write_bytes(inputs)
    math=ROOT/'common/src/main/java/raccoonman/reterraforged/world/worldgen/feature/ClassicPillarMath.java';probe=ROOT/'horizon/tests/java/NativeClassicPillarMathProbe.java'
    suffix='.exe' if os.name=='nt' else ''
    cmd=[str(a.java_home/'bin'/('javac'+suffix)),'--release','25','-encoding','UTF-8','-proc:none','-d',str(classes),str(math),str(probe)]
    result=subprocess.run(cmd,capture_output=True,text=True,timeout=60);(a.work/'compile.log').write_text(result.stdout+result.stderr,encoding='utf-8')
    if result.returncode:raise RuntimeError('Java math probe compile failed')
    observations=[];runs=[]
    for index in range(2):
        file=a.work/f'java-{index}.tsv'
        result=subprocess.run([str(a.java_home/'bin'/('java'+suffix)),'-cp',str(classes),'NativeClassicPillarMathProbe',str(input_file.resolve()),str(file.resolve()),str(classes.resolve())],capture_output=True,text=True,timeout=90)
        log=result.stdout+result.stderr;(a.work/f'java-{index}.log').write_text(log,encoding='utf-8');match=re.search(r'PILLAR_MATH_CAPTURE_OK records=(\d+) pid=(\d+)',log)
        if result.returncode or not match or int(match[1])!=len(lines):raise RuntimeError('Java producer failed')
        raw=file.read_bytes();observations.append(raw);runs.append({'pid':int(match[2]),'exitCode':result.returncode,'records':len(lines),'outputSha256':sha(raw),'logSha256':sha(log.encode())})
    if observations[0]!=observations[1] or runs[0]['pid']==runs[1]['pid']:raise ValueError('Independent Java captures differ')
    # The first 16,418 results also match the pre-existing original Noise2D legacy-terrace reference.
    terrace=bytes.fromhex(''.join(line.split('\t')[1] for line in observations[0].decode().splitlines()[:16418]))
    reference=json.loads((ROOT/'tests/terrain/classic-pillar-reference.json').read_text(encoding='utf-8'))
    if sha(terrace)!=reference['terraceRawBitsSha256']:raise ValueError('Legacy reference differs')
    compressed=gzip.compress(observations[0],mtime=0)
    report={'schemaVersion':1,'status':'frozen-unmodified-java-classic-pillar-math','capturedAtUtc':dt.datetime.now(dt.timezone.utc).isoformat(),'recordCount':len(lines),'inputsSha256':sha(inputs),'fixtureSha256':sha(compressed),'uncompressedValuesSha256':sha(observations[0]),'sourceSha256Lf':{str(p.relative_to(ROOT)).replace(os.sep,'/'):sha(p.read_bytes().replace(b'\r\n',b'\n')) for p in (math,probe)},'producerReports':runs,'freshJavaRunsAgree':True,'legacyTerraceReferenceSha256':sha(terrace),'nativeExpectedValuesUsed':False,'nanPayloadBitsCompared':False,'scope':'Exact finite/infinite/zero f32 results and integer heights; NaN results compare as NaN. Unmodified project math source, plus an independently frozen original Noise2D terrace reference. No game APIs or downloads.','notCovered':['Pillar noise shapes/palettes','Biome boundary masks','Block placement','Native world integration or scenery']}
    a.output.mkdir(parents=True);(a.output/'inputs.tsv').write_bytes(inputs);(a.output/'values.tsv.gz').write_bytes(compressed);(a.output/'metadata.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8',newline='\n')
    print(json.dumps({'status':report['status'],'records':len(lines),'freshJavaRunsAgree':True}))
if __name__=='__main__':main()
