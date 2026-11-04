"""Freeze graph results from two genuine production Fabric JVMs with active mixins."""
from __future__ import annotations
import argparse
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
NAMES = ['Constant','Add','Multiply','Frequency','ShiftSeed','Abs','Invert','Min','Max','Clamp','Map','Simplex','Perlin2']
SHAPING_NAMES = ['Constant','Add','Multiply','Map','Simplex','White','Cubic','PerlinRidge','SimplexRidge','Billow',
    'Sin','Power','PowerCurve','Curve','Gradient','Blend','Alpha','Boost','Steps','Threshold',
    'function.SCurveFunction','function.Interpolation','NoiseUtil']
CELLULAR_NAMES = ['Constant','Add','Simplex','Worley','WorleyEdge','LegacyTemperature','LegacyMoisture','LinearSpline','Cache2d','Cache2d$Cached',
    'function.CellFunction','function.DistanceFunction','function.EdgeFunction','NoiseUtil']
FINAL_NAMES = ['Constant','Add','Multiply','Map','Simplex','Perlin2','Line','Terrace','AdvancedTerrace','Warp','Erosion',
    'domain.DirectWarp','domain.DomainWarp','domain.DirectionWarp','domain.AddWarp','domain.CompoundWarp','NoiseUtil']


def sha(data): return hashlib.sha256(data).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--first',type=Path,required=True)
    parser.add_argument('--second',type=Path,required=True)
    parser.add_argument('--output',type=Path,required=True)
    parser.add_argument('--family',choices=['composition','shaping','cellular','final'],default='composition')
    args = parser.parse_args()
    names = {'composition':NAMES,'shaping':SHAPING_NAMES,'cellular':CELLULAR_NAMES,'final':FINAL_NAMES}[args.family]
    graph_count = {'composition':40,'shaping':85,'cellular':158,'final':78}[args.family]
    extra_records = {'cellular':18,'final':15898}.get(args.family,0)
    metadata = args.output.with_suffix('.json')
    if args.output.exists() or metadata.exists(): parser.error('Refusing to replace graph reference bytes')
    snapshots, reports = [], []
    for path in (args.first,args.second):
        raw = path.read_bytes()
        report = json.loads(raw)
        qa = report['qaResult']
        if report['result']!='passed' or report['processExitCode']!=0 or report['developmentMode'] is not False or not report['packagedArtifactOriginObserved']:
            raise ValueError('Genuine production runtime did not pass')
        capture = qa[{'composition':'noiseComposition','shaping':'noiseShaping','cellular':'noiseCellular','final':'noiseFinal'}[args.family]]
        data = Path(capture['file']).read_bytes()
        if capture['status']!='captured-genuine-java-graphs' or sha(data)!=capture['sha256']:
            raise ValueError('Reference capture bytes changed')
        jar_name = Path(report['artifact']['name']).name
        class_log = Path(report['classLoadLog'])
        if not class_log.is_absolute(): class_log = path.resolve().parents[2]/class_log
        class_bytes = class_log.read_bytes()
        origins = class_bytes.decode('utf-8',errors='strict').splitlines()
        verified_origins = []
        for name in names:
            suffix = name if '.' in name or name=='NoiseUtil' else 'module.'+name
            expected = 'raccoonman.reterraforged.world.worldgen.noise.'+suffix+' source:'
            lines = [line for line in origins if expected in line]
            if len(lines)!=1 or jar_name not in lines[0]:
                raise ValueError('Original production class origin absent: '+name)
            verified_origins.append(lines[0])
        snapshots.append(data)
        reports.append({'report':path.name,'sha256':sha(raw),'productionJarSha256':qa['runtime']['modSha256'],
                        'pid':report['pid'],'instance':report['instance'],
                        'classLoadLogSha256':sha(class_bytes),'verifiedOriginalClassOrigins':verified_origins})
    if snapshots[0]!=snapshots[1] or reports[0]['pid']==reports[1]['pid'] or reports[0]['productionJarSha256']!=reports[1]['productionJarSha256']:
        raise ValueError('Independent JVM graphs or artifact identities differ')
    lines = snapshots[0].decode().splitlines()
    records = dict(line.split('\t',1) for line in lines)
    if len(records)!=len(lines): raise ValueError('Duplicate graph record keys')
    graphs = {k:v for k,v in records.items() if k.startswith('graph/')}
    if len(graphs)!=graph_count or len(lines)!=graph_count*(1+6*17*17)+extra_records: raise ValueError('Graph corpus incomplete')
    source_root = ROOT/'common/src/main/java/raccoonman/reterraforged/world/worldgen/noise'
    sources = {name:hashlib.sha256((source_root/((name.replace('.','/') if '.' in name or name=='NoiseUtil' else 'module/'+name).split('$')[0]+'.java')).read_bytes().replace(b'\r\n',b'\n')).hexdigest() for name in names}
    probe = ROOT/('horizon/tests/java/'+{'composition':'NativeNoiseCompositionProbe','shaping':'NativeNoiseShapingProbe','cellular':'NativeNoiseCellularProbe','final':'NativeNoiseFinalProbe'}[args.family]+'.java')
    result = {'schemaVersion':1,'status':'frozen-genuine-game-java-reference','freshRunsAgree':True,
        'productionJarSha256':reports[0]['productionJarSha256'],'producerReports':reports,
        'fixtureSha256':sha(snapshots[0]),'recordCount':len(lines),'graphCount':len(graphs),
        'sourceSha256Lf':sources,'oracleSourceSha256Lf':sha(probe.read_bytes().replace(b'\r\n',b'\n')),
        'graphManifests':graphs,'scope':str(graph_count)+' raw constructor graphs: compute values plus actual legacy min/max; no visitor or terrain integration.',
        'nativeExpectedValuesUsed':False,'format':'UTF-8 LF TSV; graph metadata and float values use raw bits',
        'notCovered':['mapAll visitor transforms','Registry graph decoding','Every float/input domain','Degenerate Map alpha ranges','Native terrain activation']}
    args.output.parent.mkdir(parents=True,exist_ok=True)
    args.output.write_bytes(snapshots[0])
    metadata.write_text(json.dumps(result,indent=2)+'\n',encoding='utf-8')
    print(json.dumps({'status':result['status'],'records':len(lines),'graphs':len(graphs)}))


if __name__=='__main__': main()
