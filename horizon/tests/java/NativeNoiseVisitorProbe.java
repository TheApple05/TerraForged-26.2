package raccoonman.reterraforged.world.worldgen.noise.module;
import java.util.*;
import java.nio.file.*;
import com.mojang.datafixers.util.Pair;
import net.minecraft.core.Holder;
import raccoonman.reterraforged.world.worldgen.noise.function.*;
import raccoonman.reterraforged.world.worldgen.noise.domain.Domains;
/** Observe actual source mapAll order and outputs, including inherited exceptions. */
public final class NativeNoiseVisitorProbe {
    private record Named(String name,Noise noise) {}
    private static Noise c(float f) {return new Constant(f);}
    private static Noise s() {return new Simplex(0.0125F,3,2.0F,0.5F,Interpolation.CURVE3);}
    private static Noise p() {return new Perlin2(12345,0.02F,2,2.0F,0.5F,Interpolation.CURVE4);}
    private static Noise a() {return new Add(s(),c(0.125F));}
    private static Noise b() {return new Multiply(p(),c(0.5F));}
    private static String bits(float f) {return String.format("%08x",Float.floatToRawIntBits(f));}
    private static List<Named> graphs() {List<Named> graphs=new ArrayList<>();
        graphs.add(new Named("constant",c(0.5F)));
        graphs.add(new Named("Perlin",new Perlin(12345,0.0125F,3,2.0F,0.5F,Interpolation.CURVE3)));
        graphs.add(new Named("Perlin2",new Perlin2(12345,0.0125F,3,2.0F,0.5F,Interpolation.CURVE3)));
        graphs.add(new Named("Simplex",new Simplex(0.0125F,3,2.0F,0.5F,Interpolation.CURVE3)));
        graphs.add(new Named("Simplex2",new Simplex2(0.0125F,3,2.0F,0.5F,Interpolation.CURVE3)));
        graphs.add(new Named("White",new White(0.0125F)));
        graphs.add(new Named("Cubic",new Cubic(0.0125F,3,2.0F,0.5F)));
        graphs.add(new Named("PerlinRidge",new PerlinRidge(0.0125F,3,2.0F,0.5F,Interpolation.CURVE3)));
        graphs.add(new Named("SimplexRidge",new SimplexRidge(0.0125F,3,2.0F,0.5F,Interpolation.CURVE3)));
        graphs.add(new Named("Billow",new Billow(0.0125F,3,2.0F,0.5F,Interpolation.CURVE3)));
        graphs.add(new Named("Worley",new Worley(0.0125F,1.0F,CellFunction.NOISE_LOOKUP,DistanceFunction.NATURAL,a())));
        graphs.add(new Named("WorleyEdge",new WorleyEdge(0.0125F,1.0F,EdgeFunction.DISTANCE_2_ADD,DistanceFunction.EUCLIDEAN)));
        graphs.add(new Named("Sin",new Sin(0.0125F,a())));
        graphs.add(new Named("ShiftSeed",new ShiftSeed(a(),12345)));
        graphs.add(new Named("Frequency",new Frequency(a(),b(),s())));
        graphs.add(new Named("Add",new Add(a(),b())));
        graphs.add(new Named("Multiply",new Multiply(a(),b())));
        graphs.add(new Named("Power",new Power(a(),1.5F)));
        graphs.add(new Named("PowerCurve",new PowerCurve(a(),1.5F)));
        graphs.add(new Named("Curve",new Curve(a(),CurveFunctions.scurve(2.0F,-0.5F))));
        graphs.add(new Named("Gradient",new Gradient(a(),c(0.25F),c(0.75F),b())));
        graphs.add(new Named("Terrace",new Terrace(a(),b(),s(),c(0.75F),0.25F,6)));
        graphs.add(new Named("AdvancedTerrace",new AdvancedTerrace(a(),b(),s(),c(0.75F),0.1F,0.8F,6,2)));
        graphs.add(new Named("Invert",new Invert(a())));
        graphs.add(new Named("Blend",new Blend(a(),b(),s(),0.5F,0.75F,Interpolation.CURVE3)));
        graphs.add(new Named("Alpha",new Alpha(a(),b())));
        graphs.add(new Named("Boost",new Boost(a(),2)));
        graphs.add(new Named("Steps",new Steps(a(),c(4.0F),c(0.1F),c(0.9F),Interpolation.CURVE3)));
        graphs.add(new Named("Abs",new Abs(new Add(a(),c(-1.0F)))));
        graphs.add(new Named("Map",new Map(a(),b(),c(2.0F))));
        graphs.add(new Named("Clamp",new Clamp(a(),b(),c(0.9F))));
        graphs.add(new Named("Threshold",new Threshold(a(),b(),s(),c(0.5F))));
        graphs.add(new Named("Min",new Min(a(),b())));
        graphs.add(new Named("Max",new Max(a(),b())));
        graphs.add(new Named("Warp",new Warp(a(),Domains.domain(b(),s(),c(16.0F)))));
        graphs.add(new Named("Erosion",new Erosion(a(),42,1,0.6F,80.0F,0.5F,2.0F,0.5F,Erosion.BlendMode.CONSTANT)));
        graphs.add(new Named("Line",new Line(-4.125F,2.25F,6.375F,-7.75F,a(),b(),s(),0.5F)));
        graphs.add(new Named("LinearSpline",new LinearSpline(a(),List.of(Pair.of(0.0F,b()),Pair.of(0.5F,new Abs(s())),Pair.of(1.0F,a())))));
        graphs.add(new Named("Cache2d",new Cache2d(a())));
        graphs.add(new Named("LegacyTemperature",new LegacyTemperature(0.0125F,3)));
        graphs.add(new Named("LegacyMoisture",new LegacyMoisture(a(),3)));
        graphs.add(new Named("HolderHolder",new Noises.HolderHolder(Holder.direct(a()))));
        graphs.add(new Named("Abs-nested",new Abs(new Abs(a()))));
        graphs.add(new Named("Min-PowerCurve",new Min(new PowerCurve(a(),2.0F),new Abs(b()))));
        graphs.add(new Named("Warp-direct",new Warp(a(),Domains.direct())));
        graphs.add(new Named("Warp-direction",new Warp(a(),Domains.direction(b(),s()))));
        graphs.add(new Named("Warp-add",new Warp(a(),Domains.add(Domains.direction(b(),s()),Domains.domain(a(),b(),c(10.0F))))));
        graphs.add(new Named("Warp-compound",new Warp(a(),Domains.compound(Domains.domain(a(),b(),c(10.0F)),Domains.direction(b(),s())))));
        return graphs;
    }
    public static void main(String[] args) throws Exception {
        StringBuilder out=new StringBuilder();
        int[] seeds={0,1,-1,42,Integer.MIN_VALUE,Integer.MAX_VALUE};
        float[] coords={-30_000_000f,-17f,-16.25f,-16f,-1.5f,-1f,-.25f,-0f,0f,.25f,1f,15f,15.75f,16f,16.25f,17f,30_000_000f};
        for(int mode=0;mode<3;mode++) {
            final int m=mode;
            List<Named> graphs=graphs();
            for(int n=0;n<graphs.size();n++) {
                Named g=graphs.get(n);
                List<String> trace=new ArrayList<>();
                Noise result=g.noise().mapAll(input->{
                    trace.add(input.getClass().getSimpleName());
                    int count=trace.size();
                    if(m==1) return new Add(input,c(0.03125F));
                    if(m==2) {
                        if(input instanceof Constant value) return c(value.value()+((count%7)-3)*0.015625F);
                        if(input instanceof Simplex || input instanceof Perlin2) return new ShiftSeed(input,count);
                    }
                    return input;
                });
                out.append("graph/").append(mode).append('/').append(n).append('\t').append(g.name()).append(',').append(result.getClass().getSimpleName()).append(',').append(bits(result.minValue())).append(',').append(bits(result.maxValue())).append('\n');
                out.append("trace/").append(mode).append('/').append(n).append('\t').append(String.join("|",trace)).append('\n');
                for(int seed:seeds) for(int i=0;i<coords.length;i++) for(int j=0;j<coords.length;j++) {
                    float value=result.compute(coords[i],coords[j],seed);
                    if(!Float.isFinite(value)) throw new IllegalStateException("Nonfinite visitor graph "+mode+"/"+n+"/"+seed+"/"+i+"/"+j);
                    out.append("sample/").append(mode).append('/').append(n).append('/').append(seed).append('/').append(i).append('/').append(j).append('\t').append(bits(value)).append('\n');
                }
            }
        }
        Files.writeString(Path.of(args[0]),out.toString(),java.nio.charset.StandardCharsets.UTF_8);
    }
}
