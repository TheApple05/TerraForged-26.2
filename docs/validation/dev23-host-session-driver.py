"""Task-local scheduler: wait for compilers to finish, then run both matrices."""
import ctypes
from ctypes import wintypes as w
import datetime as dt
import json
from pathlib import Path
import subprocess
import sys
import threading
import time

ROOT = Path(__file__).resolve().parents[2]
REPORT = ROOT/'docs/validation/benchmark-dev23-host-session.json'
if REPORT.exists():
    raise ValueError('Refusing to replace host session evidence')

class Entry(ctypes.Structure):
    _fields_ = [('size',w.DWORD),('usage',w.DWORD),('pid',w.DWORD),('heap',ctypes.c_size_t),
        ('module',w.DWORD),('threads',w.DWORD),('parent',w.DWORD),('priority',w.LONG),
        ('flags',w.DWORD),('name',w.WCHAR*260)]

k = ctypes.WinDLL('kernel32',use_last_error=True)
k.CreateToolhelp32Snapshot.argtypes = [w.DWORD,w.DWORD]
k.CreateToolhelp32Snapshot.restype = w.HANDLE
k.Process32FirstW.argtypes = [w.HANDLE,ctypes.POINTER(Entry)]
k.Process32NextW.argtypes = [w.HANDLE,ctypes.POINTER(Entry)]
k.CloseHandle.argtypes = [w.HANDLE]

def heavy(include_java=False):
    names = {'cargo.exe','rustc.exe','link.exe','cl.exe','clang.exe','gcc.exe'}
    if include_java:
        names |= {'java.exe','javaw.exe'}
    handle = k.CreateToolhelp32Snapshot(2,0)
    if handle == ctypes.c_void_p(-1).value:
        raise ctypes.WinError(ctypes.get_last_error())
    result = []
    try:
        entry = Entry()
        entry.size = ctypes.sizeof(entry)
        success = k.Process32FirstW(handle,ctypes.byref(entry))
        while success:
            if entry.name.lower() in names:
                result.append({'pid':entry.pid,'name':entry.name})
            success = k.Process32NextW(handle,ctypes.byref(entry))
    finally:
        k.CloseHandle(handle)
    return result

def now():
    return dt.datetime.now(dt.timezone.utc).isoformat()

report = {'schemaVersion':1,'status':'waiting-for-quiet','startedAtUtc':now(),'preflight':[],
          'compilerInterference':[],'matrices':[],
          'scope':'No concurrent native Java/Cargo workload at launch; compiler process monitor throughout. General desktop/OS background activity is not eliminated.'}

def persist():
    REPORT.write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8')

persist()
deadline = time.monotonic()+1800
quiet = 0
while quiet < 3:
    rows = heavy(include_java=True)
    quiet = quiet+1 if not rows else 0
    report['preflight'].append({'timeUtc':now(),'processes':rows})
    persist()
    if time.monotonic()>deadline:
        report.update(status='deferred-host-busy',finishedAtUtc=now())
        persist()
        raise SystemExit(2)
    time.sleep(5)

stop = threading.Event()
def monitor():
    while not stop.wait(2):
        rows = heavy()
        if rows:
            report['compilerInterference'].append({'timeUtc':now(),'processes':rows})

thread = threading.Thread(target=monitor,daemon=True)
thread.start()
try:
    report.update(status='measuring',quietWindowStartedAtUtc=now())
    persist()
    print('QUIET_BENCHMARK_START',flush=True)
    for stack in ('base','optimized'):
        packs = {}
        for loader in ('fabric','neoforge'):
            if stack=='base':
                path = ROOT/f'docs/validation/server-{loader}-dev23-export.json'
            else:
                path = ROOT/('docs/validation/compatibility-dev23-fabric.json' if loader=='fabric' else 'docs/validation/compatibility-dev23-neoforge-optimized.json')
                row = next(row for row in json.loads(path.read_text())['cases'] if row['case']=='optimized')
                path = ROOT/row['exportReport']
            raw = json.loads(path.read_text())
            packs[loader] = raw.get('qaResult',raw.get('runtimeQa'))['export']['presetPack']
        command = [sys.executable,str(ROOT/'tools/qa/benchmark_matrix.py'),
            '--java-home','C:/Users/Main/AppData/Local/Programs/Java/jdk-25.0.5+8',
            '--fabric-jar','build/libs/terraforged-revival-0.1.0-dev.23+26.2.jar',
            '--neoforge-jar','neoforge/build/libs/terraforged-revival-0.1.0-dev.23+26.2-neoforge.jar',
            '--fabric-qa','build/qa/runtime-dev23/terraforged-runtime-qa.jar',
            '--neoforge-qa','build/qa/neoforge-runtime-dev23/terraforged-neoforge-runtime-qa.jar',
            '--fabric-pack',packs['fabric'],'--neoforge-pack',packs['neoforge'],
            '--stack',stack,'--trials','3','--label','dev23-'+stack,
            '--report','docs/validation/benchmark-dev23-'+stack+'.json']
        result = subprocess.run(command,cwd=ROOT)
        report['matrices'].append({'stack':stack,'driverExit':result.returncode,'command':command})
        persist()
        if result.returncode:
            report['status']='failed-matrix'
            break
    else:
        report['status']='passed' if not report['compilerInterference'] else 'invalid-host-interference'
finally:
    stop.set()
    thread.join(timeout=5)
    report['finishedAtUtc']=now()
    persist()
print('CONTROLLED_SESSION='+report['status'],flush=True)
raise SystemExit(0 if report['status']=='passed' else 1)
