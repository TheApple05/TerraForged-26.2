package raccoonman.reterraforged.world.worldgen.noise.module;
import java.util.ArrayList;
import java.util.List;
import raccoonman.reterraforged.world.worldgen.noise.domain.Domain;
import raccoonman.reterraforged.world.worldgen.noise.domain.Domains;
import java.nio.file.Files;
import java.nio.file.Path;
import raccoonman.reterraforged.world.worldgen.noise.function.Interpolation;
import raccoonman.reterraforged.world.worldgen.noise.function.CurveFunctions;
/** Genuine Java constructor observations; never native expected values. */
public final class NativeNoiseFinalProbe {
    private record Named(String name,Noise noise) {}
    private static Noise c(float f) {return new Constant(f);}
    private static Noise s() {return new Simplex(0.0125F,3,2.0F,0.5F,Interpolation.CURVE3);}
    private static String bits(float f) {return String.format("%08x",Float.floatToRawIntBits(f));}
    private static Noise p() {return new Perlin2(12345,0.02F,2,2.0F,0.5F,Interpolation.CURVE4);}
    private static Domain d(int index) {return switch(index) {
        case 0 -> Domains.direct();
        case 1 -> Domains.domain(s(),p(),c(10.0F));
        case 2 -> Domains.domain(new Add(s(),c(-0.5F)),new Add(p(),c(-0.5F)),c(10.0F));
        case 3 -> Domains.direction(s(),c(16.0F));
        case 4 -> Domains.domain(s(),p(),new Add(c(1.0F),new Multiply(s(),c(8.0F))));
        case 5 -> Domains.add(d(1),d(3));
        case 6 -> Domains.compound(d(1),d(3));
        case 7 -> Domains.compound(d(3),d(1));
        case 8 -> Domains.add(Domains.compound(d(1),d(3)),d(4));
        default -> throw new IllegalArgumentException();
    };}
    public static void main(String[] args) throws Exception {
        List<Named> graphs=new ArrayList<>();
        graphs.add(new Named("line-0.0F-1.0F",new Line(-4.125F,2.25F,6.375F,-7.75F,c(1.0F),c(0.25F),c(0.5F),0.0F)));
        graphs.add(new Named("line-0.0F-20.0F",new Line(-4.125F,2.25F,6.375F,-7.75F,c(20.0F),c(0.25F),c(0.5F),0.0F)));
        graphs.add(new Named("line-0.0F-100.0F",new Line(-4.125F,2.25F,6.375F,-7.75F,c(100.0F),c(0.25F),c(0.5F),0.0F)));
        graphs.add(new Named("line-0.5F-1.0F",new Line(-4.125F,2.25F,6.375F,-7.75F,c(1.0F),c(0.25F),c(0.5F),0.5F)));
        graphs.add(new Named("line-0.5F-20.0F",new Line(-4.125F,2.25F,6.375F,-7.75F,c(20.0F),c(0.25F),c(0.5F),0.5F)));
        graphs.add(new Named("line-0.5F-100.0F",new Line(-4.125F,2.25F,6.375F,-7.75F,c(100.0F),c(0.25F),c(0.5F),0.5F)));
        graphs.add(new Named("line-1.0F-1.0F",new Line(-4.125F,2.25F,6.375F,-7.75F,c(1.0F),c(0.25F),c(0.5F),1.0F)));
        graphs.add(new Named("line-1.0F-20.0F",new Line(-4.125F,2.25F,6.375F,-7.75F,c(20.0F),c(0.25F),c(0.5F),1.0F)));
        graphs.add(new Named("line-1.0F-100.0F",new Line(-4.125F,2.25F,6.375F,-7.75F,c(100.0F),c(0.25F),c(0.5F),1.0F)));
        graphs.add(new Named("line-variable",new Line(-4.125F,2.25F,6.375F,-7.75F,new Add(c(1.0F),new Multiply(s(),c(128.0F))),s(),s(),0.5F)));
        graphs.add(new Named("terrace-2-0.0F",new Terrace(s(),c(0.5F),c(0.75F),c(0.6F),0.0F,2)));
        graphs.add(new Named("terrace-2-0.25F",new Terrace(s(),c(0.5F),c(0.75F),c(0.6F),0.25F,2)));
        graphs.add(new Named("terrace-2-1.0F",new Terrace(s(),c(0.5F),c(0.75F),c(0.6F),1.0F,2)));
        graphs.add(new Named("terrace-2-1.5F",new Terrace(s(),c(0.5F),c(0.75F),c(0.6F),1.5F,2)));
        graphs.add(new Named("terrace-4-0.0F",new Terrace(s(),c(0.5F),c(0.75F),c(0.6F),0.0F,4)));
        graphs.add(new Named("terrace-4-0.25F",new Terrace(s(),c(0.5F),c(0.75F),c(0.6F),0.25F,4)));
        graphs.add(new Named("terrace-4-1.0F",new Terrace(s(),c(0.5F),c(0.75F),c(0.6F),1.0F,4)));
        graphs.add(new Named("terrace-4-1.5F",new Terrace(s(),c(0.5F),c(0.75F),c(0.6F),1.5F,4)));
        graphs.add(new Named("terrace-8-0.0F",new Terrace(s(),c(0.5F),c(0.75F),c(0.6F),0.0F,8)));
        graphs.add(new Named("terrace-8-0.25F",new Terrace(s(),c(0.5F),c(0.75F),c(0.6F),0.25F,8)));
        graphs.add(new Named("terrace-8-1.0F",new Terrace(s(),c(0.5F),c(0.75F),c(0.6F),1.0F,8)));
        graphs.add(new Named("terrace-8-1.5F",new Terrace(s(),c(0.5F),c(0.75F),c(0.6F),1.5F,8)));
        graphs.add(new Named("terrace-16-0.0F",new Terrace(s(),c(0.5F),c(0.75F),c(0.6F),0.0F,16)));
        graphs.add(new Named("terrace-16-0.25F",new Terrace(s(),c(0.5F),c(0.75F),c(0.6F),0.25F,16)));
        graphs.add(new Named("terrace-16-1.0F",new Terrace(s(),c(0.5F),c(0.75F),c(0.6F),1.0F,16)));
        graphs.add(new Named("terrace-16-1.5F",new Terrace(s(),c(0.5F),c(0.75F),c(0.6F),1.5F,16)));
        graphs.add(new Named("terrace-variable",new Terrace(s(),s(),s(),s(),0.25F,6)));
        graphs.add(new Named("terrace-nonzero-bounds",new Terrace(new Add(s(),c(3.0F)),s(),s(),s(),0.25F,6)));
        graphs.add(new Named("advanced-3-0-0.0F",new AdvancedTerrace(s(),c(0.2F),c(0.0F),c(0.75F),0.1F,0.8F,3,0)));
        graphs.add(new Named("advanced-3-0-0.25F",new AdvancedTerrace(s(),c(0.2F),c(0.25F),c(0.75F),0.1F,0.8F,3,0)));
        graphs.add(new Named("advanced-3-0-1.0F",new AdvancedTerrace(s(),c(0.2F),c(1.0F),c(0.75F),0.1F,0.8F,3,0)));
        graphs.add(new Named("advanced-3-1-0.0F",new AdvancedTerrace(s(),c(0.2F),c(0.0F),c(0.75F),0.1F,0.8F,3,1)));
        graphs.add(new Named("advanced-3-1-0.25F",new AdvancedTerrace(s(),c(0.2F),c(0.25F),c(0.75F),0.1F,0.8F,3,1)));
        graphs.add(new Named("advanced-3-1-1.0F",new AdvancedTerrace(s(),c(0.2F),c(1.0F),c(0.75F),0.1F,0.8F,3,1)));
        graphs.add(new Named("advanced-3-3-0.0F",new AdvancedTerrace(s(),c(0.2F),c(0.0F),c(0.75F),0.1F,0.8F,3,3)));
        graphs.add(new Named("advanced-3-3-0.25F",new AdvancedTerrace(s(),c(0.2F),c(0.25F),c(0.75F),0.1F,0.8F,3,3)));
        graphs.add(new Named("advanced-3-3-1.0F",new AdvancedTerrace(s(),c(0.2F),c(1.0F),c(0.75F),0.1F,0.8F,3,3)));
        graphs.add(new Named("advanced-8-0-0.0F",new AdvancedTerrace(s(),c(0.2F),c(0.0F),c(0.75F),0.1F,0.8F,8,0)));
        graphs.add(new Named("advanced-8-0-0.25F",new AdvancedTerrace(s(),c(0.2F),c(0.25F),c(0.75F),0.1F,0.8F,8,0)));
        graphs.add(new Named("advanced-8-0-1.0F",new AdvancedTerrace(s(),c(0.2F),c(1.0F),c(0.75F),0.1F,0.8F,8,0)));
        graphs.add(new Named("advanced-8-1-0.0F",new AdvancedTerrace(s(),c(0.2F),c(0.0F),c(0.75F),0.1F,0.8F,8,1)));
        graphs.add(new Named("advanced-8-1-0.25F",new AdvancedTerrace(s(),c(0.2F),c(0.25F),c(0.75F),0.1F,0.8F,8,1)));
        graphs.add(new Named("advanced-8-1-1.0F",new AdvancedTerrace(s(),c(0.2F),c(1.0F),c(0.75F),0.1F,0.8F,8,1)));
        graphs.add(new Named("advanced-8-3-0.0F",new AdvancedTerrace(s(),c(0.2F),c(0.0F),c(0.75F),0.1F,0.8F,8,3)));
        graphs.add(new Named("advanced-8-3-0.25F",new AdvancedTerrace(s(),c(0.2F),c(0.25F),c(0.75F),0.1F,0.8F,8,3)));
        graphs.add(new Named("advanced-8-3-1.0F",new AdvancedTerrace(s(),c(0.2F),c(1.0F),c(0.75F),0.1F,0.8F,8,3)));
        graphs.add(new Named("advanced-below",new AdvancedTerrace(c(0.05F),c(0.2F),c(1.0F),c(0.75F),0.1F,0.8F,4,2)));
        graphs.add(new Named("advanced-equal",new AdvancedTerrace(c(0.1F),c(0.2F),c(1.0F),c(0.75F),0.1F,0.8F,4,2)));
        graphs.add(new Named("advanced-mid",new AdvancedTerrace(c(0.5F),c(0.2F),c(1.0F),c(0.75F),0.1F,0.8F,4,2)));
        graphs.add(new Named("advanced-above",new AdvancedTerrace(c(0.9F),c(0.2F),c(1.0F),c(0.75F),0.1F,0.8F,4,2)));
        graphs.add(new Named("advanced-variable",new AdvancedTerrace(s(),new Multiply(s(),c(0.25F)),s(),s(),0.1F,0.8F,4,2)));
        graphs.add(new Named("warp-0",new Warp(s(),d(0))));
        graphs.add(new Named("warp-1",new Warp(s(),d(1))));
        graphs.add(new Named("warp-2",new Warp(s(),d(2))));
        graphs.add(new Named("warp-3",new Warp(s(),d(3))));
        graphs.add(new Named("warp-4",new Warp(s(),d(4))));
        graphs.add(new Named("warp-5",new Warp(s(),d(5))));
        graphs.add(new Named("warp-6",new Warp(s(),d(6))));
        graphs.add(new Named("warp-7",new Warp(s(),d(7))));
        graphs.add(new Named("warp-8",new Warp(s(),d(8))));
        graphs.add(new Named("erosion-CONSTANT-0-1",new Erosion(s(),0,1,0.6F,80.0F,0.5F,2.0F,0.5F,Erosion.BlendMode.CONSTANT)));
        graphs.add(new Named("erosion-CONSTANT-0-3",new Erosion(s(),0,3,0.6F,80.0F,0.5F,2.0F,0.5F,Erosion.BlendMode.CONSTANT)));
        graphs.add(new Named("erosion-CONSTANT-42-1",new Erosion(s(),42,1,0.6F,80.0F,0.5F,2.0F,0.5F,Erosion.BlendMode.CONSTANT)));
        graphs.add(new Named("erosion-CONSTANT-42-3",new Erosion(s(),42,3,0.6F,80.0F,0.5F,2.0F,0.5F,Erosion.BlendMode.CONSTANT)));
        graphs.add(new Named("erosion-CONSTANT--2147483648-1",new Erosion(s(),-2147483648,1,0.6F,80.0F,0.5F,2.0F,0.5F,Erosion.BlendMode.CONSTANT)));
        graphs.add(new Named("erosion-CONSTANT--2147483648-3",new Erosion(s(),-2147483648,3,0.6F,80.0F,0.5F,2.0F,0.5F,Erosion.BlendMode.CONSTANT)));
        graphs.add(new Named("erosion-INPUT_LINEAR-0-1",new Erosion(s(),0,1,0.6F,80.0F,0.5F,2.0F,0.5F,Erosion.BlendMode.INPUT_LINEAR)));
        graphs.add(new Named("erosion-INPUT_LINEAR-0-3",new Erosion(s(),0,3,0.6F,80.0F,0.5F,2.0F,0.5F,Erosion.BlendMode.INPUT_LINEAR)));
        graphs.add(new Named("erosion-INPUT_LINEAR-42-1",new Erosion(s(),42,1,0.6F,80.0F,0.5F,2.0F,0.5F,Erosion.BlendMode.INPUT_LINEAR)));
        graphs.add(new Named("erosion-INPUT_LINEAR-42-3",new Erosion(s(),42,3,0.6F,80.0F,0.5F,2.0F,0.5F,Erosion.BlendMode.INPUT_LINEAR)));
        graphs.add(new Named("erosion-INPUT_LINEAR--2147483648-1",new Erosion(s(),-2147483648,1,0.6F,80.0F,0.5F,2.0F,0.5F,Erosion.BlendMode.INPUT_LINEAR)));
        graphs.add(new Named("erosion-INPUT_LINEAR--2147483648-3",new Erosion(s(),-2147483648,3,0.6F,80.0F,0.5F,2.0F,0.5F,Erosion.BlendMode.INPUT_LINEAR)));
        graphs.add(new Named("erosion-OUTPUT_LINEAR-0-1",new Erosion(s(),0,1,0.6F,80.0F,0.5F,2.0F,0.5F,Erosion.BlendMode.OUTPUT_LINEAR)));
        graphs.add(new Named("erosion-OUTPUT_LINEAR-0-3",new Erosion(s(),0,3,0.6F,80.0F,0.5F,2.0F,0.5F,Erosion.BlendMode.OUTPUT_LINEAR)));
        graphs.add(new Named("erosion-OUTPUT_LINEAR-42-1",new Erosion(s(),42,1,0.6F,80.0F,0.5F,2.0F,0.5F,Erosion.BlendMode.OUTPUT_LINEAR)));
        graphs.add(new Named("erosion-OUTPUT_LINEAR-42-3",new Erosion(s(),42,3,0.6F,80.0F,0.5F,2.0F,0.5F,Erosion.BlendMode.OUTPUT_LINEAR)));
        graphs.add(new Named("erosion-OUTPUT_LINEAR--2147483648-1",new Erosion(s(),-2147483648,1,0.6F,80.0F,0.5F,2.0F,0.5F,Erosion.BlendMode.OUTPUT_LINEAR)));
        graphs.add(new Named("erosion-OUTPUT_LINEAR--2147483648-3",new Erosion(s(),-2147483648,3,0.6F,80.0F,0.5F,2.0F,0.5F,Erosion.BlendMode.OUTPUT_LINEAR)));
        int[] seeds={0,1,-1,42,Integer.MIN_VALUE,Integer.MAX_VALUE};
        float[] coords={-30_000_000f,-17f,-16.25f,-16f,-1.5f,-1f,-.25f,-0f,0f,.25f,1f,15f,15.75f,16f,16.25f,17f,30_000_000f};
        StringBuilder out=new StringBuilder();
        for(int n=0;n<graphs.size();n++) {
            Named g=graphs.get(n);
            out.append("graph/").append(n).append('\t').append(g.name()).append(',').append(bits(g.noise().minValue())).append(',').append(bits(g.noise().maxValue())).append('\n');
            for(int seed:seeds) for(int i=0;i<coords.length;i++) for(int j=0;j<coords.length;j++) {
                float v=g.noise().compute(coords[i],coords[j],seed);
                if(!Float.isFinite(v)) throw new IllegalStateException("Nonfinite graph "+n+" seed "+seed+" coords "+i+","+j);
                out.append("sample/").append(n).append('/').append(seed).append('/').append(i).append('/').append(j).append('\t').append(bits(v)).append('\n');
            }
        }
        for(int n=0;n<9;n++) {Domain domain=d(n);
            for(int seed:seeds) for(int i=0;i<coords.length;i++) for(int j=0;j<coords.length;j++) {
                float x=coords[i],z=coords[j];
                out.append("domain/").append(n).append('/').append(seed).append('/').append(i).append('/').append(j).append('\t')
                    .append(bits(domain.getOffsetX(x,z,seed))).append(',').append(bits(domain.getOffsetZ(x,z,seed))).append(',')
                    .append(bits(domain.getX(x,z,seed))).append(',').append(bits(domain.getZ(x,z,seed))).append('\n');
            }
        }
        for(int i=0;i<coords.length;i++) for(int j=0;j<coords.length;j++) {
            out.append("line-helper/").append(i).append('/').append(j).append('\t')
                .append(bits(Line.distSq(coords[i],coords[j],-4.125F,2.25F))).append(',')
                .append(bits(Line.distanceOnLine(coords[i],coords[j],-4.125F,2.25F,6.375F,-7.75F))).append(',')
                .append(Line.intersect(coords[i],coords[j],6.375F,-7.75F,-1.0F,0.0F,1.0F,0.0F)).append('\n');
        }
        Noise line=new Line(-4.125F,2.25F,6.375F,-7.75F,c(100.0F),c(0.25F),c(0.5F),0.5F);
        out.append("nonfinite/line-start\t").append(Float.isNaN(line.compute(-4.125F,2.25F,42))).append('\n');
        out.append("nonfinite/line-end\t").append(Float.isNaN(line.compute(6.375F,-7.75F,42))).append('\n');
        Noise erosion0=new Erosion(s(),42,0,0.6F,80.0F,0.5F,2.0F,0.5F,Erosion.BlendMode.CONSTANT);
        out.append("nonfinite/erosion-zero-octaves\t").append(Float.isNaN(erosion0.compute(0.0F,0.0F,42))).append('\n');
        Files.writeString(Path.of(args[0]),out.toString(),java.nio.charset.StandardCharsets.UTF_8);
    }
}
