package raccoonman.reterraforged.world.worldgen.noise.module;

import raccoonman.reterraforged.world.worldgen.noise.function.Interpolation;

/** Independent observer of genuine Java source records; no native expected data. */
public final class NativeNoiseSourcesProbe {
    private static final int[] SEEDS = {0,1,-1,42,Integer.MIN_VALUE,Integer.MAX_VALUE};
    private static final float[] COORDS = {-30_000_000f,-17f,-16.25f,-16f,-1.5f,-1f,-.25f,-0f,
        0f,.25f,1f,15f,15.75f,16f,16.25f,17f,30_000_000f};
    private static String bits(float f) { return String.format("%08x",Float.floatToRawIntBits(f)); }
    private static void metadata(String key, Noise n) {
        System.out.println("meta/"+key+"\t"+bits(n.minValue())+","+bits(n.maxValue()));
    }
    public static void main(String[] args) {
        float[][] params = {{1f,0f,2f,.5f},{1f,1f,2f,.5f},{.0125f,3f,2f,.5f},{.001f,7f,2.5f,.75f},
            {.0000001f,30f,1.1f,.975f},{.0000001f,31f,1.1f,.975f},{.0000001f,32f,1.1f,.975f}};
        for (int p=0;p<params.length;p++) {
            float[] f=params[p];
            White white = new White(f[0]);
            Cubic cubic = new Cubic(f[0],(int)f[1],f[2],f[3]);
            metadata("white/"+p,white);
            metadata("cubic/"+p,cubic);
            for (Interpolation mode : Interpolation.values()) {
                PerlinRidge perlin = new PerlinRidge(f[0],(int)f[1],f[2],f[3],mode);
                Billow billow = new Billow(f[0],(int)f[1],f[2],f[3],mode);
                SimplexRidge simplex = p<5 ? new SimplexRidge(f[0],(int)f[1],f[2],f[3],mode) : null;
                String prefix=mode.ordinal()+"/"+p;
                metadata("perlin_ridge/"+prefix,perlin);
                metadata("billow/"+prefix,billow);
                if (simplex!=null) metadata("simplex_ridge/"+prefix,simplex);
                for(int seed:SEEDS) for(int i=0;i<COORDS.length;i++) for(int j=0;j<COORDS.length;j++) {
                    float x=COORDS[i],z=COORDS[j];
                    String key=prefix+"/"+seed+"/"+i+"/"+j;
                    System.out.println("perlin_ridge/compute/"+key+"\t"+bits(perlin.compute(x,z,seed)));
                    System.out.println("billow/compute/"+key+"\t"+bits(billow.compute(x,z,seed)));
                    if(simplex!=null) System.out.println("simplex_ridge/compute/"+key+"\t"+bits(simplex.compute(x,z,seed)));
                    if(mode.ordinal()==0) {
                        key=p+"/"+seed+"/"+i+"/"+j;
                        System.out.println("white/compute/"+key+"\t"+bits(white.compute(x,z,seed)));
                        System.out.println("cubic/compute/"+key+"\t"+bits(cubic.compute(x,z,seed)));
                        if(p==0) {
                            System.out.println("white/sample/"+key+"\t"+bits(White.sample(x,z,seed))+","+bits(White.sample(x,z,seed,Integer.MAX_VALUE)));
                            System.out.println("cubic/sample/"+key+"\t"+bits(Cubic.sample(x,z,seed)));
                        }
                    }
                }
            }
        }
    }
}
