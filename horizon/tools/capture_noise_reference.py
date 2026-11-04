"""Freeze noise results from genuine production Java classes, independently of Rust."""
from __future__ import annotations
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[2]


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--java-home', type=Path, required=True)
    parser.add_argument('--jar', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--family', choices=['base','second'], default='base')
    args = parser.parse_args()
    metadata = args.output.with_suffix('.json')
    if args.output.exists() or metadata.exists():
        parser.error('Refusing to replace a frozen oracle')
    modules = ['Perlin','Simplex'] if args.family=='base' else ['Perlin2','Simplex2']
    probe = 'NativeNoiseProbe' if args.family=='base' else 'NativeNoiseFamilyProbe'
    source = ROOT/('horizon/tests/java/'+probe+'.java')
    jar = args.jar.resolve()
    cp = str(jar) + os.pathsep + (ROOT/'build/qa/target/compile-classpath.txt').read_text().strip()
    suffix = '.exe' if os.name == 'nt' else ''
    workroot = ROOT/'build/qa/horizon/noise-reference'
    workroot.mkdir(parents=True, exist_ok=True)
    captures = []
    with tempfile.TemporaryDirectory(dir=workroot) as temporary:
        compiled = subprocess.run([str(args.java_home/'bin'/('javac'+suffix)), '--release','25','-proc:none',
            '-cp',cp,'-d',temporary,str(source)], cwd=ROOT, capture_output=True, text=True, timeout=90)
        if compiled.returncode:
            raise RuntimeError(compiled.stdout + compiled.stderr)
        for index in range(2):
            log = workroot/f'class-load-{index}.log'
            result = subprocess.run([str(args.java_home/'bin'/('java'+suffix)),
                '-Xlog:class+load=info:file='+log.relative_to(ROOT).as_posix(),
                '-cp',temporary+os.pathsep+cp,
                'raccoonman.reterraforged.world.worldgen.noise.module.'+probe],
                cwd=ROOT, capture_output=True, timeout=90)
            if result.returncode:
                raise RuntimeError(result.stderr.decode(errors='replace'))
            text = log.read_text()
            for name in ['NoiseUtil']+['module.'+name for name in modules]:
                expected = 'raccoonman.reterraforged.world.worldgen.noise.'+name+' source: '
                lines = [line for line in text.splitlines() if expected in line]
                if len(lines) != 1 or jar.name not in lines[0]:
                    raise ValueError('Genuine package origin missing: '+name)
            captures.append(result.stdout.replace(b'\r\n',b'\n'))
    if captures[0] != captures[1]:
        raise ValueError('Independent fresh Java runs differed')
    lines = captures[0].splitlines()
    keys = [line.split(b'\t',1)[0] for line in lines]
    if len(set(keys)) != len(keys):
        raise ValueError('Duplicate reference keys')
    args.output.parent.mkdir(parents=True,exist_ok=True)
    args.output.write_bytes(captures[0])
    classes = ['NoiseUtil.java']+['module/'+name+'.java' for name in modules]+['function/Interpolation.java']
    srcroot = ROOT/'common/src/main/java/raccoonman/reterraforged/world/worldgen/noise'
    record = {'schemaVersion':1,'status':'frozen-java-reference','productionJarSha256':digest(jar),
        'sourceSha256Lf':{name:hashlib.sha256((srcroot/name).read_bytes().replace(b'\r\n',b'\n')).hexdigest() for name in classes},
        'oracleSourceSha256Lf':hashlib.sha256(source.read_bytes().replace(b'\r\n',b'\n')).hexdigest(),
        'fixtureSha256':digest(args.output),'recordCount':len(lines),'freshRunsAgree':True,
        'scope':'296 literal vectors and '+','.join(modules)+' static/fractal; independent of native output.',
        'captureJava':subprocess.check_output([str(args.java_home/'bin'/('java'+suffix)),'-version'],stderr=subprocess.STDOUT,text=True)}
    metadata.write_text(json.dumps(record,indent=2)+'\n',encoding='utf-8')
    print(json.dumps(record))


if __name__ == '__main__':
    main()
