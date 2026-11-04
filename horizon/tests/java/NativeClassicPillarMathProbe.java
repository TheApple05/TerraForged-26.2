import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import raccoonman.reterraforged.world.worldgen.feature.ClassicPillarMath;
/** Executes the unmodified project Java source; no game installation is required. */
public final class NativeClassicPillarMathProbe {
    private static float number(String bits) { return Float.intBitsToFloat(Integer.parseUnsignedInt(bits,16)); }
    private static String bits(float value) { return Float.isNaN(value) ? "nan" : String.format("%08x",Float.floatToRawIntBits(value)); }
    public static void main(String[] args) throws Exception {
        Path expected=Path.of(args[2]).toRealPath();
        Path origin=Path.of(ClassicPillarMath.class.getProtectionDomain().getCodeSource().getLocation().toURI()).toRealPath();
        if (!origin.equals(expected)) throw new AssertionError("Unexpected math class origin "+origin);
        StringBuilder values=new StringBuilder(); int count=0;
        for(String line:Files.readAllLines(Path.of(args[0]),StandardCharsets.UTF_8)) {
            String[] f=line.split("\\t");
            String value=switch(f[1]) {
                case "terrace" -> bits(ClassicPillarMath.terrace(number(f[2]),number(f[3])));
                case "mask" -> bits(ClassicPillarMath.mask(number(f[2]),number(f[3]),number(f[4])));
                case "height" -> Integer.toString(ClassicPillarMath.addedHeight(number(f[2]),number(f[3]),f[4].equals("1")));
                default -> throw new IllegalArgumentException("Unknown operation");
            };
            values.append(f[0]).append('\t').append(value).append('\n'); count++;
        }
        Files.writeString(Path.of(args[1]),values,StandardCharsets.UTF_8);
        System.out.println("PILLAR_MATH_CAPTURE_OK records="+count+" pid="+ProcessHandle.current().pid());
    }
}
