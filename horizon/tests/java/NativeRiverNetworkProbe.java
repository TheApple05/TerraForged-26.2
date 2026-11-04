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
import raccoonman.reterraforged.world.worldgen.cell.Cell;
import raccoonman.reterraforged.world.worldgen.cell.biome.type.BiomeType;
import raccoonman.reterraforged.world.worldgen.cell.terrain.*;
import raccoonman.reterraforged.world.worldgen.cell.continent.SimpleContinent;
import raccoonman.reterraforged.world.worldgen.cell.continent.simple.SimpleRiverGenerator;
import raccoonman.reterraforged.world.worldgen.cell.rivermap.Rivermap;
import raccoonman.reterraforged.world.worldgen.cell.rivermap.gen.GenWarp;
import raccoonman.reterraforged.world.worldgen.cell.rivermap.river.*;
import raccoonman.reterraforged.world.worldgen.noise.module.Noise;
import raccoonman.reterraforged.world.worldgen.util.*;
/** Actual cached maps and original construction methods, not a replacement generator. */
public final class NativeRiverNetworkProbe {
    private static final Field[] FIELDS=Arrays.stream(Cell.class.getFields()).filter(f->!Modifier.isStatic(f.getModifiers())).sorted(Comparator.comparing(Field::getName)).toArray(Field[]::new);
    private static String bits(float f){return Float.isNaN(f)?"NaN":String.format("%08x",Float.floatToRawIntBits(f));}
    private static Object field(Object object,String key)throws Exception{Field f=object.getClass().getDeclaredField(key);f.setAccessible(true);return f.get(object);}
    private static void floats(String key,StringBuilder out,float... values){out.append(key).append('\t');for(int i=0;i<values.length;i++){if(i>0)out.append(',');out.append(bits(values[i]));}out.append('\n');}
    private static Cell cell(JsonObject v)throws Exception{Cell c=new Cell();for(Field f:FIELDS){if(f.getType()==float.class)f.setFloat(c,-3.25F);else if(f.getType()==int.class)f.setInt(c,-123456);}c.height=v.get("height").getAsFloat();c.riverMask=v.get("mask").getAsFloat();c.terrain=TerrainType.get(v.get("terrain").getAsInt());c.continentEdge=v.get("edge").getAsFloat();return c;}
    private static void snapshot(String key,Cell c,StringBuilder out)throws Exception{out.append(key).append('\t');for(int i=0;i<FIELDS.length;i++){if(i>0)out.append(',');Object v=FIELDS[i].get(c);out.append(v instanceof Float f?bits(f):v instanceof Terrain t?t.getId():v instanceof BiomeType b?b.ordinal():v);}out.append('\n');}
    private static void tree(String prefix,Network n,JsonObject root,StringBuilder out)throws Exception{
        var c=n.riverCarver();var r=c.river;var cfg=c.config;var b=n.bounds();
        out.append("node/").append(prefix).append('\t').append(n.children().length).append(',').append(n.lakes().length).append(',').append(n.wetlands().length).append('\n');
        floats("river/"+prefix,out,r.dx,r.dz,r.length,r.length2,r.maxX,r.maxZ,r.minX,r.minZ,r.ndx,r.ndz,r.normX,r.normZ,r.x1,r.x2,r.z1,r.z2,r.getAngle());
        floats("bounds/"+prefix,out,b.minX(),b.minY(),b.maxX(),b.maxY());
        out.append("config/").append(prefix).append('\t').append(cfg.bankWidth).append(',').append(bits(cfg.bedHeight)).append(',').append(cfg.bedWidth).append(',').append(bits(cfg.fade)).append(',').append(cfg.length).append(',').append(cfg.length2).append(',').append(cfg.main).append(',').append(bits(cfg.maxBankHeight)).append(',').append(bits(cfg.minBankHeight)).append(',').append(cfg.order).append('\n');
        out.append("warp/").append(prefix).append('\t').append(field(c.warp,"seed"));for(String key:List.of("lower","upper","lowerRange","upperRange","frequency","scale"))out.append(',').append(bits((float)field(c.warp,key)));out.append('\n');
        out.append("curve/").append(prefix).append('\t');int index=0;for(var v:root.getAsJsonArray("curveValues")){if(index++>0)out.append(',');out.append(bits(c.valleyCurve.apply(v.getAsFloat())));}out.append('\n');
        for(int i=0;i<n.lakes().length;i++){var builder=Boundsf.builder();n.lakes()[i].recordBounds(builder);var box=builder.build();floats("lake/"+prefix+"/"+i,out,box.minX(),box.minY(),box.maxX(),box.maxY());}
        for(int i=0;i<n.wetlands().length;i++){var builder=Boundsf.builder();n.wetlands()[i].recordBounds(builder);var box=builder.build();floats("wetland/"+prefix+"/"+i,out,box.minX(),box.minY(),box.maxX(),box.maxY());}
        for(int i=0;i<n.children().length;i++)tree(prefix+"/"+i,n.children()[i],root,out);
    }
    private static void sample(String key,Rivermap map,float x,float z,JsonObject root,StringBuilder out)throws Exception{
        int i=0;for(var v:root.getAsJsonArray("cellCases")){Cell c=cell(v.getAsJsonObject());map.apply(c,x,z);snapshot("map/"+key+"/"+i++,c,out);}
    }
    private static void lines(String key,Network n,Rivermap map,JsonObject root,StringBuilder out)throws Exception{
        var r=n.riverCarver().river;int t=0;for(var value:root.getAsJsonArray("lineT")){float alpha=value.getAsFloat();int d=0;for(var offset:root.getAsJsonArray("lineOffsets")){float distance=offset.getAsFloat();float x=r.x1+r.dx*alpha+r.normX*distance,z=r.z1+r.dz*alpha+r.normZ*distance;sample("line/"+key+"/"+t+"/"+d++,map,x,z,root,out);}t++;}
        for(int i=0;i<n.children().length;i++)lines(key+"/"+i,n.children()[i],map,root,out);
    }
    public static void capture(MinecraftServer server,Path values,Path inputFile)throws Exception{
        String input;try(var stream=NativeRiverNetworkProbe.class.getResourceAsStream("/terraforged-qa-river-networks.json")){if(stream==null)throw new IllegalStateException("Explicit network inputs missing");input=new String(stream.readAllBytes(),java.nio.charset.StandardCharsets.UTF_8);}
        Files.writeString(inputFile,input,java.nio.charset.StandardCharsets.UTF_8);var root=JsonParser.parseString(input).getAsJsonObject();var lookup=server.registryAccess().lookupOrThrow(RTFRegistries.NOISE);var binding=root.getAsJsonObject("noiseRegistry");
        for(var key:List.of(PresetTerrainTypeNoise.GROUND,PresetClimateNoise.BIOME_EDGE_SHAPE)){var actual=Noise.DIRECT_CODEC.encodeStart(JsonOps.INSTANCE,lookup.getOrThrow(key).value()).getOrThrow();if(!JsonParser.parseString(actual.toString()).equals(JsonParser.parseString(binding.get(key.identifier().toString()).getAsString())))throw new IllegalStateException("Actual noise binding differs");}
        StringBuilder out=new StringBuilder();int baseline=CacheManager.activeCacheCount();var cases=root.getAsJsonObject("presets");
        for(String name:new TreeSet<>(cases.keySet()))for(var seedElement:root.getAsJsonArray("worldSeeds")){
            long worldSeed=seedElement.getAsLong();int seed=(int)worldSeed;Preset preset=Preset.DIRECT_CODEC.parse(JsonOps.INSTANCE,cases.get(name)).getOrThrow();String contextKey=name+"/"+worldSeed;
            try(var context=GeneratorContext.makeUncached(preset,lookup,seed,1,1,1)){
                var heightmap=context.generator.getHeightmap();var continent=(SimpleContinent)heightmap.continent();int centerIndex=0;
                for(var query:root.getAsJsonArray("centerQueries")){var p=query.getAsJsonArray();long center=continent.getNearestCenter(p.get(0).getAsFloat(),p.get(1).getAsFloat());int cx=PosUtil.unpackLeft(center),cz=PosUtil.unpackRight(center);String key=contextKey+"/"+centerIndex++;
                    var map=continent.getRivermap(cx,cz);var networks=(Network[])field(map,"networks");out.append("map-info/").append(key).append('\t').append(cx).append(',').append(cz).append(',').append(networks.length).append('\n');
                    StringBuilder expected=new StringBuilder();for(int n=0;n<networks.length;n++)tree(key+"/"+n,networks[n],root,expected);out.append(expected);
                    // Exercise original stage methods with a caller-owned RNG and
                    // compare against the actual cached generateRivers result.
                    var generator=new SimpleRiverGenerator(continent,context);Random random=new Random(center+(seed+preset.rivers().seedOffset));var warp=GenWarp.make((int)center,preset.world().continent.continentScale);var builders=generator.generateRoots(cx,cz,random,warp);Collections.shuffle(builders,random);for(var b:builders)generator.generateForks(b,River.MAIN_SPACING,RiverConfig.builder(context.levels).bankHeight(preset.rivers().branchRivers.minBankHeight,preset.rivers().branchRivers.maxBankHeight).bankWidth(preset.rivers().branchRivers.bankWidth).bedWidth(preset.rivers().branchRivers.bedWidth).bedDepth(preset.rivers().branchRivers.bedDepth).fade(preset.rivers().branchRivers.fade).length(4500).order(1).build(),random,warp,builders,0);for(var b:builders)generator.generateWetlands(b,random);
                    StringBuilder replay=new StringBuilder();for(int n=0;n<builders.size();n++)tree(key+"/"+n,builders.get(n).build(),root,replay);if(!expected.toString().equals(replay.toString()))throw new IllegalStateException("Original staged construction differs from cached map");out.append("rng/").append(key).append('\t').append(random.nextInt()).append('\n');
                    var grid=root.getAsJsonArray("grid");for(int x=0;x<grid.size();x++)for(int z=0;z<grid.size();z++)sample("grid/"+key+"/"+x+"/"+z,map,cx+grid.get(x).getAsFloat(),cz+grid.get(z).getAsFloat(),root,out);
                    for(int n=0;n<networks.length;n++)lines(key+"/"+n,networks[n],map,root,out);
                    if(continent.getRivermap(cx,cz)!=map)throw new IllegalStateException("Original cache did not reuse map");
                }
                var coords=root.getAsJsonArray("fullCoordinates");for(int x=0;x<coords.size();x++)for(int z=0;z<coords.size();z++)for(int climate=0;climate<2;climate++){Cell c=new Cell();heightmap.apply(c,coords.get(x).getAsFloat(),coords.get(z).getAsFloat(),climate==1);snapshot("full/"+contextKey+"/"+x+"/"+z+"/"+climate,c,out);}
            }
        }
        if(CacheManager.activeCacheCount()!=baseline)throw new IllegalStateException("Network oracle leaked context caches");Files.writeString(values,out.toString(),java.nio.charset.StandardCharsets.UTF_8);
    }
}
