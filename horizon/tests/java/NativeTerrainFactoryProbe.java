package terraforged.qa;
import java.nio.file.*;
import java.lang.reflect.*;
import java.util.*;
import com.google.gson.*;
import com.mojang.serialization.JsonOps;
import raccoonman.reterraforged.data.worldgen.preset.settings.TerrainSettings;
import raccoonman.reterraforged.world.worldgen.cell.*;
import raccoonman.reterraforged.world.worldgen.cell.biome.type.BiomeType;
import raccoonman.reterraforged.world.worldgen.cell.terrain.*;
import raccoonman.reterraforged.world.worldgen.cell.terrain.populator.*;
import raccoonman.reterraforged.world.worldgen.cell.heightmap.Levels;
import raccoonman.reterraforged.world.worldgen.noise.module.*;
import raccoonman.reterraforged.world.worldgen.util.Seed;
/** All original terrain factory branches and seed consumption, independently observed. */
public final class NativeTerrainFactoryProbe {
    private static final Field[] FIELDS=Arrays.stream(Cell.class.getFields()).filter(f->!Modifier.isStatic(f.getModifiers())).sorted(Comparator.comparing(Field::getName)).toArray(Field[]::new);
    private static String bits(float f) {return Float.isNaN(f)?"NaN":String.format("%08x",Float.floatToRawIntBits(f));}
    private static Cell cell() throws Exception {Cell c=new Cell();for(Field f:FIELDS) {
        if(f.getType()==float.class) f.setFloat(c,-3.25F);else if(f.getType()==int.class) f.setInt(c,-123456);else if(f.getType()==boolean.class) f.setBoolean(c,true);
    }return c;}
    private static void snapshot(String key,Cell c,StringBuilder out) throws Exception {
        out.append(key).append('\t');for(int i=0;i<FIELDS.length;i++) {if(i>0) out.append(',');Object v=FIELDS[i].get(c);out.append(v instanceof Float f?bits(f):v instanceof Terrain t?t.getId():v instanceof BiomeType b?b.ordinal():v);}out.append('\n');
    }
    public static void capture(Path values,Path inputFile) throws Exception {
        String input;try(var stream=NativeTerrainFactoryProbe.class.getResourceAsStream("/terraforged-qa-terrain-factories.json")) {
            if(stream==null) throw new IllegalStateException("Explicit factory inputs missing");input=new String(stream.readAllBytes(),java.nio.charset.StandardCharsets.UTF_8);
        }
        Files.writeString(inputFile,input,java.nio.charset.StandardCharsets.UTF_8);var root=JsonParser.parseString(input).getAsJsonObject();
        var coords=root.getAsJsonArray("coordinates");var cases=root.getAsJsonObject("cases");StringBuilder out=new StringBuilder();
        for(String name:new TreeSet<>(cases.keySet())) for(var seedElement:root.getAsJsonArray("seeds")) for(int factory=0;factory<20;factory++) {
            var config=cases.getAsJsonObject(name);int rootSeed=seedElement.getAsInt();Seed seed=new Seed(rootSeed);
            var s=TerrainSettings.Terrain.CODEC.parse(JsonOps.INSTANCE,config.get("entry")).getOrThrow();
            var border=TerrainSettings.Terrain.CODEC.parse(JsonOps.INSTANCE,config.get("borderScaling")).getOrThrow();
            float vertical=config.get("globalVerticalScale").getAsFloat();Levels levels=new Levels(config.get("height").getAsInt(),config.get("sea").getAsInt());Noise ground=Noises.constant(levels.ground);
            CellPopulator populator=switch(factory) {
                case 0->Populators.makeSteppe(seed,ground,s);case 1->Populators.makePlains(seed,ground,s,vertical);case 2->Populators.makePlateau(seed,ground,s,vertical);
                case 3->Populators.makeHills1(seed,ground,s,vertical);case 4->Populators.makeHills2(seed,ground,s,vertical);case 5->Populators.makeDales(seed,ground,s);
                case 6->Populators.makeBadlands(seed,ground,s);case 7->Populators.makeTorridonian(seed,ground,s);
                case 8->Populators.makeMountains(seed,ground,s,vertical,false);case 9->Populators.makeMountainChain(seed,ground,s,vertical,false);
                case 10->Populators.makeMountains2(seed,ground,s,vertical,false);case 11->Populators.makeMountains3(seed,ground,s,vertical,false);
                case 12->Populators.makeMountains(seed,ground,s,vertical,true);case 13->Populators.makeMountainChain(seed,ground,s,vertical,true);
                case 14->Populators.makeMountains2(seed,ground,s,vertical,true);case 15->Populators.makeMountains3(seed,ground,s,vertical,true);
                case 16->Populators.makeBorder(seed,ground,s,border,vertical);case 17->Populators.makeDeepOcean(rootSeed,levels.water);
                case 18->Populators.makeShallowOcean(levels);case 19->Populators.makeCoast(levels);default->throw new IllegalStateException();
            };
            String prefix=name+"/"+rootSeed+"/"+factory;out.append("cursor/").append(prefix).append('\t').append(seed.get()).append('\n');
            if(populator instanceof TerrainPopulator p) {
                out.append("meta/").append(prefix).append('\t').append(p.type().getId()).append(',').append(bits(p.baseScale())).append(',').append(bits(p.heightScale())).append(',').append(bits(p.weight()));
                for(Noise noise:List.of(p.base(),p.height(),p.erosion(),p.weirdness())) out.append(',').append(noise.getClass().getSimpleName()).append(',').append(bits(noise.minValue())).append(',').append(bits(noise.maxValue()));out.append('\n');
            }
            for(int i=0;i<coords.size();i++) for(int j=0;j<coords.size();j++) {
                float x=coords.get(i).getAsFloat(),z=coords.get(j).getAsFloat();Cell value=cell();populator.apply(value,x,z);snapshot("cell/"+prefix+"/"+i+"/"+j,value,out);
                if(populator instanceof TerrainPopulator p) out.append("noise/").append(prefix).append('/').append(i).append('/').append(j).append('\t').append(bits(p.base().compute(x,z,0))).append(',').append(bits(p.height().compute(x,z,0))).append(',').append(bits(p.erosion().compute(x,z,0))).append(',').append(bits(p.weirdness().compute(x,z,0))).append('\n');
            }
        }
        for(var v:root.getAsJsonArray("roundInputs")) {String key=v.getAsString();out.append("round/").append(key).append('\t').append(Math.round(Float.intBitsToFloat(Integer.parseUnsignedInt(key,16)))).append('\n');}
        Files.writeString(values,out.toString(),java.nio.charset.StandardCharsets.UTF_8);
    }
}
