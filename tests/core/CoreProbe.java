/* Project-owned regression probe. Upstream code is compiled separately, unchanged. */
import java.util.ArrayList;
import java.util.Collections;
import java.util.List;
import java.util.Map;
import java.util.Random;
import java.util.TreeMap;
import java.util.concurrent.ConcurrentHashMap;
import raccoonman.reterraforged.world.worldgen.noise.NoiseUtil;
import raccoonman.reterraforged.world.worldgen.util.FastRandom;
import raccoonman.reterraforged.world.worldgen.util.PosUtil;
import raccoonman.reterraforged.world.worldgen.util.Seed;

public final class CoreProbe {
    private static final int[] SEEDS = {0, 1, -1, 42, Integer.MIN_VALUE, Integer.MAX_VALUE};
    private static final float[] COORDS = {
        -30_000_000f, -17f, -16.25f, -16f, -1.5f, -1f, -0.25f,
        -0.0f, 0f, 0.25f, 1f, 15f, 15.75f, 16f, 16.25f, 17f, 30_000_000f
    };

    private record Point(int seed, int xIndex, int zIndex) {}

    private static String f(float value) {
        if (!Float.isFinite(value)) throw new AssertionError("Non-finite kernel result");
        return String.format("%08x", Float.floatToRawIntBits(value));
    }

    private static void require(boolean condition, String message) {
        if (!condition) throw new AssertionError(message);
    }

    private static String noise(Point p) {
        float x = COORDS[p.xIndex], z = COORDS[p.zIndex];
        int ix = NoiseUtil.floor(x), iz = NoiseUtil.floor(z);
        float dx = x - ix, dz = z - iz;
        var cell = NoiseUtil.cell(p.seed, ix, iz);
        var gradient = NoiseUtil.coord2D(p.seed, ix, iz);
        var gradient24 = NoiseUtil.coord2D_24(p.seed, ix, iz);
        return ix + "," + iz + "," + NoiseUtil.hash(ix, iz) + ","
            + NoiseUtil.hash2D(p.seed, ix, iz) + "," + NoiseUtil.seed(ix, iz) + ","
            + f(NoiseUtil.valCoord2D(p.seed, ix, iz)) + ","
            + f(NoiseUtil.gradCoord2D(p.seed, ix, iz, dx, dz)) + ","
            + f(NoiseUtil.gradCoord2D_24(p.seed, ix, iz, dx, dz)) + ","
            + f(cell.x()) + "," + f(cell.y()) + ","
            + f(gradient.x()) + "," + f(gradient.y()) + ","
            + f(gradient24.x()) + "," + f(gradient24.y());
    }

