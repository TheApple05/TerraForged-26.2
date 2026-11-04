package terraforged.qa;
import java.nio.file.*;
import java.util.*;
import com.google.gson.*;
import com.mojang.serialization.JsonOps;
import net.minecraft.core.Holder;
import net.minecraft.server.MinecraftServer;
import net.minecraft.world.level.levelgen.*;
import raccoonman.reterraforged.concurrent.cache.CacheManager;
import raccoonman.reterraforged.data.worldgen.preset.settings.Preset;
import raccoonman.reterraforged.registries.RTFRegistries;
import raccoonman.reterraforged.world.worldgen.GeneratorContext;
import raccoonman.reterraforged.world.worldgen.densityfunction.*;
import raccoonman.reterraforged.world.worldgen.noise.module.Noise;
/** Genuine mapped density functions over generated/filtered tiles, no native outputs. */
public final class NativeHostDensityProbe {
    private static String bits(double value) {float f=(float)value;return Float.isNaN(f)?"NaN":String.format("%08x",Float.floatToRawIntBits(f));}
    private record Volume(int[] min,int[] size,int[] step) implements DensityFunction.ContextProvider {
        public DensityFunction.FunctionContext forIndex(int i) {int y=i%size[1];i/=size[1];int x=i%size[0],z=i/size[0];return new DensityFunction.SinglePointContext(min[0]+x*step[0],min[1]+y*step[1],min[2]+z*step[2]);}
        public void fillAllDirectly(double[] output,DensityFunction fn) {for(int i=0;i<output.length;i++)output[i]=fn.compute(forIndex(i));}
        int count(){return size[0]*size[1]*size[2];}
    }
    private static int[] ints(JsonArray a){return new int[]{a.get(0).getAsInt(),a.get(1).getAsInt(),a.get(2).getAsInt()};}
    public static void capture(MinecraftServer server,Path values,Path inputFile)throws Exception {
        String input;try(var s=NativeHostDensityProbe.class.getResourceAsStream("/terraforged-qa-host-density.json")){if(s==null)throw new IllegalStateException("Host density configuration missing");input=new String(s.readAllBytes(),java.nio.charset.StandardCharsets.UTF_8);}
        Files.writeString(inputFile,input);var root=JsonParser.parseString(input).getAsJsonObject();var lookup=server.registryAccess().lookupOrThrow(RTFRegistries.NOISE);
        Preset preset=Preset.DIRECT_CODEC.parse(JsonOps.INSTANCE,root.get("preset")).getOrThrow();
        var noises=new TreeMap<String,Noise>();lookup.listElements().forEach(h->noises.put(h.key().identifier().toString(),h.value()));
        var ops=net.minecraft.resources.RegistryOps.create(JsonOps.INSTANCE,server.registryAccess());
        for(var entry:root.getAsJsonObject("noiseRegistry").entrySet()) {
            var expected=entry.getValue().isJsonPrimitive()?JsonParser.parseString(entry.getValue().getAsString()):entry.getValue();
            var actual=Noise.DIRECT_CODEC.encodeStart(ops,Objects.requireNonNull(noises.get(entry.getKey()))).getOrThrow();
            // Gson Float-valued primitives and parsed decimal primitives use different
            // doubleValue representations. Compare the actual serialized JSON form.
            if(!JsonParser.parseString(actual.toString()).equals(expected))throw new IllegalStateException("Runtime noise binding differs: "+entry.getKey()+" actual="+actual+" expected="+expected);
        }
        StringBuilder out=new StringBuilder();int owners=CacheManager.activeCacheCount();
        for(var seedInput:root.getAsJsonArray("seeds")){long worldSeed=seedInput.getAsLong();try(var ctx=GeneratorContext.makeCached(preset.copy(),lookup,(int)worldSeed,3,6,false)) {
            var functions=new TreeMap<String,DensityFunction>();
            for(var field:CellSampler.Field.values())functions.put(field.getSerializedName(),new CellSampler(()->ctx.lookup,field));
            var height=functions.get("height");for(var resolution:root.getAsJsonArray("resolutions"))functions.put("clamp_"+resolution.getAsInt(),new ClampToNearestUnit(height,resolution.getAsInt()));
            functions.put("height_plus_half",DensityFunctions.add(height,DensityFunctions.constant(0.5)));
            for(String name:new TreeSet<>(root.getAsJsonObject("noiseRegistry").keySet()))functions.put("noise_"+name,new NoiseFunction(Holder.direct(noises.get(name)),(int)worldSeed));
            CellSampler.withAccurateSampling(()->{
                for(var entry:functions.entrySet()){
                    int p=0;for(var x:root.getAsJsonArray("coordinates"))for(var z:root.getAsJsonArray("coordinates"))for(var y:root.getAsJsonArray("heights")){
                        out.append("scalar/").append(worldSeed).append('/').append(entry.getKey()).append('/').append(p++).append('\t').append(bits(entry.getValue().compute(new DensityFunction.SinglePointContext(x.getAsInt(),y.getAsInt(),z.getAsInt())))).append('\n');
                    }
                    int vi=0;for(var v:root.getAsJsonArray("volumes")){var o=v.getAsJsonObject();var volume=new Volume(ints(o.getAsJsonArray("minimum")),ints(o.getAsJsonArray("sizes")),ints(o.getAsJsonArray("steps")));double[] buffer=new double[volume.count()];entry.getValue().fillArray(buffer,volume);for(int i=0;i<buffer.length;i++)out.append("volume/").append(worldSeed).append('/').append(entry.getKey()).append('/').append(vi).append('/').append(i).append('\t').append(bits(buffer[i])).append('\n');vi++;}
                }
                return null;
            });
        }}
        if(CacheManager.activeCacheCount()!=owners)throw new IllegalStateException("Host oracle leaked managed caches");
        Files.writeString(values,out.toString(),java.nio.charset.StandardCharsets.UTF_8);
    }
}
