"""Construct boundary inputs for the actual published Java biome policy."""
import json, math, struct
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
def bits(value):return struct.unpack('>I',struct.pack('>f',value))[0]
def f32(value):return struct.unpack('>f',struct.pack('>f',value))[0]
def undo(value,shift):
    result=value
    for n in range(shift,32,shift):result^=value>>n
    return result
def region_for_bucket(bucket):
    # Invert the bijective integer hash to exercise both sides of each cut.
    for tail in range(256):
        value=undo((bucket<<8)|tail,16)
        value=(value*pow(0x846ca68b,-1,1<<32))&0xffffffff
        value=undo(value,15)
        value=(value*pow(0x7feb352d,-1,1<<32))&0xffffffff
        value=undo(value,16)^0x632be59b
        if value&0x7f800000!=0x7f800000:return value
    raise ValueError('No finite input for bucket')
def main():
    regions=[0,0x80000000,1,0x7f7fffff,0xff7fffff,0x7f800000,0xff800000,0x7fc00001,0x7f800001,bits(.1),bits(.5)]
    for cut in (0.,.25,.35,.55,1/6,1.):
        target=f32(cut)*(1<<24)
        for bucket in sorted({max(0,min((1<<24)-1,int(math.floor(target))+offset)) for offset in (-1,0,1)}):
            regions.append(region_for_bucket(bucket))
    temperatures=[bits(-1),0,bits(.5),bits(1),0x7f800000,0xff800000,0x7fc00001]
    for cut in (.2,.25,.4,.75):
        center=bits(cut);temperatures.extend([center-1,center,center+1])
    parents=['badlands','eroded_badlands','old_growth_spruce_taiga','taiga','snowy_taiga','flower_forest','swamp','savanna','savanna_plateau','windswept_savanna','jungle','plains','sunflower_plains','snowy_plains','beach','snowy_beach','river','frozen_river','lush_caves','dripstone_caves','deep_dark','mushroom_fields','mangrove_swamp','cherry_grove','pale_garden','frozen_peaks','jagged_peaks','stony_peaks','grove','snowy_slopes','warm_ocean','future_surface_family']
    hashes=list(dict.fromkeys(regions))+[((i*0x9e3779b9)^0x12345678)&0xffffffff for i in range(8192)]
    inputs={'schemaVersion':1,'parents':['minecraft:'+p for p in parents]+['biomesoplenty:overgrown_greens'],
        'terrains':['flats','lake','wetland','beach','river','coast','native_clone_lake','native_clone_wetland','native_clone_beach'],
        'regionBits':[f'{v:08x}' for v in dict.fromkeys(regions)],'temperatureBits':[f'{v:08x}' for v in dict.fromkeys(temperatures)],
        'hashBits':[f'{v:08x}' for v in dict.fromkeys(hashes)]}
    dest=ROOT/'horizon/tests/inputs/classic-biomes-v1.json'
    with dest.open('x',encoding='utf-8',newline='\n') as f:f.write(json.dumps(inputs,indent=2)+'\n')
    print(json.dumps({'regions':len(inputs['regionBits']),'temperatures':len(inputs['temperatureBits']),'parents':len(inputs['parents'])}))
if __name__=='__main__':main()
