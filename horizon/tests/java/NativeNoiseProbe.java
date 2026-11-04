package raccoonman.reterraforged.world.worldgen.noise.module;

import raccoonman.reterraforged.world.worldgen.noise.NoiseUtil;
import raccoonman.reterraforged.world.worldgen.noise.NoiseUtil.Vec2f;
import raccoonman.reterraforged.world.worldgen.noise.function.Interpolation;

/** Developer-only independent oracle. Calls genuine packaged Java noise classes. */
public final class NativeNoiseProbe {
    private static final int[] SEEDS = {0, 1, -1, 42, Integer.MIN_VALUE, Integer.MAX_VALUE};
    private static final float[] COORDS = {-30_000_000f, -17f, -16.25f, -16f, -1.5f, -1f,
        -0.25f, -0f, 0f, 0.25f, 1f, 15f, 15.75f, 16f, 16.25f, 17f, 30_000_000f};
    private static String bits(float value) { return String.format("%08x", Float.floatToRawIntBits(value)); }
    private static void vectors(String name, Vec2f[] values) {
        for (int i = 0; i < values.length; i++)
            System.out.println("vectors/" + name + "/" + i + "\t" + bits(values[i].x()) + "," + bits(values[i].y()));
    }
    public static void main(String[] args) {
        vectors("grad", NoiseUtil.GRAD_2D);
        vectors("grad24", NoiseUtil.GRAD_2D_24);
        vectors("cell", NoiseUtil.CELL_2D);
        float[][] parameters = {{1f,0f,2f,.5f}, {1f,1f,2f,.5f}, {.0125f,3f,2f,.5f}, {.001f,7f,2.5f,.75f}};
        for (int seed : SEEDS) for (int i = 0; i < COORDS.length; i++) for (int j = 0; j < COORDS.length; j++) {
            float x = COORDS[i], z = COORDS[j];
            String key = seed + "/" + i + "/" + j;
            for (Interpolation interpolation : Interpolation.values()) {
                System.out.println("perlin/sample/" + interpolation.ordinal() + "/" + key + "\t" + bits(Perlin.sample(x,z,seed,interpolation)));
                for (int p = 0; p < parameters.length; p++) {
                    float[] f = parameters[p];
                    Perlin noise = new Perlin(seed,f[0],(int)f[1],f[2],f[3],interpolation);
                    System.out.println("perlin/compute/" + interpolation.ordinal() + "/" + p + "/" + key + "\t"
                        + bits(noise.compute(x,z,seed)) + "," + bits(noise.compute(x,z,seed+104729)));
                }
            }
            System.out.println("simplex/sample/" + key + "\t" + bits(Simplex.sample(x,z,seed)));
            for (int p = 0; p < parameters.length; p++) {
                float[] f = parameters[p];
                Simplex noise = new Simplex(f[0],(int)f[1],f[2],f[3],Interpolation.CURVE4);
                System.out.println("simplex/compute/" + p + "/" + key + "\t" + bits(noise.compute(x,z,seed)));
            }
        }
    }
}
