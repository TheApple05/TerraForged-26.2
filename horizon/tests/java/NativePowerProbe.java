import raccoonman.reterraforged.world.worldgen.noise.NoiseUtil;

public class NativePowerProbe {

    private static String hex(float v) {
        return String.format("%08x", Float.floatToRawIntBits(v));
    }

    private static void emitPower(int bi, int ei, float base, float exp) {
        float r = NoiseUtil.pow(base, exp);
        System.out.println("power/" + bi + "/" + ei + "\t" + hex(r));
    }

    private static void emitNegative(int bi, int ei, float base, float exp) {
        float r = NoiseUtil.pow(base, exp);
        System.out.println("negative/" + bi + "/" + ei + "\t" + hex(r));
    }

    public static void main(String[] args) {
        int[] baseBits = {
            0x00000000, 0x80000000, 0x00000001, 0x00800000,
            0x3a83126f, 0x3e800000, 0x3f000000, 0x3f800000,
            0x3fc00000, 0x40000000, 0x41200000, 0x7f7fffff
        };
        int[] expBits = {
            0xc0400000, 0xc0000000, 0xbf800000, 0xbf000000,
            0x80000000, 0x00000000, 0x3e000000, 0x3f000000,
            0x3f400000, 0x3f800000, 0x3fa00000, 0x3fc00000,
            0x40000000, 0x40400000, 0x41200000
        };

        for (int bi = 0; bi < baseBits.length; bi++) {
            float base = Float.intBitsToFloat(baseBits[bi]);
            for (int ei = 0; ei < expBits.length; ei++) {
                float exp = Float.intBitsToFloat(expBits[ei]);
                emitPower(bi, ei, base, exp);
            }
        }

        int[] negBits = {0xbe800000, 0xbf800000, 0xc0000000, 0xc1200000};
        int[] negExpBits = {
            0xc0400000, 0xc0000000, 0xbf800000, 0x80000000,
            0x00000000, 0x3f800000, 0x40000000, 0x40400000, 0x41200000
        };

        for (int bi = 0; bi < negBits.length; bi++) {
            float base = Float.intBitsToFloat(negBits[bi]);
            for (int ei = 0; ei < negExpBits.length; ei++) {
                float exp = Float.intBitsToFloat(negExpBits[ei]);
                emitNegative(bi, ei, base, exp);
            }
        }

        int[] lacBits = {0x3f8ccccd, 0x3fc00000, 0x40000000, 0x40200000, 0x40400000};

        for (int li = 0; li < lacBits.length; li++) {
            float lacunarity = Float.intBitsToFloat(lacBits[li]);
            float frequency = 1.0f;
            for (int step = 0; step < 32; step++) {
                float r = NoiseUtil.pow(frequency, -1.0f);
                System.out.println("spectral/" + li + "/" + step + "\t" + hex(r));
                frequency *= lacunarity;
            }
        }
    }
}
