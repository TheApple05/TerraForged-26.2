package terraforged.qa;
import java.nio.file.*;
import java.lang.reflect.*;
import java.util.*;
import com.google.gson.*;
import com.mojang.serialization.JsonOps;
import net.minecraft.server.MinecraftServer;
import raccoonman.reterraforged.concurrent.cache.CacheManager;
import raccoonman.reterraforged.data.worldgen.preset.PresetTerrainTypeNoise;
import raccoonman.reterraforged.data.worldgen.preset.PresetClimateNoise;
import raccoonman.reterraforged.data.worldgen.preset.settings.Preset;
import raccoonman.reterraforged.registries.RTFRegistries;
import raccoonman.reterraforged.world.worldgen.GeneratorContext;
import raccoonman.reterraforged.world.worldgen.cell.*;
import raccoonman.reterraforged.world.worldgen.cell.biome.type.BiomeType;
import raccoonman.reterraforged.world.worldgen.cell.terrain.*;
import raccoonman.reterraforged.world.worldgen.noise.module.Noise;
/** Genuine complete terrain and climate stages; deliberately does not run rivers/tiles. */
public final class NativeHeightmapClimateProbe {
    private static final Field[] FIELDS=Arrays.stream(Cell.class.getFields()).filter(f->!Modifier.isStatic(f.getModifiers())).sorted(Comparator.comparing(Field::getName)).toArray(Field[]::new);
    private static String bits(float f) {return Float.isNaN(f)?"NaN":String.format("%08x",Float.floatToRawIntBits(f));}
    private static Cell cell(boolean dirty) throws Exception {Cell c=new Cell();if(dirty) for(Field f:FIELDS) {
        if(f.getType()==float.class) f.setFloat(c,-3.25F);else if(f.getType()==int.class) f.setInt(c,-123456);else if(f.getType()==boolean.class) f.setBoolean(c,true);
    }return c;}
    private static void snapshot(String key,Cell c,StringBuilder out) throws Exception {
        out.append(key).append('\t');for(int i=0;i<FIELDS.length;i++) {if(i>0) out.append(',');Object v=FIELDS[i].get(c);out.append(v instanceof Float f?bits(f):v instanceof Terrain t?t.getId():v instanceof BiomeType b?b.ordinal():v);}out.append('\n');
    }
    public static void capture(MinecraftServer server,Path values,Path inputFile) throws Exception {
        String input;try(var stream=NativeHeightmapClimateProbe.class.getResourceAsStream("/terraforged-qa-heightmap-climate.json")) {
            if(stream==null) throw new IllegalStateException("Explicit heightmap inputs missing");input=new String(stream.readAllBytes(),java.nio.charset.StandardCharsets.UTF_8);
        }
        Files.writeString(inputFile,input,java.nio.charset.StandardCharsets.UTF_8);var root=JsonParser.parseString(input).getAsJsonObject();
        var lookup=server.registryAccess().lookupOrThrow(RTFRegistries.NOISE);var binding=root.getAsJsonObject("noiseRegistry");
        for(var key:List.of(PresetTerrainTypeNoise.GROUND,PresetClimateNoise.BIOME_EDGE_SHAPE)) {
            var actual=Noise.DIRECT_CODEC.encodeStart(JsonOps.INSTANCE,lookup.getOrThrow(key).value()).getOrThrow();
            // Compare the serialized binding, not boxed Float.doubleValue against
            // parsed decimal doubleValue (e.g. 2.65f has different widened bits).
            if(!JsonParser.parseString(actual.toString()).equals(JsonParser.parseString(binding.get(key.identifier().toString()).getAsString()))) throw new IllegalStateException("Actual registry binding differs: "+key);
        }
        var cases=root.getAsJsonObject("presets");var coords=root.getAsJsonArray("coordinates");StringBuilder out=new StringBuilder();int baseline=CacheManager.activeCacheCount();
        for(String name:new TreeSet<>(cases.keySet())) for(var seedElement:root.getAsJsonArray("worldSeeds")) {
            long worldSeed=seedElement.getAsLong();int seed=(int)worldSeed;Preset preset=Preset.DIRECT_CODEC.parse(JsonOps.INSTANCE,cases.get(name)).getOrThrow();String prefix=name+"/"+worldSeed;
            try(var context=GeneratorContext.makeUncached(preset,lookup,seed,1,1,1)) {
                var map=context.generator.getHeightmap();out.append("context/").append(prefix).append('\t').append(seed).append(',').append(context.seed.get()).append(',').append(map.climate().randomSeed()).append('\n');
                for(int i=0;i<coords.size();i++) for(int j=0;j<coords.size();j++) for(int dirty=0;dirty<2;dirty++) {
                    float x=coords.get(i).getAsFloat(),z=coords.get(j).getAsFloat();Cell terrain=cell(dirty==1);map.applyTerrain(terrain,x,z);
                    String key=prefix+"/"+i+"/"+j+"/"+dirty;snapshot("terrain/"+key,terrain,out);
                    Cell yes=new Cell();yes.copyFrom(terrain);map.applyClimate(yes,x,z,true);snapshot("climate-true/"+key,yes,out);
                    Cell no=new Cell();no.copyFrom(terrain);map.applyClimate(no,x,z,false);snapshot("climate-false/"+key,no,out);
                }
                var synthetic=root.getAsJsonObject("climateInputs");var ts=synthetic.getAsJsonArray("terrainIds");var hs=synthetic.getAsJsonArray("heights");var rs=synthetic.getAsJsonArray("riverMasks");var bs=synthetic.getAsJsonArray("macroBiomeIds");var ps=synthetic.getAsJsonArray("positions");
                for(int t=0;t<ts.size();t++) for(int h=0;h<hs.size();h++) for(int r=0;r<rs.size();r++) for(int b=0;b<bs.size();b++) for(int p=0;p<ps.size();p++) for(int enabled=0;enabled<2;enabled++) {
                    Cell c=cell(true);c.terrain=TerrainType.get(ts.get(t).getAsInt());c.height=hs.get(h).getAsFloat();c.riverMask=rs.get(r).getAsFloat();c.macroBiomeId=bs.get(b).getAsFloat();var pos=ps.get(p).getAsJsonArray();map.applyClimate(c,pos.get(0).getAsFloat(),pos.get(1).getAsFloat(),enabled==1);
                    snapshot("synthetic/"+prefix+"/"+t+"/"+h+"/"+r+"/"+b+"/"+p+"/"+enabled,c,out);
                }
            }
        }
        if(CacheManager.activeCacheCount()!=baseline) throw new IllegalStateException("Heightmap oracle leaked context caches");
        Files.writeString(values,out.toString(),java.nio.charset.StandardCharsets.UTF_8);
    }
}
