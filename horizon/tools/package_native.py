"""Package the staged procedural source bundle and its own preset as a QA hmod.

This artifact excludes Minecraft resources, terrain snapshots and Java/runtime
test binaries. A package is not evidence of launcher or player acceptance.
"""
from __future__ import annotations
import argparse,hashlib,json,tomllib,zipfile
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
MIXINS='e8cd0aa38cffef49cb2e088ef716baa107148c02'
def sha(data):return hashlib.sha256(data).hexdigest()
def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--stage',required=True,type=Path);p.add_argument('--output',required=True,type=Path);a=p.parse_args()
    stage=a.stage.resolve();output=a.output.resolve()
    if not stage.is_relative_to((ROOT/'build/qa/horizon').resolve()):p.error('Use a stage below build/qa/horizon')
    if not output.is_relative_to((ROOT/'build/distributions').resolve()) or output.exists() or output.with_suffix(output.suffix+'.json').exists():p.error('Choose a fresh output below build/distributions')
    raw=(stage/'stage-result.json').read_bytes();r=json.loads(raw)
    if not r['automaticWorldSelectionIncluded']:raise ValueError('Entry-mode stage required')
    for name,digest in r['inputFiles'].items():
        if sha((ROOT/name).read_bytes())!=digest:raise ValueError('Restage changed input '+name)
    bundle=stage/'mods/terraforged_native';manifest=tomllib.loads((bundle/'horizon-mixin.toml').read_text(encoding='utf-8'))
    if manifest['mod']['id']!='terraforged_native' or manifest['compatibility']['horizon_revision']!=r['horizonRevision']:raise ValueError('Wrong source bundle')
    entries={}
    for path in sorted(bundle.rglob('*')):
        if path.is_symlink():raise ValueError('Symlinked package source')
        if path.is_file():
            name=path.relative_to(bundle).as_posix()
            if any(part in ('target','.git','.local','node_modules') for part in path.relative_to(bundle).parts) or path.name=='Cargo.lock':raise ValueError('Unexpected build/runtime artifact')
            entries['mixins/terraforged_native/'+name]=path.read_bytes()
    for file in sorted((ROOT/'horizon/native-content').rglob('*')):
        if file.is_file():
            entries[file.relative_to(ROOT/'horizon/native-content').as_posix()]=file.read_bytes()
    version=manifest['mod']['version']
    native={'schema':1,'id':'terraforged_native','version':version,'game':'26.3','horizon':r['horizonRevision'],'mixins':MIXINS}
    entries['horizon-native.json']=(json.dumps(native,indent=2)+'\n').encode()
    entries['horizon-mod.toml']=('''id = "terraforged_native"
name = "TerraForged native prototype"
version = "'''+version+'''"
description = "Experimental procedural TerraForged terrain for the pinned test engine."
authors = ["TerraForged Revival contributors"]
api = "0.2"
side = "both"
capabilities = ["registries"]
namespaces = ["terraforged", "reterraforged"]
''').encode()
    entries['mixins/terraforged_native/LICENSE-HORIZON']=(stage/'pristine/LICENSE').read_bytes()
    for original,destination in [('licenses/ReTerraForged-MIT.txt','LICENSE-RETERRAFORGED'),('licenses/TerraForged-MIT.txt','LICENSE-TERRAFORGED'),('horizon/LICENSE','LICENSE')]:
        entries['mixins/terraforged_native/'+destination]=(ROOT/original).read_bytes()
    entries['mixins/terraforged_native/README-TERRAFORGED.txt']=('TerraForged native prototype for Horizon 26.3\n\nRequires Horizon source '+r['horizonRevision']+' from TheApple05/horizon. Standard launcher installation and player acceptance remain unverified.\nSelect terraforged:default for the native overworld. Vanilla, Nether and End keep their engine behavior.\nNative classic biome/features, source carver settings and save/client/performance/modpack acceptance are unfinished.\nThis source package contains no Minecraft files or terrain snapshots.\n').encode()
    output.parent.mkdir(parents=True,exist_ok=True)
    with zipfile.ZipFile(output,'w',compression=zipfile.ZIP_DEFLATED,compresslevel=9) as z:
        for name,data in sorted(entries.items()):
            info=zipfile.ZipInfo(name,(1980,1,1,0,0,0));info.compress_type=zipfile.ZIP_DEFLATED;info.external_attr=0o100644<<16;z.writestr(info,data)
    with zipfile.ZipFile(output) as z:
        if z.namelist()!=sorted(entries) or any(z.read(name)!=data for name,data in entries.items()):raise ValueError('Packaged bytes differ')
    report={'schemaVersion':1,'status':'packaged-procedural-native-prototype-not-player-qualified','artifact':str(output),'sha256':sha(output.read_bytes()),
        'stageResultSha256':sha(raw),'manifest':native,'entrySha256':{n:sha(b) for n,b in sorted(entries.items())},
        'containsMinecraftResources':False,'containsTerrainSnapshots':False,'containsJavaOrQaRuntimeBinaries':False,
        'notCovered':['Actual hmod loader installation','Native client rendering','Full native biome/feature restoration','Save/modpack/player acceptance']}
    output.with_suffix(output.suffix+'.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8',newline='\n')
    print(json.dumps({k:v for k,v in report.items() if k!='entrySha256'},indent=2))
    return 0
if __name__=='__main__':raise SystemExit(main())
