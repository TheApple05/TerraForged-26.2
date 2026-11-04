package raccoonman.reterraforged.world.worldgen.noise.module;
import java.util.ArrayList;
import java.util.List;
import java.nio.file.Files;
import java.nio.file.Path;
import raccoonman.reterraforged.world.worldgen.noise.function.Interpolation;
import raccoonman.reterraforged.world.worldgen.noise.function.CurveFunctions;
/** Genuine Java constructor observations; never native expected values. */
public final class NativeNoiseShapingProbe {
    private record Named(String name,Noise noise) {}
    private static Noise c(float f) {return new Constant(f);}
    private static Noise s() {return new Simplex(0.0125F,3,2.0F,0.5F,Interpolation.CURVE3);}
    private static String bits(float f) {return String.format("%08x",Float.floatToRawIntBits(f));}
    public static void main(String[] args) throws Exception {
        List<Named> graphs=new ArrayList<>();
        graphs.add(new Named("sin-0.0F-0.0F",new Sin(0.0F,c(0.0F))));
        graphs.add(new Named("sin-0.0F--0.0F",new Sin(0.0F,c(-0.0F))));
        graphs.add(new Named("sin-0.0F-1.0F",new Sin(0.0F,c(1.0F))));
        graphs.add(new Named("sin-0.0F-0.25F",new Sin(0.0F,c(0.25F))));
        graphs.add(new Named("sin-0.0F--0.25F",new Sin(0.0F,c(-0.25F))));
        graphs.add(new Named("sin-0.0F-1.25F",new Sin(0.0F,c(1.25F))));
        graphs.add(new Named("sin-0.125F-0.0F",new Sin(0.125F,c(0.0F))));
        graphs.add(new Named("sin-0.125F--0.0F",new Sin(0.125F,c(-0.0F))));
        graphs.add(new Named("sin-0.125F-1.0F",new Sin(0.125F,c(1.0F))));
        graphs.add(new Named("sin-0.125F-0.25F",new Sin(0.125F,c(0.25F))));
        graphs.add(new Named("sin-0.125F--0.25F",new Sin(0.125F,c(-0.25F))));
        graphs.add(new Named("sin-0.125F-1.25F",new Sin(0.125F,c(1.25F))));
        graphs.add(new Named("sin--0.25F-0.0F",new Sin(-0.25F,c(0.0F))));
        graphs.add(new Named("sin--0.25F--0.0F",new Sin(-0.25F,c(-0.0F))));
        graphs.add(new Named("sin--0.25F-1.0F",new Sin(-0.25F,c(1.0F))));
        graphs.add(new Named("sin--0.25F-0.25F",new Sin(-0.25F,c(0.25F))));
        graphs.add(new Named("sin--0.25F--0.25F",new Sin(-0.25F,c(-0.25F))));
        graphs.add(new Named("sin--0.25F-1.25F",new Sin(-0.25F,c(1.25F))));
        graphs.add(new Named("sin-dynamic",new Sin(0.0125F,s())));
        graphs.add(new Named("power--0.5F",new Power(s(),-0.5F)));
        graphs.add(new Named("power-0.0F",new Power(s(),0.0F)));
        graphs.add(new Named("power-0.125F",new Power(s(),0.125F)));
        graphs.add(new Named("power-0.5F",new Power(s(),0.5F)));
        graphs.add(new Named("power-1.25F",new Power(s(),1.25F)));
        graphs.add(new Named("power-2.0F",new Power(s(),2.0F)));
        graphs.add(new Named("power-3.0F",new Power(s(),3.0F)));
        graphs.add(new Named("power-negative-2.0F",new Power(new Add(s(),c(-2.0F)),2.0F)));
        graphs.add(new Named("power-negative-3.0F",new Power(new Add(s(),c(-2.0F)),3.0F)));
        graphs.add(new Named("powercurve-unit-0.5F",new PowerCurve(s(),0.5F)));
        graphs.add(new Named("powercurve-shift-0.5F",new PowerCurve(new Add(s(),c(3.0F)),0.5F)));
        graphs.add(new Named("powercurve-constant-0.5F",new PowerCurve(c(-0.0F),0.5F)));
        graphs.add(new Named("powercurve-unit-1.0F",new PowerCurve(s(),1.0F)));
        graphs.add(new Named("powercurve-shift-1.0F",new PowerCurve(new Add(s(),c(3.0F)),1.0F)));
        graphs.add(new Named("powercurve-constant-1.0F",new PowerCurve(c(-0.0F),1.0F)));
        graphs.add(new Named("powercurve-unit-2.0F",new PowerCurve(s(),2.0F)));
        graphs.add(new Named("powercurve-shift-2.0F",new PowerCurve(new Add(s(),c(3.0F)),2.0F)));
        graphs.add(new Named("powercurve-constant-2.0F",new PowerCurve(c(-0.0F),2.0F)));
        graphs.add(new Named("powercurve-unit-3.0F",new PowerCurve(s(),3.0F)));
        graphs.add(new Named("powercurve-shift-3.0F",new PowerCurve(new Add(s(),c(3.0F)),3.0F)));
        graphs.add(new Named("powercurve-constant-3.0F",new PowerCurve(c(-0.0F),3.0F)));
        graphs.add(new Named("curve-LINEAR",new Curve(s(),Interpolation.LINEAR)));
        graphs.add(new Named("curve-CURVE3",new Curve(s(),Interpolation.CURVE3)));
        graphs.add(new Named("curve-CURVE4",new Curve(s(),Interpolation.CURVE4)));
        graphs.add(new Named("scurve-0.5F-2.0F",new Curve(s(),CurveFunctions.scurve(0.5F,2.0F))));
        graphs.add(new Named("scurve-2.0F--3.0F",new Curve(s(),CurveFunctions.scurve(2.0F,-3.0F))));
        graphs.add(new Named("scurve-2.0F--0.5F",new Curve(s(),CurveFunctions.scurve(2.0F,-0.5F))));
        graphs.add(new Named("gradient-dynamic",new Gradient(s(),c(0.25F),c(0.75F),c(0.6F))));
        graphs.add(new Named("gradient-below",new Gradient(c(0.1F),c(0.25F),c(0.75F),c(0.6F))));
        graphs.add(new Named("gradient-lower",new Gradient(c(0.25F),c(0.25F),c(0.75F),c(0.6F))));
        graphs.add(new Named("gradient-upper",new Gradient(c(0.75F),c(0.25F),c(0.75F),c(0.6F))));
        graphs.add(new Named("gradient-above",new Gradient(c(0.9F),c(0.25F),c(0.75F),c(0.6F))));
        graphs.add(new Named("gradient-variable",new Gradient(s(),new Multiply(s(),c(0.5F)),new Add(c(0.5F),new Multiply(s(),c(0.5F))),s())));
        graphs.add(new Named("alpha-0.0F",new Alpha(s(),c(0.0F))));
        graphs.add(new Named("alpha-1.0F",new Alpha(s(),c(1.0F))));
        graphs.add(new Named("alpha-0.25F",new Alpha(s(),c(0.25F))));
        graphs.add(new Named("alpha--0.25F",new Alpha(s(),c(-0.25F))));
        graphs.add(new Named("alpha-1.25F",new Alpha(s(),c(1.25F))));
        graphs.add(new Named("alpha-variable",new Alpha(new Add(s(),c(-2.0F)),s())));
        graphs.add(new Named("boost--3",new Boost(s(),-3)));
        graphs.add(new Named("boost-0",new Boost(s(),0)));
        graphs.add(new Named("boost-1",new Boost(s(),1)));
        graphs.add(new Named("boost-2",new Boost(s(),2)));
        graphs.add(new Named("boost-5",new Boost(s(),5)));
        graphs.add(new Named("boost-remap",new Boost(new Add(s(),c(3.0F)),3)));
        graphs.add(new Named("steps-LINEAR",new Steps(s(),c(4.0F),c(0.1F),c(0.9F),Interpolation.LINEAR)));
        graphs.add(new Named("steps-CURVE3",new Steps(s(),c(4.0F),c(0.1F),c(0.9F),Interpolation.CURVE3)));
        graphs.add(new Named("steps-CURVE4",new Steps(s(),c(4.0F),c(0.1F),c(0.9F),Interpolation.CURVE4)));
        graphs.add(new Named("steps-flat",new Steps(s(),c(4.0F),c(0.0F),c(0.0F),Interpolation.CURVE3)));
        graphs.add(new Named("steps-reverse",new Steps(s(),c(2.5F),c(0.8F),c(0.2F),Interpolation.CURVE3)));
        graphs.add(new Named("steps-fractional",new Steps(s(),c(2.5F),c(0.1F),c(0.9F),Interpolation.CURVE3)));
        graphs.add(new Named("steps-negative",new Steps(s(),c(-4.0F),c(0.1F),c(0.9F),Interpolation.CURVE3)));
        graphs.add(new Named("steps-scurve",new Steps(s(),c(3.5F),c(0.1F),c(0.9F),CurveFunctions.scurve(2.0F,-0.5F))));
        graphs.add(new Named("threshold-dynamic",new Threshold(s(),c(-2.0F),new Add(s(),c(3.0F)),c(0.5F))));
        graphs.add(new Named("threshold-equal",new Threshold(c(0.5F),c(-2.0F),new Add(s(),c(3.0F)),c(0.5F))));
        graphs.add(new Named("threshold-below",new Threshold(c(0.25F),c(-2.0F),new Add(s(),c(3.0F)),c(0.5F))));
        graphs.add(new Named("threshold-above",new Threshold(c(0.75F),c(-2.0F),new Add(s(),c(3.0F)),c(0.5F))));
        graphs.add(new Named("blend-LINEAR",new Blend(s(),c(-2.0F),new Add(s(),c(3.0F)),0.4F,0.5F,Interpolation.LINEAR)));
        graphs.add(new Named("blend-CURVE3",new Blend(s(),c(-2.0F),new Add(s(),c(3.0F)),0.4F,0.5F,Interpolation.CURVE3)));
        graphs.add(new Named("blend-CURVE4",new Blend(s(),c(-2.0F),new Add(s(),c(3.0F)),0.4F,0.5F,Interpolation.CURVE4)));
        graphs.add(new Named("blend-shifted",new Blend(new Add(s(),c(3.0F)),s(),c(-2.0F),0.9F,2.0F,Interpolation.CURVE3)));
        graphs.add(new Named("white-graph",new White(0.0125F)));
        graphs.add(new Named("cubic-graph",new Cubic(0.0125F,3,2.0F,0.5F)));
        graphs.add(new Named("PerlinRidge-graph",new PerlinRidge(0.0125F,3,2.0F,0.5F,Interpolation.CURVE3)));
        graphs.add(new Named("SimplexRidge-graph",new SimplexRidge(0.0125F,3,2.0F,0.5F,Interpolation.CURVE3)));
        graphs.add(new Named("Billow-graph",new Billow(0.0125F,3,2.0F,0.5F,Interpolation.CURVE3)));
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
        Files.writeString(Path.of(args[0]),out.toString(),java.nio.charset.StandardCharsets.UTF_8);
    }
}
