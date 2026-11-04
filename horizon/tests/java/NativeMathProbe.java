import java.io.DataOutputStream;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import raccoonman.reterraforged.world.worldgen.noise.NoiseUtil;

/** Read-only capture of the genuine MIT NoiseUtil math table and public helpers. */
public final class NativeMathProbe {
    private static String bits(float value) { return String.format("%08x", Float.floatToRawIntBits(value)); }
    private static Object field(String name) throws Exception {
        var field = NoiseUtil.class.getDeclaredField(name);
        field.setAccessible(true);
        return field.get(null);
    }
    public static void main(String[] args) throws Exception {
        float[] table = (float[]) field("SIN");
        int count = (Integer) field("SIN_COUNT");
        if (table.length != count) throw new IllegalStateException("Sine table/count mismatch");
        try (var output = new DataOutputStream(Files.newOutputStream(Path.of(args[0])))) {
            for (float value : table) output.writeInt(Float.floatToRawIntBits(value));
        }
        StringBuilder values = new StringBuilder();
        for (int i = 0; i < count; i++) {
            float center = (i + .5f) / count * NoiseUtil.PI2;
            float[] angles = {Math.nextDown(center),center,Math.nextUp(center),-Math.nextDown(center),-center,-Math.nextUp(center)};
            for (int j = 0; j < angles.length; j++) {
                float angle = angles[j];
                values.append("trig/").append(i).append('/').append(j).append('\t')
                    .append(bits(angle)).append(',').append(bits(NoiseUtil.sin(angle))).append(',')
                    .append(bits(NoiseUtil.cos(angle))).append('\n');
            }
            float input = i / (float) (count - 1);
            values.append("sqrt/").append(i).append('\t').append(bits(input)).append(',').append(bits(NoiseUtil.sqrt(input))).append('\n');
        }
        int[] extras = {0,0x80000000,1,0x007fffff,0x00800000,0x3f800000,0x40000000,0x7f7fffff};
        for (int i = 0; i < extras.length; i++) {
            float input = Float.intBitsToFloat(extras[i]);
            values.append("extra/").append(i).append('\t').append(bits(input)).append(',')
                .append(bits(NoiseUtil.sin(input))).append(',').append(bits(NoiseUtil.cos(input))).append(',')
                .append(bits(NoiseUtil.sqrt(input))).append('\n');
        }
        Files.writeString(Path.of(args[1]), values.toString(), StandardCharsets.UTF_8);
        System.out.println("SIN_COUNT=" + count);
        System.out.println("SIN_MASK=" + field("SIN_MASK"));
        System.out.println("RAD_TO_INDEX_BITS=" + bits((Float) field("radToIndex")));
        System.out.println("PI2_BITS=" + bits(NoiseUtil.PI2));
        System.out.println("SQRT2_BITS=" + bits(NoiseUtil.SQRT2));
        System.out.println("ORIGIN=" + NoiseUtil.class.getProtectionDomain().getCodeSource().getLocation());
    }
}
