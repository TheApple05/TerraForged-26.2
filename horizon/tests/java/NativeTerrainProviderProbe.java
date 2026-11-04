package terraforged.qa;
import java.nio.file.*;
import java.lang.reflect.*;
import java.util.*;
import com.google.gson.*;
import com.mojang.serialization.JsonOps;
import net.minecraft.server.MinecraftServer;
import raccoonman.reterraforged.data.worldgen.preset.PresetTerrainTypeNoise;
import raccoonman.reterraforged.data.worldgen.preset.settings.Preset;
import raccoonman.reterraforged.registries.RTFRegistries;
import raccoonman.reterraforged.world.worldgen.cell.*;
import raccoonman.reterraforged.world.worldgen.cell.biome.type.BiomeType;
import raccoonman.reterraforged.world.worldgen.cell.terrain.*;
import raccoonman.reterraforged.world.worldgen.cell.terrain.populator.*;
import raccoonman.reterraforged.world.worldgen.cell.terrain.provider.TerrainProvider;
import raccoonman.reterraforged.world.worldgen.cell.terrain.region.RegionSelector;
import raccoonman.reterraforged.world.worldgen.cell.heightmap.*;
import raccoonman.reterraforged.world.worldgen.noise.module.*;
import raccoonman.reterraforged.world.worldgen.util.Seed;
/** Genuine provider ordering/registry use and volcano observations. */
public final class NativeTerrainProviderProbe {
    private static final Field[] FIELDS=Arrays.stream(Cell.class.getFields()).filter(f->!Modifier.isStatic(f.getModifiers())).sorted(Comparator.comparing(Field::getName)).toArray(Field[]::new);
    private static String bits(float f) {return Float.isNaN(f)?"NaN":String.format("%08x",Float.floatToRawIntBits(f));}
    private static Cell cell() throws Exception {Cell c=new Cell();for(Field f:FIELDS) {
        if(f.getType()==float.class) f.setFloat(c,-3.25F);else if(f.getType()==int.class) f.setInt(c,-123456);else if(f.getType()==boolean.class) f.setBoolean(c,true);
    }return c;}
    private static void snapshot(String key,Cell c,StringBuilder out) throws Exception {
        out.append(key).append('\t');for(int i=0;i<FIELDS.length;i++) {if(i>0) out.append(',');Object v=FIELDS[i].get(c);out.append(v instanceof Float f?bits(f):v instanceof Terrain t?t.getId():v instanceof BiomeType b?b.ordinal():v);}out.append('\n');
    }
    public static void capture(MinecraftServer server,Path values,Path inputFile) throws Exception {
        String input;try(var stream=NativeTerrainProviderProbe.class.getResourceAsStream("/terraforged-qa-terrain-provider.json")) {
            if(stream==null) throw new IllegalStateException("Explicit provider inputs missing");input=new String(stream.readAllBytes(),java.nio.charset.StandardCharsets.UTF_8);
        }
        Files.writeString(inputFile,input,java.nio.charset.StandardCharsets.UTF_8);var root=JsonParser.parseString(input).getAsJsonObject();
        var lookup=server.registryAccess().lookupOrThrow(RTFRegistries.NOISE);var ground=lookup.getOrThrow(PresetTerrainTypeNoise.GROUND).value();
        String actualGround=Noise.DIRECT_CODEC.encodeStart(JsonOps.INSTANCE,ground).getOrThrow().toString();
        if(!actualGround.equals(root.getAsJsonObject("noiseRegistry").get("reterraforged:terrain/ground").getAsString())) throw new IllegalStateException("Actual registry ground differs from predefined binding");
        var cases=root.getAsJsonObject("presets");var coords=root.getAsJsonArray("coordinates");var ids=root.getAsJsonArray("identities");StringBuilder out=new StringBuilder();
        Set<Integer> observedVolcanoTypes=new TreeSet<>();
        for(String name:new TreeSet<>(cases.keySet())) for(var seedElement:root.getAsJsonArray("seeds")) {
            Preset preset=Preset.DIRECT_CODEC.parse(JsonOps.INSTANCE,cases.get(name)).getOrThrow();int seedValue=seedElement.getAsInt();Seed seed=new Seed(seedValue);
            var props=preset.world().properties;var levels=new Levels(props.terrainScaler(),props.seaLevel);var settings=preset.terrain();
            int warpSeed=seedValue+8934;var region=new RegionConfig(seedValue+789124,settings.general.terrainRegionSize,Noises.simplex(warpSeed,400,1),Noises.simplex(warpSeed+1,400,1),200);
            var nodes=TerrainProvider.generateTerrain(seed,settings,region,levels,lookup);String prefix=name+"/"+seedValue;
            out.append("provider/").append(prefix).append('\t').append(seed.get()).append(',').append(nodes.size()).append('\n');
            for(int n=0;n<nodes.size();n++) {
                var p=nodes.get(n);out.append("node/").append(prefix).append('/').append(n).append('\t').append(p.getClass().getSimpleName()).append(',').append(bits(((WeightedPopulator)p).weight()));
                if(p instanceof TerrainPopulator t) {
                    out.append(',').append(t.type().getId()).append(',').append(t.type().getName()).append(',').append(bits(t.baseScale())).append(',').append(bits(t.heightScale()));
                    for(Noise noise:List.of(t.base(),t.height(),t.erosion(),t.weirdness())) out.append(',').append(noise.getClass().getSimpleName()).append(',').append(bits(noise.minValue())).append(',').append(bits(noise.maxValue()));
                }out.append('\n');
                for(int i=0;i<coords.size();i++) for(int j=0;j<coords.size();j++) {Cell c=cell();p.apply(c,coords.get(i).getAsFloat(),coords.get(j).getAsFloat());snapshot("node-cell/"+prefix+"/"+n+"/"+i+"/"+j,c,out);}
            }
            var selector=new RegionSelector(nodes);
            for(int id=0;id<ids.size();id++) for(int i=0;i<coords.size();i++) for(int j=0;j<coords.size();j++) {Cell c=cell();c.terrainRegionId=ids.get(id).getAsFloat();selector.apply(c,coords.get(i).getAsFloat(),coords.get(j).getAsFloat());snapshot("selector/"+prefix+"/"+id+"/"+i+"/"+j,c,out);}
            Seed volcanoSeed=new Seed(seedValue);var volcano=new VolcanoPopulator(volcanoSeed,region,levels,settings.volcano.weight);
            out.append("volcano-cursor/").append(prefix).append('\t').append(volcanoSeed.get()).append('\n');
            var grid=root.getAsJsonObject("volcanoGrid");int min=grid.get("min").getAsInt(),max=grid.get("max").getAsInt(),step=grid.get("step").getAsInt();
            for(int z=min;z<=max;z+=step) for(int x=min;x<=max;x+=step) {Cell c=cell();volcano.apply(c,x,z);observedVolcanoTypes.add(c.terrain.getId());snapshot("volcano/"+prefix+"/"+x+"/"+z,c,out);}
            var hs=root.getAsJsonArray("modifyHeights");var ms=root.getAsJsonArray("modifyMasks");
            for(int h=0;h<hs.size();h++) for(int m=0;m<ms.size();m++) for(int initial:new int[]{0,16,17}) {Cell c=cell();c.terrain=TerrainType.get(initial);c.height=hs.get(h).getAsFloat();c.riverMask=ms.get(m).getAsFloat();VolcanoPopulator.modifyVolcanoType(c,levels);snapshot("modify/"+prefix+"/"+h+"/"+m+"/"+initial,c,out);}
        }
        if(!observedVolcanoTypes.containsAll(Set.of(0,16,17))) throw new IllegalStateException("Volcano branches not all observed: "+observedVolcanoTypes);
        Files.writeString(values,out.toString(),java.nio.charset.StandardCharsets.UTF_8);
    }
}
