import java.nio.file.*;
import raccoonman.reterraforged.world.worldgen.cell.biome.type.*;
import raccoonman.reterraforged.world.worldgen.noise.NoiseUtil;

/** Export source-owned palette data and independent climate lookup observations. */
public final class NativeBiomeProbe {
    private static String bits(float f) {return Float.isNaN(f)?"NaN":String.format("%08x",Float.floatToRawIntBits(f));}
    private static String lookup(float t,float m,boolean curve) {
        try {return Integer.toString((curve?BiomeType.getCurve(t,m):BiomeType.getLinear(t,m)).ordinal());}
        catch(IndexOutOfBoundsException error) {return "error";}
    }
    private static String pair(BiomeType type,float identity) {
        try {return bits(type.getTemperature(identity))+","+bits(type.getMoisture(identity));}
        catch(IndexOutOfBoundsException error) {return "error";}
    }
    public static void main(String[] args) throws Exception {
        BiomeType[][] table=BiomeTypeLoader.getInstance().getTypeMap();
        byte[] bytes=new byte[256*256];
        for(int y=0;y<256;y++) for(int x=0;x<256;x++) bytes[y*256+x]=(byte)table[y][x].ordinal();
        Files.write(Path.of(args[0]),bytes);
        StringBuilder out=new StringBuilder();
        for(int t=0;t<256;t++) for(int m=0;m<256;m++) {
            float temperature=t/255.0F,moisture=m/255.0F;
            out.append("grid/").append(t).append('/').append(m).append('\t').append(lookup(temperature,moisture,true)).append(',').append(lookup(temperature,moisture,false)).append('\n');
        }
        float[] edges={-1.0F,-0.001F,-0.0F,0.0F,Float.MIN_VALUE,0.25F,Math.nextDown(0.5F),0.5F,Math.nextUp(0.5F),0.75F,1.0F,Math.nextUp(1.0F),1.5F,Float.NaN,Float.POSITIVE_INFINITY};
        for(int t=0;t<edges.length;t++) for(int m=0;m<edges.length;m++)
            out.append("edge/").append(t).append('/').append(m).append('\t').append(lookup(edges[t],edges[m],true)).append(',').append(lookup(edges[t],edges[m],false)).append('\n');
        for(BiomeType type:BiomeType.values()) {
            var ranges=BiomeTypeLoader.getInstance().getRanges(type);
            out.append("type/").append(type.ordinal()).append('\t').append(type.name()).append(',').append(bits(ranges[0].x())).append(',').append(bits(ranges[0].y())).append(',').append(bits(ranges[1].x())).append(',').append(bits(ranges[1].y())).append('\n');
            for(int i=0;i<=256;i++) out.append("pair/").append(type.ordinal()).append('/').append(i).append('\t').append(pair(type,i/256.0F)).append('\n');
            for(int i=0;i<edges.length;i++) {
                out.append("pair-edge/").append(type.ordinal()).append('/').append(i).append('\t').append(pair(type,edges[i])).append('\n');
                out.append("map/").append(type.ordinal()).append('/').append(i).append('\t').append(bits(type.mapTemperature(edges[i]))).append(',').append(bits(type.mapMoisture(edges[i]))).append('\n');
            }
        }
        Files.writeString(Path.of(args[1]),out.toString(),java.nio.charset.StandardCharsets.UTF_8);
    }
}
