"""Sample only a harness-owned process; report native resident memory, not heap estimates."""
from __future__ import annotations

import os
from pathlib import Path
import threading


class ProcessMetrics:
    def __init__(self, pid: int):
        self.pid = pid
        self.peak_rss = 0
        self.samples = 0
        self.errors = []
        self.stop_event = threading.Event()
        self.thread = threading.Thread(target=self._sample, daemon=True)
        self.thread.start()

    def _sample(self):
        if os.name == "nt":
            import ctypes
            from ctypes import wintypes

            class Counters(ctypes.Structure):
                _fields_ = [("cb", wintypes.DWORD), ("PageFaultCount", wintypes.DWORD)] + [
                    (name, ctypes.c_size_t) for name in ("PeakWorkingSetSize", "WorkingSetSize",
                    "QuotaPeakPagedPoolUsage", "QuotaPagedPoolUsage", "QuotaPeakNonPagedPoolUsage",
                    "QuotaNonPagedPoolUsage", "PagefileUsage", "PeakPagefileUsage")]

            kernel = ctypes.WinDLL("kernel32", use_last_error=True)
            psapi = ctypes.WinDLL("psapi", use_last_error=True)
            kernel.OpenProcess.argtypes = [wintypes.DWORD, wintypes.BOOL, wintypes.DWORD]
            kernel.OpenProcess.restype = wintypes.HANDLE
            kernel.CloseHandle.argtypes = [wintypes.HANDLE]
            psapi.GetProcessMemoryInfo.argtypes = [wintypes.HANDLE, ctypes.POINTER(Counters), wintypes.DWORD]
            handle = kernel.OpenProcess(0x1000 | 0x0010, False, self.pid)
            if not handle:
                self.errors.append("OpenProcess failed: " + str(ctypes.get_last_error()))
                return
            try:
                while not self.stop_event.is_set():
                    counters = Counters()
                    counters.cb = ctypes.sizeof(counters)
                    if not psapi.GetProcessMemoryInfo(handle, ctypes.byref(counters), counters.cb):
                        break  # The owned process may have just exited.
                    self.peak_rss = max(self.peak_rss, counters.PeakWorkingSetSize)
                    self.samples += 1
                    self.stop_event.wait(0.1)
            finally:
                kernel.CloseHandle(handle)
        else:
            while not self.stop_event.is_set():
                try:
                    rows = (Path("/proc") / str(self.pid) / "status").read_text().splitlines()
                    high_water = next(row for row in rows if row.startswith("VmHWM:"))
                    self.peak_rss = max(self.peak_rss, int(high_water.split()[1]) * 1024)
                    self.samples += 1
                except (OSError, StopIteration, ValueError):
                    break
                self.stop_event.wait(0.1)

    def finish(self) -> dict:
        self.stop_event.set()
        self.thread.join(timeout=2)
        return {"peakResidentBytes": self.peak_rss or None, "samples": self.samples,
            "scope": "Entire owned Java process, including startup and shutdown",
            "source": "Windows PeakWorkingSetSize" if os.name == "nt" else "Linux /proc VmHWM",
            "samplingIntervalSeconds": 0.1, "errors": self.errors}
