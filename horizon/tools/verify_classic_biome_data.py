"""Check native biome derivation against installed resource inputs, without exporting game files to Git."""
from __future__ import annotations
import argparse,datetime as dt,hashlib,json,os,subprocess,shutil
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
def sha(path):return hashlib.sha256(path.read_bytes()).hexdigest()
def resolve_tags(raw,known):
    # Match the host's missing-tag policy: reject that tag independently.
    # An invalid tag remains absent, so using it in a feature still fails.
    definitions={name:json.loads(value) for name,value in raw.items()};memo={};active=set();invalid={}
    def visit(name,depth=0):
        if depth>128:raise ValueError('Tag depth exceeded')
        if name in active:raise ValueError('Cyclic tag '+name)
        if name in memo:return memo[name]
        if name not in definitions:raise ValueError('Missing tag '+name)
        active.add(name)
        try:
            values=set()
            for entry in definitions[name]['values']:
                identifier=entry if isinstance(entry,str) else entry['id'];required=isinstance(entry,str) or entry.get('required',True)
                tag=identifier.startswith('#');identifier=identifier.lstrip('#')
                if ':' not in identifier:identifier='minecraft:'+identifier
                exists=identifier in (definitions if tag else known)
                if not exists:
                    if required:raise ValueError('Required tag member absent: '+identifier)
                    continue
                try:values.update(visit(identifier,depth+1) if tag else [identifier])
                except ValueError:
                    if required:raise
            memo[name]=sorted(values);return memo[name]
        finally:active.remove(name)
    for name in definitions:
        try:visit(name)
        except ValueError as error:invalid[name]=str(error)
    return memo,invalid

def main():
    p=argparse.ArgumentParser(description=__doc__)
    for name in ('resources','work','report'):p.add_argument('--'+name,required=True,type=Path)
    a=p.parse_args();work=a.work.resolve()
    if not work.is_relative_to((ROOT/'build/qa/horizon').resolve()) or work.exists() or a.report.exists():p.error('Choose fresh QA work and report paths')
    work.mkdir(parents=True);shutil.copy2(Path(__file__),work/'verifier-source.py');directory=a.resources.resolve()/'worldgen'
    names=('biomes','features','placed_features','biome_tags','feature_tags');paths={n:directory/(n+'.json') for n in names}
    before={n:sha(path) for n,path in paths.items()};bundles={n:json.loads(path.read_text(encoding='utf-8')) for n,path in paths.items()}
    invalid_tags={}
    for tag,known in [('biome_tags','biomes'),('feature_tags','placed_features')]:bundles[tag],invalid_tags[tag]=resolve_tags(bundles[tag],bundles[known])
    inputs=work/'inputs.json';inputs.write_text(json.dumps(bundles,sort_keys=True,separators=(',',':')),encoding='utf-8')
    env=dict(os.environ,CARGO_BUILD_JOBS='4',CARGO_NET_OFFLINE='true');runs=[];outputs=[]
    manifest=ROOT/'horizon/native-adapter/Cargo.toml'
    for profile in ('debug','release'):
        cmd=['cargo','+stable','build','--offline','--locked','--manifest-path',str(manifest),'--bin','classic_biome_data_probe']+(['--release'] if profile=='release' else [])
        result=subprocess.run(cmd,cwd=ROOT,env=env,capture_output=True,text=True,timeout=300);(work/(profile+'-build.log')).write_text(result.stdout+result.stderr,encoding='utf-8')
        if result.returncode:raise RuntimeError('Resource probe build failed; see preserved logs')
        exe=ROOT/'horizon/native-adapter/target'/profile/('classic_biome_data_probe'+('.exe' if os.name=='nt' else ''))
        result=subprocess.run([str(exe),str(inputs)],cwd=work,env=env,capture_output=True,timeout=120)
        output=work/(profile+'-output.json');output.write_bytes(result.stdout);(work/(profile+'-stderr.log')).write_bytes(result.stderr)
        if result.returncode:raise RuntimeError('Resource probe failed: '+result.stderr.decode(errors='replace'))
        data=json.loads(result.stdout)
        if len(data['biomes'])!=17 or len(data['placed_features'])!=8 or len(data['configured_features'])!=3:raise ValueError('Incomplete variant/feature resources')
        for name,raw in data['biomes'].items():
            biome=json.loads(raw)
            if len(biome['features'])!=11 or 'attributes' not in biome or 'effects' not in biome:raise ValueError('Invalid derived biome '+name)
        outputs.append(result.stdout);runs.append({'profile':profile,'exitCode':result.returncode,'executableSha256':sha(exe),'outputSha256':sha(output),'biomes':len(data['biomes']),'placedFeatures':len(data['placed_features']),'configuredFeatures':len(data['configured_features']),'inheritedTags':len(data['biome_tags'])})
    if outputs[0]!=outputs[1]:raise ValueError('Resource derivation differs between profiles')
    unchanged=all(sha(paths[n])==digest for n,digest in before.items())
    if not unchanged:raise ValueError('Resource inputs changed')
    report={'schemaVersion':1,'status':'passed-installed-resource-transform-smoke-not-runtime-integration','completedAtUtc':dt.datetime.now(dt.timezone.utc).isoformat(),'resourceInputsSha256':before,'invalidUnusedTags':invalid_tags,'verifierSourceSha256':sha(work/'verifier-source.py'),'preparedInputSha256':sha(inputs),'probeSourceSha256':sha(ROOT/'horizon/native-adapter/src/bin/classic_biome_data_probe.rs'),'transformSourceSha256':sha(ROOT/'horizon/native-adapter/src/classic_biome_data.rs'),'runs':runs,'profilesAgree':True,'resourceInputsUnchanged':unchanged,'gameAssetsDownloaded':False,'requiredCustomAlgorithms':data['required_custom_algorithms'],'notCovered':['Independent Java definition/feature parity','Native registry and renderer installation','Native custom feature execution','Natural biome generation or stored chunks','Client, launcher, performance and modpack acceptance']}
    a.report.parent.mkdir(parents=True,exist_ok=True);a.report.write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8',newline='\n');print(json.dumps(report,indent=2))
if __name__=='__main__':main()
