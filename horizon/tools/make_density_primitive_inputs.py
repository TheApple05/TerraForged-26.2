"""Build explicit test inputs, never expected native/Java results."""
import copy,json,math,struct
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
def bits(value,kind='d'):return struct.pack('>'+kind,value).hex()
def main():
    prior=json.loads((ROOT/'horizon/tests/inputs/heightmap-climate-v1.json').read_text(encoding='utf-8'))
    presets={name:copy.deepcopy(prior['presets']['default']) for name in ('default','degenerate','fallback')}
    presets['degenerate']['world']['properties'].update(worldHeight=448,seaLevel=96)
    presets['degenerate']['world']['controlPoints']['deepOcean']=0.0
    presets['fallback']['world']['properties'].update(worldHeight=1,seaLevel=0)
    presets['fallback']['world']['controlPoints']['deepOcean']=2.0
    heights=[-1.0,-0.0,0.0,0.24,0.24609375,0.25,0.265625,0.3,0.5,1.0,2.0,math.inf,math.nan]
    edges=[-1.0,-0.0,0.0,0.05,0.1,0.2,0.25,0.327,0.448,0.502,0.75,1.0,2.0,math.inf,math.nan]
    doubles=[-math.inf,-1e300,-2147483649.0,-2147483648.0,-2.0,-1.0,-0.5,math.nextafter(-0.5,-math.inf),math.nextafter(-0.5,math.inf),-0.0,0.0,math.nextafter(0.0,1.0),math.nextafter(0.3,-math.inf),0.3,math.nextafter(0.3,math.inf),math.nextafter(0.5,-math.inf),0.5,math.nextafter(0.5,math.inf),0.7,1.0,math.nextafter(1.0,0.0),math.nextafter(1.0,math.inf),2.0,2147483647.0,2147483648.0,1e300,math.inf,math.nan]
    splines=[]
    for locations,values in [([-1,0,1],[-3,1,4]),([0,0.3,0.3,1],[1,2,-2,3]),([0],[0.1]),([-1,1],[-1e-300,1e-300])]:
        splines.append({'locationBits':[bits(v) for v in locations],'valueBits':[bits(v) for v in values],
            'boundsBits':[[bits(v-1.3),bits(v+2.65)] for v in values]})
    data={'presets':presets,'noiseRegistry':prior['noiseRegistry'],'heightBits':[bits(v,'f') for v in heights],
        'edgeBits':[bits(v,'f') for v in edges],'beachNoises':[-1.0,0.0,0.025,1.0],
        'doubleBits':[bits(v) for v in doubles],'resolutions':[-2147483648,-256,-1,0,1,2,3,128,256,2147483647],
        'splineCases':splines,'positions':[[0,0,0],[-1024,64,2048],[2147483647,-64,-2147483648]],
        'cacheWidths':[0,1,4,16],'cacheOrigins':[[0,0],[-1,2],[2147483645,-2147483646]],
        'cacheYs':[-64,0,1,320],'noiseSeeds':[-2147483648,-1,0,8675309,2147483647]}
    output=ROOT/'horizon/tests/inputs/density-primitives-v1.json';raw=(json.dumps(data,indent=2)+'\n').encode('utf-8')
    if output.exists():
        if output.read_bytes()!=raw:raise ValueError('Refusing to change predefined primitive inputs')
    else:output.write_bytes(raw)
    print(json.dumps({'status':'predefined-inputs-ready','heightInputs':len(heights),'edgeInputs':len(edges),'doubleInputs':len(doubles)}))
if __name__=='__main__':main()
