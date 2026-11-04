package terraforged.qa;
import java.nio.file.*;
import java.util.*;
import com.google.gson.*;
import com.mojang.serialization.JsonOps;
import com.mojang.datafixers.util.Pair;
import net.minecraft.core.Holder;
import net.minecraft.server.MinecraftServer;
import net.minecraft.util.KeyDispatchDataCodec;
import net.minecraft.world.level.levelgen.*;
import raccoonman.reterraforged.concurrent.cache.CacheManager;
import raccoonman.reterraforged.data.worldgen.preset.settings.Preset;
import raccoonman.reterraforged.registries.RTFRegistries;
import raccoonman.reterraforged.world.worldgen.GeneratorContext;
import raccoonman.reterraforged.world.worldgen.cell.Cell;
import raccoonman.reterraforged.world.worldgen.cell.terrain.TerrainType;
import raccoonman.reterraforged.world.worldgen.densityfunction.*;
import raccoonman.reterraforged.world.worldgen.noise.module.Noise;
/** Genuine source field conversions and custom density functions on controlled leaves. */
public final class NativeDensityPrimitiveProbe {
    private static String f(float v){return Float.isNaN(v)?"NaN":String.format("%08x",Float.floatToRawIntBits(v));}
    private static String d(double v){return Double.isNaN(v)?"NaN":String.format("%016x",Double.doubleToRawLongBits(v));}
    private static float asFloat(JsonElement v){return Float.intBitsToFloat(Integer.parseUnsignedInt(v.getAsString(),16));}
    private static double asDouble(JsonElement v){return Double.longBitsToDouble(Long.parseUnsignedLong(v.getAsString(),16));}
    private static double coordinate(double base,DensityFunction.FunctionContext ctx){return base+ctx.blockX()*0.1D+ctx.blockY()*0.01D+ctx.blockZ()*-0.05D;}
    private record Leaf(int id,double value,double minValue,double maxValue,List<Integer> trace,List<String> positions) implements DensityFunction.SimpleFunction {
        public double compute(DensityFunction.FunctionContext ctx){if(trace!=null)trace.add(id);double result=id<0?value:coordinate(value,ctx);if(positions!=null)positions.add(ctx.blockX()+","+ctx.blockY()+","+ctx.blockZ()+","+d(result));return result;}
        public KeyDispatchDataCodec<? extends DensityFunction> codec(){throw new UnsupportedOperationException("QA leaf has no resource codec");}
    }
    public static void capture(MinecraftServer server,Path values,Path inputFile)throws Exception {
        String input;try(var stream=NativeDensityPrimitiveProbe.class.getResourceAsStream("/terraforged-qa-density-primitives.json")){if(stream==null)throw new IllegalStateException("Explicit density inputs missing");input=new String(stream.readAllBytes(),java.nio.charset.StandardCharsets.UTF_8);}
        Files.writeString(inputFile,input,java.nio.charset.StandardCharsets.UTF_8);var root=JsonParser.parseString(input).getAsJsonObject();var lookup=server.registryAccess().lookupOrThrow(RTFRegistries.NOISE);StringBuilder out=new StringBuilder();int baseline=CacheManager.activeCacheCount();var presets=root.getAsJsonObject("presets");
        for(String name:new TreeSet<>(presets.keySet())) {Preset preset=Preset.DIRECT_CODEC.parse(JsonOps.INSTANCE,presets.get(name)).getOrThrow();try(var context=GeneratorContext.makeUncached(preset,lookup,0,1,1,1)) {var map=context.generator.getHeightmap();var levels=map.levels();var cp=map.controlPoints();out.append("context/").append(name).append('\t').append(levels.worldHeight).append(',').append(f(levels.water)).append(',').append(f(levels.ground)).append(',').append(f(cp.deepOcean())).append(',').append(f(cp.shallowOcean())).append(',').append(f(cp.beach())).append(',').append(f(cp.coast())).append(',').append(f(cp.inland())).append('\n');
            for(int terrain=0;terrain<18;terrain++)for(var h:root.getAsJsonArray("heightBits"))for(var e:root.getAsJsonArray("edgeBits")){int b=0;for(var beach:root.getAsJsonArray("beachNoises")){Cell c=new Cell();c.height=asFloat(h);c.continentEdge=asFloat(e);c.beachNoise=beach.getAsFloat();c.terrain=TerrainType.get(terrain);c.erosion=-3.25F;c.weirdness=-0.0F;c.biomeRegionId=0.125F;c.temperature=1.25F;c.moisture=-0.25F;c.gradient=0.75F;c.heightErosion=-0.1F;c.sediment=0.3F;out.append("field/").append(name).append('/').append(terrain).append('/').append(h.getAsString()).append('/').append(e.getAsString()).append('/').append(b++).append('\t');int n=0;for(var field:CellSampler.Field.values()){if(n++>0)out.append(',');out.append(f(field.read(c,map)));}out.append('\n');}}
        }}
        var ctx=new MutableFunctionContext();for(var resolution:root.getAsJsonArray("resolutions"))for(var value:root.getAsJsonArray("doubleBits")){var fn=new ClampToNearestUnit(new Leaf(-1,asDouble(value),asDouble(value),asDouble(value),null,null),resolution.getAsInt());out.append("clamp/").append(resolution.getAsInt()).append('/').append(value.getAsString()).append('\t').append(d(fn.compute(ctx.at(0,0,0)))).append(',').append(d(fn.minValue())).append(',').append(d(fn.maxValue())).append('\n');}
        int splineIndex=0;for(var element:root.getAsJsonArray("splineCases")){var s=element.getAsJsonObject();var locations=s.getAsJsonArray("locationBits");var leaves=s.getAsJsonArray("valueBits");var bounds=s.getAsJsonArray("boundsBits");List<Integer> trace=new ArrayList<>();List<Pair<Double,DensityFunction>> points=new ArrayList<>();for(int i=0;i<locations.size();i++){var pair=bounds.get(i).getAsJsonArray();points.add(Pair.of(asDouble(locations.get(i)),new Leaf(i,asDouble(leaves.get(i)),asDouble(pair.get(0)),asDouble(pair.get(1)),trace,null)));}
            for(var value:root.getAsJsonArray("doubleBits")){var fn=new LinearSplineFunction(new Leaf(-1,asDouble(value),asDouble(value),asDouble(value),trace,null),points);int p=0;for(var position:root.getAsJsonArray("positions")){var a=position.getAsJsonArray();trace.clear();String result;try{result=d(fn.compute(ctx.at(a.get(0).getAsInt(),a.get(1).getAsInt(),a.get(2).getAsInt())));}catch(IndexOutOfBoundsException expected){result="rejected";}out.append("spline/").append(splineIndex).append('/').append(value.getAsString()).append('/').append(p++).append('\t').append(result).append(',').append(d(fn.minValue())).append(',').append(d(fn.maxValue())).append(',').append(trace.stream().map(Object::toString).reduce((a1,b1)->a1+":"+b1).orElse("none")).append('\n');}}
            splineIndex++;
        }
        int originIndex=0;for(var origin:root.getAsJsonArray("cacheOrigins")){var a=origin.getAsJsonArray();int ox=a.get(0).getAsInt(),oz=a.get(1).getAsInt();for(var widthInput:root.getAsJsonArray("cacheWidths")){int width=widthInput.getAsInt();List<String> calls=new ArrayList<>();var wrapper=new ConditionalArrayCache(new Leaf(0,0.125,-1e9,1e9,null,calls));var cache=wrapper.new Cache(ox,oz,width);String prefix=originIndex+"/"+width;for(int i=0;i<calls.size();i++)out.append("array-init/").append(prefix).append('/').append(i).append('\t').append(calls.get(i)).append('\n');out.append("array-bounds/").append(prefix).append('\t').append(d(cache.minValue())).append(',').append(d(cache.maxValue())).append('\n');int[] offsets={-1,0,1,width-1,width,width+1};for(int x=0;x<offsets.length;x++)for(int z=0;z<offsets.length;z++){int y=0;for(var height:root.getAsJsonArray("cacheYs")){int before=calls.size();double result=cache.compute(ctx.at(ox+offsets[x],height.getAsInt(),oz+offsets[z]));out.append("array/").append(prefix).append('/').append(x).append('/').append(z).append('/').append(y++).append('\t').append(d(result)).append(',').append(calls.size()-before).append('\n');}}}originIndex++;}
        var noises=new TreeMap<String,Noise>();lookup.listElements().forEach(holder->noises.put(holder.key().identifier().toString(),holder.value()));for(String name:new TreeSet<>(root.getAsJsonObject("noiseRegistry").keySet()))for(var seed:root.getAsJsonArray("noiseSeeds")){var fn=new NoiseFunction(Holder.direct(Objects.requireNonNull(noises.get(name))),seed.getAsInt());int p=0;for(var position:root.getAsJsonArray("positions")){var a=position.getAsJsonArray();out.append("noise/").append(name).append('/').append(seed.getAsInt()).append('/').append(p++).append('\t').append(d(fn.compute(ctx.at(a.get(0).getAsInt(),a.get(1).getAsInt(),a.get(2).getAsInt())))).append(',').append(d(fn.minValue())).append(',').append(d(fn.maxValue())).append('\n');}}
        if(CacheManager.activeCacheCount()!=baseline)throw new IllegalStateException("Density oracle leaked context caches");Files.writeString(values,out.toString(),java.nio.charset.StandardCharsets.UTF_8);
    }
}
