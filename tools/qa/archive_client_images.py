"""Archive byte-exact native screenshots with their production report provenance."""
from __future__ import annotations
import argparse
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def sha(data):
    return hashlib.sha256(data).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--report', type=Path, action='append', required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    if args.output.exists():
        parser.error('Refusing to overwrite an image archive manifest')
    records = []
    for path in args.report:
        path = path.resolve()
        original = path.read_bytes()
        report = json.loads(original)
        client = report.get('client', report)
        qa = client['clientQa']
        if report.get('status', report.get('result')) not in {'passed', 'passed-client-qa-awaiting-visual-review'} or qa['status'] != 'passed':
            raise ValueError('Only passed production captures may enter the archive: ' + str(path))
        if client['processExitCode'] != 0 or not client['artifactUnchanged']:
            raise ValueError('Native client did not exit cleanly with unchanged artifact')
        images = []
        for row in qa['screenshots']:
            source = Path(row['path']).resolve()
            if not source.is_relative_to(ROOT.resolve()):
                raise ValueError('Capture escaped workspace')
            data = source.read_bytes()
            if sha(data) != row['sha256'] or len(data) != row['bytes']:
                raise ValueError('Capture bytes changed: ' + str(source))
            target = ROOT / 'docs/validation/images' / path.stem / source.name
            if target.exists() and target.read_bytes() != data:
                raise ValueError('Refusing to replace an archived image')
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes(data)
            images.append({'path': target.relative_to(ROOT).as_posix(), 'sha256': sha(data),
                           'bytes': len(data), 'width': row['width'], 'height': row['height']})
        if not images:
            raise ValueError('No positive native screenshot captures')
        records.append({'report': path.relative_to(ROOT).as_posix(), 'reportSha256': sha(original), 'images': images})
    result = {'schemaVersion': 1, 'status': 'archived-byte-exact', 'reports': records,
              'imageCount': sum(len(row['images']) for row in records),
              'scope': 'Actual native Screenshot.grab output; archival checks do not establish visual approval.'}
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(result, indent=2) + '\n', encoding='utf-8')
    print(json.dumps({'status': result['status'], 'imageCount': result['imageCount']}))


if __name__ == '__main__':
    main()
