"""Run observable publication/concurrency/ownership cache tests in both profiles."""
from __future__ import annotations
import argparse,datetime as dt,hashlib,json,re,subprocess
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
def sha(path):return hashlib.sha256(path.read_bytes()).hexdigest()
def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--report',type=Path,required=True);args=p.parse_args()
    if args.report.exists():p.error('Refusing to replace cache lifecycle evidence')
    work=ROOT/'build/qa/horizon/tile-cache'/args.report.stem;work.mkdir(parents=True,exist_ok=True)
    report={'schemaVersion':1,'status':'failed','startedAtUtc':dt.datetime.now(dt.timezone.utc).isoformat(),
        'scope':'Observable completed-tile publication, failure/unwind retry, LRU ownership, concurrent identity, in-flight clear and bounded parallel slots. Controlled builders isolate cache behavior; independent Java column/tile comparisons qualify terrain.',
        'rustc':subprocess.check_output(['rustc','+stable','-Vv'],text=True),'profiles':[]}
    for profile in ('debug','release'):
        command=['cargo','+stable','test','--offline','--locked','--manifest-path',str(ROOT/'horizon/native-core/Cargo.toml'),'--lib']
        if profile=='release':command.append('--release')
        run=subprocess.run(command,cwd=ROOT,capture_output=True,text=True,timeout=60)
        output=work/(profile+'.stdout.log');error=work/(profile+'.stderr.log');output.write_text(run.stdout,encoding='utf-8');error.write_text(run.stderr,encoding='utf-8')
        tests=re.findall(r'^test tile_work_cache::tests::([^ ]+) \.\.\. ok$',run.stdout,re.M)
        passed=run.returncode==0 and len(tests)==5 and '5 passed; 0 failed; 0 ignored;' in run.stdout
        report['profiles'].append({'profile':profile,'exitCode':run.returncode,'passed':passed,'testNames':tests,'stdoutSha256':sha(output),'stderrSha256':sha(error)})
    report['status']='passed-tile-cache-lifecycle-only' if all(p['passed'] for p in report['profiles']) else 'failed'
    report['sourceSha256Lf']={name:hashlib.sha256((ROOT/name).read_bytes().replace(b'\r\n',b'\n')).hexdigest() for name in ['horizon/native-core/src/tile_work_cache.rs','horizon/native-core/src/kernel_sampler.rs','horizon/native-core/Cargo.toml','horizon/native-core/Cargo.lock']}
    report['verifierSha256']=sha(Path(__file__));report['finishedAtUtc']=dt.datetime.now(dt.timezone.utc).isoformat()
    args.report.parent.mkdir(parents=True,exist_ok=True);args.report.write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8')
    print(json.dumps({'status':report['status'],'testsPerProfile':5}));return 0 if report['status']=='passed-tile-cache-lifecycle-only' else 1
if __name__=='__main__':raise SystemExit(main())