    private static void utilities(Map<String, String> output) {
        int[] ints = {Integer.MIN_VALUE, -30_000_000, -17, -16, -1, 0, 1, 15, 16, 30_000_000, Integer.MAX_VALUE};
        for (int x : ints) for (int z : ints) {
            long packed = PosUtil.pack(x, z);
            require(PosUtil.unpackLeft(packed) == x && PosUtil.unpackRight(packed) == z,
                    "Integer coordinate packing lost sign or bits");
            output.put("pack/" + x + "/" + z, Long.toString(packed));
        }
        for (int i = 0; i < COORDS.length; i++) for (int j = 0; j < COORDS.length; j++) {
            float x = COORDS[i], z = COORDS[j];
            long packed = PosUtil.packf(x, z);
            require(Float.floatToRawIntBits(PosUtil.unpackLeftf(packed)) == Float.floatToRawIntBits(x)
                    && Float.floatToRawIntBits(PosUtil.unpackRightf(packed)) == Float.floatToRawIntBits(z),
                    "Float packing lost raw bits, including signed zero");
            long mixA = PosUtil.packMix(i - 17, z), mixB = PosUtil.packMix(x, j - 17);
            require(PosUtil.unpackLeft(mixA) == i - 17 && PosUtil.unpackRight(mixB) == j - 17,
                    "Mixed packing lost integer bits");
            require(Float.floatToRawIntBits(PosUtil.unpackRightf(mixA)) == Float.floatToRawIntBits(z)
                    && Float.floatToRawIntBits(PosUtil.unpackLeftf(mixB)) == Float.floatToRawIntBits(x),
                    "Mixed packing lost float bits");
            output.put("packf/" + i + "/" + j, packed + "," + mixA + "," + mixB);
        }
        require(PosUtil.contains(-16, -16, -16, -16, 0, 0), "Lower bounds must be included");
        require(!PosUtil.contains(0, -1, -16, -16, 0, 0), "Upper bounds must be excluded");
        require(PosUtil.contains(-0.5f, -0.5f, -1f, -1f, 0f, 0f), "Float containment failed");

        for (int seed : SEEDS) {
            Seed sequence = new Seed(seed);
            StringBuilder values = new StringBuilder();
            for (int i = 0; i < 16; i++) values.append(sequence.next()).append(',');
            require(sequence.root() == seed && sequence.split().get() == seed, "Seed split/root changed");
            require(sequence.offset(17).get() == seed + 17, "Seed offset changed");
            output.put("seed/" + seed, values + "/" + sequence.get());
            FastRandom random = new FastRandom(seed);
            FastRandom repeated = new FastRandom(seed);
            StringBuilder bits = new StringBuilder();
            for (int i = 0; i < 64; i++) {
                int value = random.nextInt();
                require(value == repeated.nextInt(), "Explicitly seeded RNG differs");
                bits.append(value).append(',');
            }
            for (int bound : new int[] {1, 2, 3, 31, 32, 1024, Integer.MAX_VALUE}) {
                for (int i = 0; i < 16; i++) {
                    int value = random.nextInt(bound);
                    require(value >= 0 && value < bound, "Bounded RNG escaped range");
                    bits.append(value).append(',');
                }
            }
            for (int i = 0; i < 32; i++) {
                float value = random.nextFloat();
                require(value >= 0 && value < 1, "RNG float escaped [0,1)");
                bits.append(f(value)).append(',').append(random.nextBoolean()).append(',');
            }
            random.seed(seed, 19).gamma(-7046029254386353131L);
            bits.append(random.nextInt());
            output.put("random/" + seed, bits.toString());
        }

        float[] values = {-16f, -1f, -0.25f, -0f, 0f, 0.125f, 0.5f, 0.999f, 1f, 16f};
        for (int i = 0; i < values.length; i++) {
            float x = values[i], t = (i + 0.5f) / values.length;
            output.put("math/" + i, NoiseUtil.floor(x) + "," + NoiseUtil.round(x) + ","
                + NoiseUtil.toInt(x) + "," + f(NoiseUtil.sin(x)) + "," + f(NoiseUtil.cos(x)) + ","
                + f(NoiseUtil.interpHermite(t)) + "," + f(NoiseUtil.interpQuintic(t)) + ","
                + f(NoiseUtil.lerp(-1f, 1f, t)) + "," + f(NoiseUtil.cubicLerp(-1f, 0f, 1f, 2f, t)) + ","
                + f(NoiseUtil.map(x, -2f, 2f, 4f)) + "," + f(NoiseUtil.curve(t, 3f)) + ","
                + f(NoiseUtil.pow(t, 3)) + "," + f(NoiseUtil.sqrt(t)));
        }
        // Observed legacy behavior, not a mathematical-floor correctness assertion.
        output.put("legacy/floor-negative-integer", Integer.toString(NoiseUtil.floor(-1f)));
    }

    public static void main(String[] args) {
        String mode = args.length == 0 ? "serial" : args[0];
        List<Point> points = new ArrayList<>();
        for (int seed : SEEDS) for (int i = 0; i < COORDS.length; i++)
            for (int j = 0; j < COORDS.length; j++) points.add(new Point(seed, i, j));
        if (mode.equals("shuffled") || mode.equals("parallel")) Collections.shuffle(points, new Random(8675309));
        else require(mode.equals("serial"), "Unknown probe mode");
        Map<String, String> output = new ConcurrentHashMap<>();
        var stream = mode.equals("parallel") ? points.parallelStream() : points.stream();
        stream.forEach(p -> output.put("noise/" + p.seed + "/" + p.xIndex + "/" + p.zIndex, noise(p)));
        utilities(output);
        new TreeMap<>(output).forEach((key, value) -> System.out.print(key + "\t" + value + "\n"));
    }
}
