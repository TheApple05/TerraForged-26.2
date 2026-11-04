"""Run the real language provider and check or update the packaged English asset."""
from __future__ import annotations
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]
TARGET = ROOT / "common/src/main/resources/assets/reterraforged/lang/en_us.json"


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--java-home", type=Path, required=True)
    parser.add_argument("--write", action="store_true", help="Update the checked-in generated asset")
    args = parser.parse_args()
    classpath = str(ROOT / "build/classes/java/main") + os.pathsep + (
        ROOT / "build/qa/target/compile-classpath.txt").read_text().strip()
    java = args.java_home / "bin/java.exe" if os.name == "nt" else args.java_home / "bin/java"
    javac = args.java_home / "bin/javac.exe" if os.name == "nt" else args.java_home / "bin/javac"
    base = ROOT / "build/qa/language"
    base.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="generate-", dir=base) as temp:
        work = Path(temp)
        subprocess.run([str(javac), "--release", "25", "-proc:none", "-encoding", "UTF-8",
                        "-cp", classpath, "-d", temp, str(ROOT / "tests/resources/LanguageAssetGenerator.java")],
                       check=True, cwd=ROOT, timeout=60)
        subprocess.run([str(java), "-cp", temp + os.pathsep + classpath,
                        "LanguageAssetGenerator", str(work / "output")], check=True, cwd=ROOT, timeout=60)
        generated = work / "output/assets/reterraforged/lang/en_us.json"
        translations = json.loads(generated.read_text(encoding="utf-8"))
        if not translations or translations.get("reterraforged.gui.worldSettings.title") != "World Settings":
            raise ValueError("Expected production language output was not generated")
        canonical = json.dumps(translations, ensure_ascii=False, indent=2, sort_keys=True) + "\n"
        if args.write:
            TARGET.parent.mkdir(parents=True, exist_ok=True)
            TARGET.write_text(canonical, encoding="utf-8", newline="\n")
        elif not TARGET.is_file() or json.loads(TARGET.read_text(encoding="utf-8")) != translations:
            raise ValueError("Packaged English asset differs from production provider; rerun with --write")
        print(json.dumps({"result": "written" if args.write else "matched", "translationCount": len(translations),
                          "asset": TARGET.relative_to(ROOT).as_posix(),
                          "sha256": hashlib.sha256(TARGET.read_bytes()).hexdigest()}))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
