"""Measure the exact optimized native kernel executable, not game/chunk/FPS performance."""
from __future__ import annotations
import argparse,ctypes,datetime as dt,hashlib,json,os,platform,statistics,subprocess,time
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
def sha(path):return hashlib.sha256(path.read_bytes()).hexdigest()
def memory_reader(pid):
    if os.name!='nt':
        def read():
            try:return max(int(line.split()[1])*1024 for line in Path(f'/proc/{pid}/status').read_text().splitlines() if line.startswith(('VmHWM:','VmRSS:')))
            except (OSError,ValueError):return None
        return read,lambda:None,'Linux VmHWM/VmRSS'
    from ctypes import wintypes as w
    class Counters(ctypes.Structure):
        _fields_=[('cb',w.DWORD),('faults',w.DWORD)]+[(n,ctypes.c_size_t) for n in ('peak','rss','peakPaged','paged','peakNonPaged','nonPaged','pagefile','peakPagefile','private')]
    kernel=ctypes.WinDLL('kernel32',use_last_error=True);api=ctypes.WinDLL('psapi',use_last_error=True)
    kernel.OpenProcess.argtypes=[w.DWORD,w.BOOL,w.DWORD];kernel.OpenProcess.restype=w.HANDLE
    kernel.CloseHandle.argtypes=[w.HANDLE];kernel.CloseHandle.restype=w.BOOL
    api.GetProcessMemoryInfo.argtypes=[w.HANDLE,ctypes.POINTER(Counters),w.DWORD];api.GetProcessMemoryInfo.restype=w.BOOL
    handle=kernel.OpenProcess(0x410,False,pid)
    if not handle:raise OSError(ctypes.get_last_error(),'Cannot observe owned benchmark process')
    def read():
        counters=Counters();counters.cb=ctypes.sizeof(counters)
        return counters.peak if api.GetProcessMemoryInfo(handle,ctypes.byref(counters),counters.cb) else None
    return read,lambda:kernel.CloseHandle(handle),'Windows PeakWorkingSetSize'
def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--report',type=Path,required=True);args=p.parse_args()
    if args.report.exists():p.error('Refusing to replace native benchmark evidence')
    manifest=ROOT/'horizon/native-adapter/Cargo.toml';folder=ROOT/'horizon/tests/fixtures/kernel-v1'
    inputs=[folder/'preset.json',folder/'noise-registry.json',ROOT/'horizon/tests/inputs/kernel-bench-v1.json']
    subprocess.run(['cargo','+stable','build','--quiet','--offline','--locked','--release','--manifest-path',str(manifest),'--bin','kernel_sampler_bench'],cwd=ROOT,check=True)
    metadata=json.loads(subprocess.check_output(['cargo','+stable','metadata','--offline','--locked','--no-deps','--format-version','1','--manifest-path',str(manifest)],cwd=ROOT,text=True))
    executable=Path(metadata['target_directory'])/'release'/('kernel_sampler_bench.exe' if os.name=='nt' else 'kernel_sampler_bench')
    work=ROOT/'build/qa/horizon/kernel-bench'/args.report.stem;work.mkdir(parents=True,exist_ok=True)
    report={'schemaVersion':1,'status':'failed','startedAtUtc':dt.datetime.now(dt.timezone.utc).isoformat(),'executableSha256':sha(executable),
        'inputSha256':{path.relative_to(ROOT).as_posix():sha(path) for path in inputs},'rustc':subprocess.check_output(['rustc','+stable','-Vv'],text=True),
        'host':{'os':platform.system(),'osVersion':platform.version(),'architecture':platform.machine(),'cpu':platform.processor(),'logicalProcessors':os.cpu_count()},
        'scope':'Optimized procedural cell/tile sampler only. Cold timings include tile generation/filtering and parallel thread startup; warm timings include a 24-field checksum. No native density, game chunks, client, save or FPS acceptance.'}
    proc=subprocess.Popen([str(executable),*map(str,inputs)],cwd=ROOT,stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True)
    read,close,metric=memory_reader(proc.pid);peak=0;started=time.monotonic();timed_out=False
    try:
        while proc.poll() is None:
            peak=max(peak,read() or 0)
            if time.monotonic()-started>300:proc.kill();timed_out=True;break
            time.sleep(0.05)
        stdout,stderr=proc.communicate(timeout=10);peak=max(peak,read() or 0)
    finally:close()
    (work/'stdout.json').write_text(stdout,encoding='utf-8');(work/'stderr.log').write_text(stderr,encoding='utf-8')
    report.update(pid=proc.pid,exitCode=proc.returncode,timedOut=timed_out,processPeakResidentBytes=peak,memoryMetric=metric,
                  executableUnchanged=sha(executable)==report['executableSha256'],stdoutSha256=sha(work/'stdout.json'),stderrSha256=sha(work/'stderr.log'))
    if proc.returncode==0 and not timed_out:
        data=json.loads(stdout);report['measurement']=data
        if data['status']=='measured-native-kernel-only' and data['allChecksumsAgree'] and report['executableUnchanged'] and peak>0:
            report['status']='measured-native-kernel-only';report['medians']={}
            for mode in ('serial','parallel'):
                values=[v for v in data['observations'] if v['mode']==mode]
                if len(values)!=3:raise ValueError('Three trials per mode required')
                cold=statistics.median(v['coldSeconds'] for v in values);warm=statistics.median(v['warmSecondsIncludingChecksum'] for v in values)
                report['medians'][mode]={'coldFourTilesSeconds':cold,'coldMillisecondsPerTile':cold*250,'warm65536ColumnsSecondsIncludingChecksum':warm}
    report['sourceSha256Lf']={p.relative_to(ROOT).as_posix():hashlib.sha256(p.read_bytes().replace(b'\r\n',b'\n')).hexdigest()
        for crate in ('native-core','native-adapter') for p in sorted((ROOT/'horizon'/crate/'src').rglob('*.rs'))}
    report['verifierSha256']=sha(Path(__file__));report['finishedAtUtc']=dt.datetime.now(dt.timezone.utc).isoformat()
    args.report.parent.mkdir(parents=True,exist_ok=True);args.report.write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8')
    print(json.dumps({k:report[k] for k in ('status','processPeakResidentBytes','medians') if k in report}))
    return 0 if report['status']=='measured-native-kernel-only' else 1
if __name__=='__main__':raise SystemExit(main())
