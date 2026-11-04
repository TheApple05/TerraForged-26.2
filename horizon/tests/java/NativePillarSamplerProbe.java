import java.nio.charset.StandardCharsets;
import java.nio.file.*;
import java.lang.reflect.*;
import java.util.*;
import net.minecraft.core.registries.BuiltInRegistries;
import net.minecraft.world.level.block.Blocks;
import net.minecraft.world.level.block.state.BlockState;
import raccoonman.reterraforged.world.worldgen.feature.ClassicPillarFeature;
import raccoonman.reterraforged.world.worldgen.feature.ClassicPillarMath;
import raccoonman.reterraforged.world.worldgen.noise.module.Noise;
import raccoonman.reterraforged.world.worldgen.noise.module.Noises;
/** Reads packaged static graphs and private palettes; detail/band expressions follow place(). */
public final class NativePillarSamplerProbe {
    record Sampler(Noise detail,Noise bands,Object[] palettes) {}
    static String bits(float f) {return String.format(Locale.ROOT,"%08x",Float.floatToRawIntBits(f));}
    static Noise graph(String name) throws Exception {Field f=ClassicPillarFeature.class.getDeclaredField(name);f.setAccessible(true);return (Noise)f.get(null);}
    static String id(BlockState state) {return BuiltInRegistries.BLOCK.getKey(state.getBlock()).toString();}
    public static void main(String[] args) throws Exception {
        net.minecraft.SharedConstants.tryDetectVersion();net.minecraft.server.Bootstrap.bootStrap();
        Path jar=Path.of(args[1]).toRealPath();
        for(Class<?> type:List.of(ClassicPillarFeature.class,ClassicPillarMath.class,Noises.class)) {
            Path origin=Path.of(type.getProtectionDomain().getCodeSource().getLocation().toURI()).toRealPath();
            if(!origin.equals(jar))throw new AssertionError("Wrong packaged origin "+type);
            System.err.println("PACKAGED_CLASS_ORIGIN="+type.getName()+"="+origin);
        }
        Noise bryce=graph("BRYCE"),a=graph("STONE_A"),b=graph("STONE_B");
        Method palettes=ClassicPillarFeature.class.getDeclaredMethod("palettes",int.class);palettes.setAccessible(true);
        Class<?> paletteClass=Arrays.stream(ClassicPillarFeature.class.getDeclaredClasses()).filter(c->c.getSimpleName().equals("Palette")).findFirst().orElseThrow();
        Method ends=paletteClass.getDeclaredMethod("ends");ends.setAccessible(true);
        Method at=paletteClass.getDeclaredMethod("at",int.class,int.class);at.setAccessible(true);
        Map<Integer,Sampler> cache=new HashMap<>();StringBuilder output=new StringBuilder();int count=0;
        for(String line:Files.readAllLines(Path.of(args[0]),StandardCharsets.UTF_8)) {
            String[] f=line.split("\\t");String value;
            if(f[1].equals("index")) {float region=Float.intBitsToFloat(Integer.parseUnsignedInt(f[2],16));value=Integer.toString(Math.min(8,(int)(Math.max(0,Math.min(0.999999F,region))*9)));}
            else if(f[1].equals("stone")) {int y=Integer.parseInt(f[2]),height=Integer.parseInt(f[3]);value=id(y==height?Blocks.GRASS_BLOCK.defaultBlockState():y==height-1?Blocks.DIRT.defaultBlockState():Blocks.STONE.defaultBlockState());}
            else {
                int seed=Integer.parseInt(f[2]);Sampler sampler=cache.get(seed);
                if(sampler==null) {sampler=new Sampler(Noises.alpha(Noises.perlin(seed+12341343,4,1),0.05F),Noises.perlin(seed+12341345,100,1),(Object[])palettes.invoke(null,seed));cache.put(seed,sampler);}
                if(f[1].equals("ends")) {StringJoiner values=new StringJoiner(",");for(float end:(float[])ends.invoke(sampler.palettes()[Integer.parseInt(f[3])]))values.add(bits(end));value=values.toString();}
                else if(f[1].equals("material"))value=id((BlockState)at.invoke(sampler.palettes()[Integer.parseInt(f[3])],Integer.parseInt(f[4]),Integer.parseInt(f[5])));
                else {int x=Integer.parseInt(f[3]),z=Integer.parseInt(f[4]);value=switch(f[1]) {
                    case "shape" -> bits(f[5].equals("1")?Math.max(ClassicPillarMath.terrace(a.compute(x,z,seed),1),ClassicPillarMath.terrace(b.compute(x,z,seed),1)):ClassicPillarMath.terrace(bryce.compute(x,z,seed),0.999F)*sampler.detail().compute(x,z,seed));
                    case "bands" -> bits(sampler.bands().compute(x,z,seed));
                    case "shift" -> Integer.toString((int)(sampler.bands().compute(x,z,seed)*4));
                    default -> throw new IllegalArgumentException("Unknown operation "+f[1]);
                };}
            }
            output.append(f[0]).append('\t').append(value).append('\n');count++;
        }
        Files.writeString(Path.of(args[2]),output,StandardCharsets.UTF_8,StandardOpenOption.CREATE_NEW);
        System.out.println("PILLAR_SAMPLER_CAPTURE_OK records="+count+" pid="+ProcessHandle.current().pid());
    }
}
