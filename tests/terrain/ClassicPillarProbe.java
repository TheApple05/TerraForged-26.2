import java.nio.ByteBuffer;
import java.security.MessageDigest;
import java.util.HexFormat;
import raccoonman.reterraforged.world.worldgen.feature.ClassicPillarMath;

public final class ClassicPillarProbe {
    public static void main(String[] args) throws Exception {
        MessageDigest digest = MessageDigest.getInstance("SHA-256");
        int assertions = 0;
        for (float maximum : new float[] {1.0F, 0.999F}) {
            for (int i = -8; i <= 8200; i++) {
                float value = i / 8192.0F;
                float result = ClassicPillarMath.terrace(value, maximum);
                if (!Float.isFinite(result) || result < 0 || result > 1.0F) throw new AssertionError("Invalid terrace " + result);
                digest.update(ByteBuffer.allocate(4).putInt(Float.floatToRawIntBits(result)).array());
                assertions++;
            }
        }
        for (boolean stone : new boolean[] {false, true}) {
            int maximum = stone ? 50 : 30;
            if (ClassicPillarMath.addedHeight(1, 1, stone) != maximum) throw new AssertionError("Height scale");
            for (float river : new float[] {0, 0.00025F, 0.0005F, 1}) {
                for (float edge : new float[] {0, 0.25F, 1}) {
                    for (float gradient : new float[] {0, 0.5F, 1}) {
                        float mask = ClassicPillarMath.mask(edge, river, gradient);
                        int height = ClassicPillarMath.addedHeight(1, mask, stone);
                        if (height < 0 || height > maximum) throw new AssertionError("Unbounded height");
                        if ((edge == 0 || river == 0 || gradient == 1) && height != 0) throw new AssertionError("Mask leakage");
                        assertions++;
                    }
                }
            }
        }
        if (ClassicPillarMath.mask(1, 0.00025F, 0) != 0.5F || ClassicPillarMath.mask(1, 1, 0.5F) != 0.5F) throw new AssertionError("Mask scale");
        System.out.println("{\"status\":\"passed\",\"assertions\":" + (assertions + 4) + ",\"terraceRawBitsSha256\":\"" + HexFormat.of().formatHex(digest.digest()) + "\"}");
    }
}
