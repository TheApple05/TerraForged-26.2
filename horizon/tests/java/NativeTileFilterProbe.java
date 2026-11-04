package terraforged.qa;
import java.nio.file.*;
import java.lang.reflect.*;
import java.util.*;
import java.util.concurrent.TimeUnit;
import com.google.gson.*;
import com.mojang.serialization.JsonOps;
import net.minecraft.server.MinecraftServer;
import raccoonman.reterraforged.concurrent.cache.CacheManager;
import raccoonman.reterraforged.data.worldgen.preset.settings.Preset;
import raccoonman.reterraforged.data.worldgen.preset.PresetTerrainTypeNoise;
import raccoonman.reterraforged.data.worldgen.preset.PresetClimateNoise;
import raccoonman.reterraforged.world.worldgen.noise.module.Noise;
import raccoonman.reterraforged.registries.RTFRegistries;
import raccoonman.reterraforged.world.worldgen.GeneratorContext;
import raccoonman.reterraforged.world.worldgen.cell.Cell;
import raccoonman.reterraforged.world.worldgen.cell.biome.type.BiomeType;
import raccoonman.reterraforged.world.worldgen.cell.terrain.*;
import raccoonman.reterraforged.world.worldgen.cell.heightmap.*;
import raccoonman.reterraforged.world.worldgen.densityfunction.tile.Size;
import raccoonman.reterraforged.world.worldgen.densityfunction.tile.filter.*;
/** Original filters on controlled backing arrays and actual zoomed TileGenerator. */
public final class NativeTileFilterProbe {
    private static final Field[] FIELDS=Arrays.stream(Cell.class.getFields()).filter(f->!java.lang.reflect.Modifier.isStatic(f.getModifiers())).sorted(Comparator.comparing(Field::getName)).toArray(Field[]::new);
    private static String bits(float f){return Float.isNaN(f)?"NaN":String.format("%08x",Float.floatToRawIntBits(f));}
    private static Object field(Object object,String key)throws Exception{Field f=object.getClass().getDeclaredField(key);f.setAccessible(true);return f.get(object);}
    private static void snapshot(String key,Cell c,StringBuilder out)throws Exception{out.append(key).append('\t');for(int i=0;i<FIELDS.length;i++){if(i>0)out.append(',');Object v=FIELDS[i].get(c);out.append(v instanceof Float f?bits(f):v instanceof Terrain t?t.getId():v instanceof BiomeType b?b.ordinal():v);}out.append('\n');}
    private static final class ArrayMap implements Filterable {
        final Size size;final Cell[] cells;final int bx,bz;
        ArrayMap(JsonObject v,JsonObject root)throws Exception {int n=v.get("size").getAsInt(),pattern=v.get("pattern").getAsInt();size=Size.make(n,0);bx=v.getAsJsonArray("origin").get(0).getAsInt();bz=v.getAsJsonArray("origin").get(1).getAsInt();cells=new Cell[n*n];
            for(int z=0;z<n;z++)for(int x=0;x<n;x++){int index=z*n+x;Cell c=new Cell();for(Field f:FIELDS){if(f.getType()==float.class)f.setFloat(c,-3.25F);else if(f.getType()==int.class)f.setInt(c,-123456);}c.height=switch(pattern){case 0->0.2F+x*0.003F+z*0.001F;case 1->((x*31+z*17)%257)/256.0F;case 2->root.getAsJsonArray("heights").get((x+z*3)%8).getAsFloat();default->index%47==0?Float.NaN:0.25F;};c.terrain=TerrainType.get((x*3+z*7)%18);c.continentEdge=((x+z)%5)*0.1F;c.terrainRegionEdge=((x*7+z*11)%17)/16.0F;c.riverMask=root.getAsJsonArray("masks").get((x+z)%7).getAsFloat();c.erosionMask=index%11==0;cells[index]=c;}
        }
        public int getBlockX(){return bx;}public int getBlockZ(){return bz;}public Size getBlockSize(){return size;}public Cell[] getBacking(){return cells;}public Cell getCellRaw(int x,int z){int i=size.indexOf(x,z);return i<0||i>=cells.length?Cell.empty():cells[i];}
    }
    public static void capture(MinecraftServer server,Path values,Path inputFile)throws Exception {
        String input;try(var stream=NativeTileFilterProbe.class.getResourceAsStream("/terraforged-qa-tile-filters.json")){if(stream==null)throw new IllegalStateException("Explicit tile-filter inputs missing");input=new String(stream.readAllBytes(),java.nio.charset.StandardCharsets.UTF_8);}
        Files.writeString(inputFile,input,java.nio.charset.StandardCharsets.UTF_8);var root=JsonParser.parseString(input).getAsJsonObject();var presets=root.getAsJsonObject("presets");StringBuilder out=new StringBuilder();Set<Integer> brushes=new HashSet<>();int caseIndex=0;
        for(var element:root.getAsJsonArray("cases")){var v=element.getAsJsonObject();int n=v.get("size").getAsInt(),seed=v.get("seed").getAsInt();var lv=v.getAsJsonArray("levels");Levels levels=new Levels(lv.get(0).getAsInt(),lv.get(1).getAsInt());Preset preset=Preset.DIRECT_CODEC.parse(JsonOps.INSTANCE,presets.get(v.get("preset").getAsString())).getOrThrow();var settings=preset.filters();var modifier=raccoonman.reterraforged.world.worldgen.densityfunction.tile.filter.Modifier.range(levels.ground,levels.ground(15));var erosion=new Erosion(seed+12768,n,settings.erosion,modifier);
            if(brushes.add(n)){var indices=(int[][])field(erosion,"erosionBrushIndices");var weights=(float[][])field(erosion,"erosionBrushWeights");for(int i=0;i<indices.length;i++){out.append("brush/").append(n).append('/').append(i).append('\t');for(int j=0;j<indices[i].length;j++){if(j>0)out.append(',');out.append(indices[i][j]).append(',').append(bits(weights[i][j]));}out.append('\n');}}
            var initial=new ArrayMap(v,root);int[] xs={-8,-1,0,n-1,n,n+8},zs={-1,0,1,n-1,n,n+1};for(int x:xs)for(int z:zs){Cell c=initial.getCellRaw(x,z);out.append("raw/").append(caseIndex).append('/').append(x).append('/').append(z).append('\t').append(c.isAbsent()?"absent":initial.size.indexOf(x,z)).append('\n');}
            for(int i=0;i<Math.min(64,initial.cells.length);i++){var mod=modifier;for(int inv=0;inv<3;inv++){Cell c=initial.cells[i];out.append("modifier/").append(caseIndex).append('/').append(i).append('/').append(inv).append('\t').append(bits(mod.getValueModifier(c.height))).append(',').append(bits(mod.modify(c,-0.125F))).append('\n');mod=mod.invert();}}
            for(var stage:root.getAsJsonArray("stages")){String name=stage.getAsString();var map=new ArrayMap(v,root);var smoothing=Smoothing.make(settings.smoothing,levels);var steepness=Steepness.make(1,10.0F,levels);var beach=new BeachDetect(levels,ControlPoints.make(preset.world().controlPoints));var correction=new NoiseCorrection(levels);
                if(name.equals("erosion")||name.equals("optional"))erosion.apply(map,0,0,settings.erosion.dropletsPerChunk);
                if(name.equals("smoothing")||name.equals("optional"))smoothing.apply(map,0,0,settings.smoothing.iterations);
                if(name.equals("steepness")||name.equals("optional")||name.equals("required"))steepness.apply(map,0,0,1);
                if(name.equals("beach")||name.equals("optional")||name.equals("required"))beach.apply(map,0,0,1);
                if(name.equals("correction")||name.equals("optional"))correction.apply(map,0,0,1);
                for(int i=0;i<map.cells.length;i++)snapshot("stage/"+caseIndex+"/"+name+"/"+i,map.cells[i],out);
            }caseIndex++;
        }
        var lookup=server.registryAccess().lookupOrThrow(RTFRegistries.NOISE);var binding=root.getAsJsonObject("noiseRegistry");
        for(var key:List.of(PresetTerrainTypeNoise.GROUND,PresetClimateNoise.BIOME_EDGE_SHAPE)){var actual=Noise.DIRECT_CODEC.encodeStart(JsonOps.INSTANCE,lookup.getOrThrow(key).value()).getOrThrow();if(!JsonParser.parseString(actual.toString()).equals(JsonParser.parseString(binding.get(key.identifier().toString()).getAsString())))throw new IllegalStateException("Actual zoom registry binding differs");}
        int baseline=CacheManager.activeCacheCount(),zoomIndex=0;
        for(var element:root.getAsJsonArray("zoomCases")){var v=element.getAsJsonObject();Preset preset=Preset.DIRECT_CODEC.parse(JsonOps.INSTANCE,presets.get(v.get("preset").getAsString())).getOrThrow();try(var context=GeneratorContext.makeUncached(preset,lookup,v.get("seed").getAsInt(),0,1,2)){var center=v.getAsJsonArray("center");for(int optional=0;optional<2;optional++)try(var tile=context.generator.generateZoomed(center.get(0).getAsFloat(),center.get(1).getAsFloat(),v.get("zoom").getAsFloat(),optional==1).get(90,TimeUnit.SECONDS)){for(int i=0;i<tile.getBacking().length;i++)snapshot("zoom/"+zoomIndex+"/"+optional+"/"+i,tile.getBacking()[i],out);}}zoomIndex++;}
        if(CacheManager.activeCacheCount()!=baseline)throw new IllegalStateException("Filter oracle leaked context caches");Files.writeString(values,out.toString(),java.nio.charset.StandardCharsets.UTF_8);
    }
}
