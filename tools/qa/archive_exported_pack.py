"""Create a standard ZIP datapack and prove every member matches a qualified QA export."""
from __future__ import annotations
import argparse
import hashlib
import json
from pathlib import Path
import zipfile


def sha(data): return hashlib.sha256(data).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--export-report", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--report", type=Path, required=True)
    args = parser.parse_args()
    raw = json.loads(args.export_report.read_text())
    qa = raw.get("qaResult", raw.get("runtimeQa")) or {}
    if raw.get("result", raw.get("status")) != "passed" or qa.get("status") != "passed":
        raise ValueError("A passing actual export report is required")
    source_text = qa["export"]["presetPack"]
    # Reports made in WSL may be consumed by the host archiver, and vice versa.
    if source_text.startswith("/mnt/c/") and Path.cwd().drive:
        source_text = "C:/" + source_text[len("/mnt/c/"):]
    source = Path(source_text)
    expected = qa["export"]["fileSha256"]
    files = {path.relative_to(source).as_posix(): path for path in source.rglob("*") if path.is_file()}
    actual = {name: sha(path.read_bytes()) for name, path in files.items()}
    if actual != expected: raise ValueError("Source pack changed after qualified export")
    if args.output.exists(): raise ValueError("Refusing to overwrite existing datapack archive")
    args.output.parent.mkdir(parents=True, exist_ok=True)
    with zipfile.ZipFile(args.output, "w", compression=zipfile.ZIP_DEFLATED) as archive:
        for name, path in sorted(files.items()): archive.write(path, name)
    with zipfile.ZipFile(args.output) as archive:
        zipped = {name: sha(archive.read(name)) for name in archive.namelist()}
    if zipped != expected: raise ValueError("Archived pack differs from qualified export")
    report = {"schemaVersion": 1, "status": "passed", "sourceExportReport": str(args.export_report),
        "sourceExportReportSha256": sha(args.export_report.read_bytes()), "zipPath": str(args.output.resolve()),
        "zipSha256": sha(args.output.read_bytes()), "files": len(files), "everyMemberMatchesQualifiedExport": True,
        "scope": "Packaging equivalence; requires separate native game load/generation evidence"}
    args.report.parent.mkdir(parents=True, exist_ok=True); args.report.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(report, indent=2))


if __name__ == "__main__": main()
