"""Capture a scalar oracle from pinned upstream LegacyTerrace methods, never the port."""
from pathlib import Path
import hashlib
import json
import subprocess
import argparse

ROOT = Path(__file__).resolve().parents[2]
REVISION = "a3c7f2d9e229b54b15783f8a202eb792ea942d39"
SOURCE = "src/main/java/com/terraforged/noise/modifier/LegacyTerrace.java"
UTIL = "src/main/java/com/terraforged/noise/util/NoiseUtil.java"


def body(text, prefix):
    start = text.index(prefix)
    brace = text.index("{", start)
    depth = 1
    end = brace + 1
    while depth:
        depth += (text[end] == "{") - (text[end] == "}")
        end += 1
    return text[start:end]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--java-home", type=Path, required=True)
    parser.add_argument("--reference-repo", type=Path, default=ROOT / ".reference/Noise2D")
    args = parser.parse_args()
    def read(path):
        return subprocess.run(["git", "-C", str(args.reference_repo), "show", REVISION + ":" + path], capture_output=True, text=True, check=True).stdout
    source, util = read(SOURCE), read(UTIL)
    code = source[source.index("public class LegacyTerrace"):source.index("    @Override\n    public boolean equals")]
    code = code.replace("public class LegacyTerrace", "class LegacyTerrace")
    code += body(source, "    private static class Step") + "\n}\n"
    code += '''
interface Module {
    float getValue(int seed, float x, float y);
    default float minValue() { return 0; }
    default float maxValue() { return 1; }
}
abstract class Modifier implements Module {
    protected final Module source;
    Modifier(Module source) { this.source = source; }
    public abstract String getSpecName();
    public abstract float modify(int seed, float x, float y, float value);
}
enum Interpolation {
    CURVE3;
    float apply(float f) { return NoiseUtil.interpHermite(f); }
}
class NoiseUtil {
'''
    for method in ["public static float clamp(", "public static int round(", "public static float interpHermite("]:
        code += body(util, method) + "\n"
    code += '''}
public class OracleMain {
    public static void main(String[] args) throws Exception {
        var digest = java.security.MessageDigest.getInstance("SHA-256");
        for (float maximum : new float[] {1.0F, 0.999F}) {
            for (int i = -8; i <= 8200; i++) {
                final float value = i / 8192.0F;
                Module source = new Module() {
                    public float getValue(int s, float x, float y) { return value; }
                    public float maxValue() { return maximum; }
                };
                float result = new LegacyTerrace(source, (s,x,y) -> 1, (s,x,y) -> 0.25F, 4, 0.25F).getValue(0,0,0);
                digest.update(java.nio.ByteBuffer.allocate(4).putInt(Float.floatToRawIntBits(result)).array());
            }
        }
        System.out.println(java.util.HexFormat.of().formatHex(digest.digest()));
    }
}
'''
    base = ROOT / "build/qa/classic-pillar-oracle"
    base.mkdir(parents=True, exist_ok=True)
    (base / "OracleMain.java").write_text(code, encoding="utf-8")
    subprocess.run([str(args.java_home / "bin/javac.exe"), "-d", str(base), str(base / "OracleMain.java")], check=True)
    result = subprocess.run([str(args.java_home / "bin/java.exe"), "-cp", str(base), "OracleMain"], capture_output=True, text=True, check=True).stdout.strip()
    report = {"upstream": "https://github.com/TerraForged/Noise2D", "revision": REVISION,
              "source": SOURCE, "sourceSha256Lf": hashlib.sha256(source.encode()).hexdigest(),
              "utilitySourceSha256Lf": hashlib.sha256(util.encode()).hexdigest(),
              "projectionSha256Lf": hashlib.sha256(code.encode()).hexdigest(), "terraceRawBitsSha256": result,
              "records": 16418, "order": "maxima[1.0f,0.999f]; i=-8..8200; input=i/8192.0f; raw IEEE754 float32 bits big endian",
              "scope": "Original constructor/getValue/modify/Step and original clamp/round/interpHermite compiled unmodified; serialization/equality removed and external module/curve interfaces projected to minimal dev-only adapters."}
    (ROOT / "tests/terrain/classic-pillar-reference.json").write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(report, indent=2))


if __name__ == "__main__":
    main()
