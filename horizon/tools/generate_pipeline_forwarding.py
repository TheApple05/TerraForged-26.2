"""Generate mechanical forwarding from the pinned MIT Horizon pipeline trait.

Only Rust API signatures are copied. No game data is read. The marker-delimited
tail is deterministic and checked against the pinned source before replacement.
"""
import argparse,hashlib,re,subprocess
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
PIN='0e0f70ff1f8191479dbb5b1b173a74d38d324005'
MARKER='// Pipeline forwarding follows below. Regenerate with the source-only helper.'
def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--horizon',type=Path,required=True);p.add_argument('--check',action='store_true');a=p.parse_args()
    source=a.horizon.resolve();git=['git','-c',f'safe.directory={source.as_posix()}','-C',str(source)]
    data=subprocess.check_output(git+['show',PIN+':crates/horizon-world/src/worldgen/generator.rs']).decode()
    trait=data[data.index('trait SeededPipeline:'):data.index('\nimpl<R: RandomSource',data.index('trait SeededPipeline:'))]
    methods=[];method_names=[]
    for match in re.finditer(r'\bfn\s+(\w+)\s*\(',trait):
        name=match.group(1);start=match.end()-1;depth=1;end=start+1
        method_names.append(name)
        while depth:
            depth+=(trait[end]=='(')-(trait[end]==')');end+=1
        arguments=trait[start+1:end-1];tail=end;squares=0
        while True:
            char=trait[tail]
            if char in ';{' and squares==0:break
            squares+=(char=='[')-(char==']');tail+=1
        attribute=re.search(r'(#\[[^\n]+\])\s*$',trait[:match.start()])
        prefix=''
        if attribute:
            if attribute[1]!='#[cfg(test)]':raise ValueError('Unexpected pipeline method attribute: '+attribute[1])
            prefix=attribute[1]+'\n'
        signature=prefix+trait[match.start():tail].strip().replace('super::','crate::worldgen::')
        names=re.findall(r'\b(\w+)\s*:(?!:)',arguments)
        if name=='release_caches':
            body='self.inner.release_caches();\nif let Err(error) = self.kernel.release_caches() { log::error!("TerraForged cache release failed: {error:?}"); }'
        else:body='self.inner.'+name+'('+', '.join(names)+')'
        methods.append(signature+' {\n'+body+'\n}')
    expected='generate decorate data cache_census release_caches end_island_origins sea_level level surface_sample lod_grid structure_starts structure_start unsupported_features use_saved_start forget_structure_starts ring_positions may_start_answers spawn_position region_random locate set_old_world debug_density_line find_closest_biome noise_biomes jigsaw_pieces'.split()
    if method_names!=expected:raise ValueError(f'Pipeline API changed: {method_names}')
    target=ROOT/'horizon/native-world/world_runtime.rs';head=target.read_text(encoding='utf-8').split(MARKER)[0]
    generated=head+MARKER+'\n// Trait source SHA256: '+hashlib.sha256(trait.encode()).hexdigest()+'\nimpl SeededPipeline for NativePipeline {\n'+'\n'.join(methods)+'\n}\n'
    # rustfmt is the only formatting step; do not mutate the source checkout.
    formatted=subprocess.run(['rustfmt','+stable','--edition','2024','--emit','stdout'],input=generated,capture_output=True,text=True)
    if formatted.returncode:raise ValueError(formatted.stderr)
    result=formatted.stdout
    if a.check:
        if target.read_text(encoding='utf-8')!=result:raise ValueError('Forwarding tail is stale')
    else:target.write_text(result,encoding='utf-8',newline='\n')
    print(f'PINNED_PIPELINE_FORWARDING_OK {len(methods)} methods')
if __name__=='__main__':main()
