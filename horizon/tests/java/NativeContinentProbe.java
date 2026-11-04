package raccoonman.reterraforged.world.worldgen.cell.continent.advanced;

import java.nio.file.*;
import java.lang.reflect.*;
import java.util.*;
import com.google.gson.*;
import com.mojang.serialization.JsonOps;
import net.minecraft.server.MinecraftServer;
import raccoonman.reterraforged.concurrent.cache.Cache;
import raccoonman.reterraforged.concurrent.cache.CacheManager;
import raccoonman.reterraforged.data.worldgen.preset.settings.Preset;
import raccoonman.reterraforged.registries.RTFRegistries;
import raccoonman.reterraforged.world.worldgen.GeneratorContext;
import raccoonman.reterraforged.world.worldgen.cell.Cell;
import raccoonman.reterraforged.world.worldgen.cell.biome.type.BiomeType;
import raccoonman.reterraforged.world.worldgen.cell.terrain.Terrain;
import raccoonman.reterraforged.world.worldgen.noise.NoiseUtil;
import raccoonman.reterraforged.world.worldgen.util.PosUtil;
import raccoonman.reterraforged.world.worldgen.util.Seed;

/** Original immutable continent geometry, including dirty-cell and river-search observations. */
public final class NativeContinentProbe {
    private static final int[] SEEDS={0,-1,Integer.MIN_VALUE,Integer.MAX_VALUE,8675309,2147483645};
    private static final float[] COORDS={-30_000_000F,-60_000F,-16_384F,-12_000F,-4096F,-3000F,-1024F,-257F,-17F,-1F,-0F,0F,.25F,1F,17F,257F,1024F,3000F,4096F,12_000F,16_384F,60_000F,30_000_000F};
    private static final Field[] FIELDS=Arrays.stream(Cell.class.getFields()).filter(f->!Modifier.isStatic(f.getModifiers())).sorted(Comparator.comparing(Field::getName)).toArray(Field[]::new);
    private static String bits(float f) {return Float.isNaN(f)?"NaN":String.format("%08x",Float.floatToRawIntBits(f));}
    private static Cell cell(boolean dirty) throws Exception {
        Cell c=new Cell();
        if(dirty) for(Field f:FIELDS) {
            if(f.getType()==float.class) f.setFloat(c,-3.25F);
            else if(f.getType()==int.class) f.setInt(c,-123456);
            else if(f.getType()==boolean.class) f.setBoolean(c,true);
        }
        return c;
    }
    private static void snapshot(String key,Cell c,StringBuilder out) throws Exception {
        out.append(key).append('\t');
        for(int i=0;i<FIELDS.length;i++) {
            Object v=FIELDS[i].get(c);if(i>0) out.append(',');
            out.append(v instanceof Float f?bits(f):v instanceof Terrain t?t.getId():v instanceof BiomeType b?b.ordinal():v);
        }
        out.append('\n');
    }
    public static void capture(MinecraftServer server,Path values,Path inputFile) throws Exception {
        String input;
        try(var stream=NativeContinentProbe.class.getResourceAsStream("/terraforged-qa-continent.json")) {
            if(stream==null) throw new IllegalStateException("Explicit continent input missing");
            input=new String(stream.readAllBytes(),java.nio.charset.StandardCharsets.UTF_8);
        }
        Files.writeString(inputFile,input,java.nio.charset.StandardCharsets.UTF_8);
        JsonObject cases=JsonParser.parseString(input).getAsJsonObject();StringBuilder out=new StringBuilder();
        if(FIELDS.length!=24) throw new IllegalStateException("Cell schema changed");
        var lookup=server.registryAccess().lookupOrThrow(RTFRegistries.NOISE);
        int cacheBaseline=CacheManager.activeCacheCount();
        for(String name:new TreeSet<>(cases.keySet())) for(int root:SEEDS) {
            Preset preset=Preset.DIRECT_CODEC.parse(JsonOps.INSTANCE,cases.get(name)).getOrThrow();
            try(var context=GeneratorContext.makeUncached(preset,lookup,root,1,1,1)) {
                Seed seed=new Seed(root);
                var captured=CacheManager.capture(()->new AdvancedContinentGenerator(seed,context));
                try {
                    var continent=captured.value();String prefix=name+"/"+root;
                    out.append("cursor/").append(prefix).append('\t').append(seed.get()).append('\n');
                    for(int i=0;i<COORDS.length;i++) for(int j=0;j<COORDS.length;j++) for(int dirty=0;dirty<2;dirty++) {
                        Cell c=cell(dirty==1);continent.apply(c,COORDS[i],COORDS[j]);
                        snapshot("cell/"+prefix+"/"+i+"/"+j+"/"+dirty,c,out);
                    }
                    for(int point=0;point<3;point++) {
                        float x=point==0?0:point==1?-60000:60000;
                        float z=point==0?0:point==1?60000:-60000;
                        long center=continent.getNearestCenter(x,z);int cx=PosUtil.unpackLeft(center),cz=PosUtil.unpackRight(center);
                        out.append("query/").append(prefix).append('/').append(point).append('\t').append(cx).append(',').append(cz).append(',').append(bits(continent.getEdgeValue(x,z))).append('\n');
                        for(int dir=0;dir<8;dir++) {
                            float angle=dir*(NoiseUtil.PI2/8.0F);float dx=NoiseUtil.sin(angle),dz=NoiseUtil.cos(angle);
                            out.append("search/").append(prefix).append('/').append(point).append('/').append(dir).append('\t').append(bits(continent.getDistanceToEdge(cx,cz,dx,dz))).append(',').append(bits(continent.getDistanceToOcean(cx,cz,dx,dz))).append('\n');
                        }
                    }
                } finally {captured.caches().forEach(Cache::close);}
            }
        }
        if(CacheManager.activeCacheCount()!=cacheBaseline) throw new IllegalStateException("Oracle leaked generator cache ownership");
        float[] v={-100F,-1F,-0F,0F,.25F,1F,100F};
        for(int i=0;i<v.length;i++) for(int j=0;j<v.length;j++) {
            out.append("geometry/").append(i).append('/').append(j).append('\t').append(bits(AdvancedContinentGenerator.getDistance(v[i],v[j],-1F,2F,3F,-4F))).append(',').append(bits(AdvancedContinentGenerator.getDistance2Line(v[i],v[j],-1F,2F,3F,-4F))).append('\n');
        }
        Files.writeString(values,out.toString(),java.nio.charset.StandardCharsets.UTF_8);
    }
}
