"""Check immutable archives with the pinned engine's actual mod readers.

Uses source and Rust dependencies only. Does not install a mod or launch the game.
"""
from __future__ import annotations
import argparse, datetime as dt, hashlib, json, os, shutil, subprocess, zipfile
from pathlib import Path
ROOT = Path(__file__).resolve().parents[2]

def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

def main():
    p = argparse.ArgumentParser(description=__doc__)
    for name in ('workspace', 'original', 'package', 'report'):
        p.add_argument('--' + name, type=Path, required=True)
    a = p.parse_args()
    workspace = a.workspace.resolve()
    if not workspace.is_relative_to((ROOT/'build/qa/horizon').resolve()) or workspace.name != 'staged':
        p.error('Use a fresh full-source stage below build/qa/horizon')
    stage_file = workspace.parent/'stage-result.json'
    stage = json.loads(stage_file.read_text(encoding='utf-8'))
    if not stage.get('fullEngineSourceIncluded') or not stage['automaticWorldSelectionIncluded']:
        p.error('Full entry-mode stage required')
    if a.report.exists():
        p.error('Refusing to replace evidence')
    for name, digest in stage['inputFiles'].items():
        if sha(ROOT/name) != digest:
            raise ValueError('Restage changed source ' + name)
    # The archive's executable sources must equal the already staged bundle.
    # Additional notices are harmless but every source/config file is checked.
    bundle = workspace.parent/'mods/terraforged_native'
    with zipfile.ZipFile(a.package) as archive:
        for source in sorted(bundle.rglob('*')):
            if source.is_file() and archive.read('mixins/terraforged_native/'+source.relative_to(bundle).as_posix()) != source.read_bytes():
                raise ValueError('Packaged source differs: ' + str(source))
    driver = ROOT/'horizon/tests/rust/native_package_probe.rs'
    binary = 'terraforged_package_probe_' + sha(driver)[:12]
    dest = workspace/f'crates/horizon-mods/src/bin/{binary}.rs'
    if dest.exists() and dest.read_bytes() != driver.read_bytes():
        raise ValueError('Existing probe source differs; fresh workspace required')
    dest.parent.mkdir(parents=True, exist_ok=True)
    if not dest.exists():
        shutil.copy2(driver, dest)
    output = workspace.parent/a.report.stem
    output.mkdir(exist_ok=False)
    mismatch = output/'mismatched-version.hmod'
    with zipfile.ZipFile(a.package) as archive, zipfile.ZipFile(mismatch, 'x', compression=zipfile.ZIP_DEFLATED) as changed:
        for name in archive.namelist():
            data = archive.read(name)
            if name == 'horizon-native.json':
                manifest = json.loads(data)
                manifest['version'] += '-mismatch'
                data = json.dumps(manifest).encode()
            changed.writestr(name, data)
    env = dict(os.environ, CARGO_BUILD_JOBS='4', CARGO_NET_OFFLINE='true')
    command = ['cargo', '+stable', 'build', '--quiet', '--offline', '--locked', '--manifest-path', str(workspace/'Cargo.toml'), '-p', 'horizon-mods', '--features', 'index', '--bin', binary]
    build = subprocess.run(command, capture_output=True, text=True, env=env, timeout=600)
    (output/'build.log').write_text(build.stdout+build.stderr, encoding='utf-8')
    report = {'schemaVersion': 1, 'status': 'failed', 'startedAtUtc': dt.datetime.now(dt.timezone.utc).isoformat(),
              'horizonRevision': stage['horizonRevision'], 'stageResultSha256': sha(stage_file),
              'driverSha256': sha(driver), 'compileExitCode': build.returncode,
              'originalSha256': sha(a.original), 'packageSha256': sha(a.package),
              'packagedSourcesMatchStagedBundle': True, 'gameAssetsReadOrDownloaded': False,
              'notCovered': ['Native build via launcher', 'Client rendering', 'Save-file round trip', 'Biome/features/structures/spawn/performance/modpack acceptance']}
    if build.returncode == 0:
        exe = workspace/'target/debug'/(binary + ('.exe' if os.name == 'nt' else ''))
        run = subprocess.run([str(exe), str(a.original.resolve()), str(a.package.resolve()), str(mismatch)], capture_output=True, text=True, env=env, timeout=60)
        (output/'stdout.json').write_text(run.stdout, encoding='utf-8', newline='\n')
        (output/'stderr.log').write_text(run.stderr, encoding='utf-8')
        report.update(executableSha256=sha(exe), exitCode=run.returncode)
        if run.returncode == 0 and 'NATIVE_PACKAGE_READERS_OK' in run.stderr:
            report['observations'] = json.loads(run.stdout)
            report['status'] = 'passed-native-package-readers'
    report['completedAtUtc'] = dt.datetime.now(dt.timezone.utc).isoformat()
    a.report.parent.mkdir(parents=True, exist_ok=True)
    a.report.write_text(json.dumps(report, indent=2)+'\n', encoding='utf-8', newline='\n')
    print(json.dumps(report, indent=2))
    return 0 if report['status'] == 'passed-native-package-readers' else 1

if __name__ == '__main__':
    raise SystemExit(main())
