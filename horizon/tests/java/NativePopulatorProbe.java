package terraforged.qa;
import java.nio.file.*;
import java.lang.reflect.*;
import java.util.*;
import com.google.gson.*;
import raccoonman.reterraforged.world.worldgen.cell.Cell;
import raccoonman.reterraforged.world.worldgen.cell.CellPopulator;
import raccoonman.reterraforged.world.worldgen.cell.biome.type.BiomeType;
import raccoonman.reterraforged.world.worldgen.cell.terrain.*;
import raccoonman.reterraforged.world.worldgen.cell.terrain.populator.*;
import raccoonman.reterraforged.world.worldgen.cell.terrain.region.*;
import raccoonman.reterraforged.world.worldgen.cell.continent.*;
import raccoonman.reterraforged.world.worldgen.cell.heightmap.RegionConfig;
import raccoonman.reterraforged.world.worldgen.noise.module.*;
import raccoonman.reterraforged.world.worldgen.noise.function.Interpolation;
/** Original region, selector and blending observations; no native expected output. */
public final class NativePopulatorProbe {
    private static final Field[] FIELDS=Arrays.stream(Cell.class.getFields()).filter(f->!Modifier.isStatic(f.getModifiers())).sorted(Comparator.comparing(Field::getName)).toArray(Field[]::new);
    private static String bits(float f) {return Float.isNaN(f)?"NaN":String.format("%08x",Float.floatToRawIntBits(f));}
    private static Cell cell(boolean dirty) throws Exception {
        Cell c=new Cell();if(dirty) for(Field f:FIELDS) {
            if(f.getType()==float.class) f.setFloat(c,-3.25F);else if(f.getType()==int.class) f.setInt(c,-123456);else if(f.getType()==boolean.class) f.setBoolean(c,true);
        }return c;
    }
    private static void snapshot(String key,Cell c,StringBuilder out) throws Exception {
        out.append(key).append('\t');for(int i=0;i<FIELDS.length;i++) {if(i>0) out.append(',');Object v=FIELDS[i].get(c);out.append(v instanceof Float f?bits(f):v instanceof Terrain t?t.getId():v instanceof BiomeType b?b.ordinal():v);}out.append('\n');
    }
    public static void capture(Path values,Path inputFile) throws Exception {
        String input;try(var stream=NativePopulatorProbe.class.getResourceAsStream("/terraforged-qa-populators.json")) {
            if(stream==null) throw new IllegalStateException("Explicit populator inputs missing");input=new String(stream.readAllBytes(),java.nio.charset.StandardCharsets.UTF_8);
        }
        Files.writeString(inputFile,input,java.nio.charset.StandardCharsets.UTF_8);var root=JsonParser.parseString(input).getAsJsonObject();
        var coords=root.getAsJsonArray("coordinates");var edges=root.getAsJsonArray("edges");var ids=root.getAsJsonArray("identities");StringBuilder out=new StringBuilder();
        for(var seedElement:root.getAsJsonArray("seeds")) for(var scaleElement:root.getAsJsonArray("regionScales")) {
            int seed=seedElement.getAsInt(),scale=scaleElement.getAsInt();int warpSeed=seed+8934;
            RegionModule region=new RegionModule(new RegionConfig(seed+789124,scale,Noises.simplex(warpSeed,400,1),Noises.simplex(warpSeed+1,400,1),200));
            for(int i=0;i<coords.size();i++) for(int j=0;j<coords.size();j++) for(int dirty=0;dirty<2;dirty++) {
                Cell c=cell(dirty==1);region.apply(c,coords.get(i).getAsFloat(),coords.get(j).getAsFloat());snapshot("region/"+seed+"/"+scale+"/"+i+"/"+j+"/"+dirty,c,out);
            }
        }
        var a=new TerrainPopulator(TerrainType.FLATS,Noises.constant(.25F),Noises.simplex(111,80,3),Noises.constant(.445F),Noises.constant(.34F),.75F,.1F,1.0F);
        var b=new TerrainPopulator(TerrainType.HILLS,Noises.constant(.5F),Noises.perlin(222,120,2),Noises.constant(.6F),Noises.constant(-.2F),1.25F,.3F,2.5F);
        var c=new TerrainPopulator(TerrainType.PLATEAU,Noises.zero(),Noises.constant(-2),Noises.constant(-.1F),Noises.constant(-.75F),1,1,0);
        var ocean=new OceanPopulator(TerrainType.DEEP_OCEAN,Noises.mul(Noises.simplex2(333,64,2),.3F));
        List<CellPopulator> graphs=new ArrayList<>(List.of(a,b,c,ocean,new RegionLerper(a,b)));
        for(var e:edges) graphs.add(new Blender(Noises.constant(e.getAsFloat()),a,b,.3F,.8F,.575F));
        graphs.add(new Blender(Noises.simplex(444,80,2),a,b,.3F,.8F,.575F));
        for(var interpolation:Interpolation.values()) graphs.add(new ContinentLerper2(a,b,.3F,.8F,interpolation));
        for(var interpolation:Interpolation.values()) graphs.add(new ContinentLerper3(ocean,a,b,.15F,.3F,.8F,interpolation));
        var selector=new RegionSelector(List.of(a,b,c,ocean));graphs.add(selector);
        var zero=new TerrainPopulator(TerrainType.BADLANDS,Noises.constant(.5F),Noises.constant(.125F),Noises.constant(.7F),Noises.constant(-.5F),0);
        graphs.add(new RegionSelector(List.of(c,zero)));graphs.add(new RegionSelector(List.of(ocean,new OceanPopulator(TerrainType.COAST,Noises.constant(.25F)))));
        graphs.add(new RegionLerper(ocean,new Blender(Noises.simplex(555,64,2),selector,b,.3F,.8F,.575F)));
        for(int g=0;g<graphs.size();g++) for(int i=0;i<coords.size();i++) for(int j=0;j<coords.size();j++) for(int e=0;e<edges.size();e++) {
            Cell value=cell(true);value.continentEdge=edges.get(e).getAsFloat();value.terrainRegionEdge=edges.get(e).getAsFloat();value.terrainRegionId=ids.get((i+j+e)%ids.size()).getAsFloat();
            graphs.get(g).apply(value,coords.get(i).getAsFloat(),coords.get(j).getAsFloat());snapshot("populator/"+g+"/"+i+"/"+j+"/"+e,value,out);
        }
        Files.writeString(values,out.toString(),java.nio.charset.StandardCharsets.UTF_8);
    }
}
