package raccoonman.reterraforged.world.worldgen.noise.module;
import java.nio.file.*;
import java.lang.reflect.*;
import java.util.*;
import com.google.gson.*;
import com.mojang.serialization.JsonOps;
import net.minecraft.util.StringRepresentable;
import raccoonman.reterraforged.data.worldgen.preset.settings.*;
import raccoonman.reterraforged.world.worldgen.cell.heightmap.ControlPoints;

/** Actual preset codecs, normalized helpers and public source factories. */
public final class NativePresetProbe {
    private record Named(String name,Noise noise) {}
    private static String bits(float f) {return Float.isNaN(f)?"NaN":String.format("%08x",Float.floatToRawIntBits(f));}
    private static Noise s() {return Noises.simplex(12345,80,3);}
    private static void flatten(String key,Object value,StringBuilder out) throws Exception {
        if(value instanceof Float f) {out.append(key).append('\t').append(bits(f)).append('\n');return;}
        if(value instanceof Integer || value instanceof Boolean || value instanceof String) {out.append(key).append('\t').append(value).append('\n');return;}
        if(value instanceof StringRepresentable e) {out.append(key).append('\t').append(e.getSerializedName()).append('\n');return;}
        Field[] fields=value.getClass().getFields();Arrays.sort(fields,Comparator.comparing(Field::getName));
        for(Field f:fields) if(!Modifier.isStatic(f.getModifiers()) && !f.getName().equals("legacyMountainScaling")) flatten(key+"/"+f.getName(),f.get(value),out);
    }
    public static void capture(Path values,Path inputFile) throws Exception {
        String input;
        try(var stream=NativePresetProbe.class.getResourceAsStream("/terraforged-qa-preset-model.json")) {
            if(stream==null) throw new IllegalStateException("Explicit preset input resource missing");
            input=new String(stream.readAllBytes(),java.nio.charset.StandardCharsets.UTF_8);
        }
        Files.writeString(inputFile,input,java.nio.charset.StandardCharsets.UTF_8);
        var cases=JsonParser.parseString(input).getAsJsonObject();
        StringBuilder out=new StringBuilder();List<Named> graphs=new ArrayList<>();
        for(String name:new TreeSet<>(cases.keySet())) {
            Preset p=Preset.DIRECT_CODEC.parse(JsonOps.INSTANCE,cases.get(name)).getOrThrow();
            flatten("preset/"+name+"/world",p.world(),out);flatten("preset/"+name+"/terrain",p.terrain(),out);flatten("preset/"+name+"/climate",p.climate(),out);flatten("preset/"+name+"/rivers",p.rivers(),out);flatten("preset/"+name+"/filters",p.filters(),out);
            ControlPoints c=ControlPoints.make(p.world().controlPoints);
            out.append("controls/").append(name).append('\t').append(bits(c.mushroomFieldsInland())).append(',').append(bits(c.mushroomFieldsCoast())).append(',').append(bits(c.deepOcean())).append(',').append(bits(c.shallowOcean())).append(',').append(bits(c.beach())).append(',').append(bits(c.coast())).append(',').append(bits(c.coastMarker())).append(',').append(bits(c.inland())).append('\n');
            out.append("controls-valid/").append(name).append('\t').append(ControlPoints.validate(p.world().controlPoints)).append('\n');
            out.append("terrain-scaler/").append(name).append('\t').append(p.world().properties.terrainScaler()).append('\n');
            for(var e:List.of(java.util.Map.entry("temperature",p.climate().temperature),java.util.Map.entry("moisture",p.climate().moisture))) {
                var r=e.getValue();out.append("range/").append(name).append('/').append(e.getKey()).append('\t').append(bits(r.getMin())).append(',').append(bits(r.getMax())).append(',').append(bits(r.getBias())).append('\n');
                graphs.add(new Named("range-"+name+"-"+e.getKey(),r.apply(s())));
            }
        }
        graphs.add(new Named("perlin-default",Noises.perlin(12345,80,3)));
        graphs.add(new Named("perlin2-default",Noises.perlin2(12345,80,3)));
        graphs.add(new Named("simplex-default",Noises.simplex(12345,80,3)));
        graphs.add(new Named("simplex2-default",Noises.simplex2(12345,80,3)));
        graphs.add(new Named("perlin-ridge-default",Noises.perlinRidge(12345,80,3)));
        graphs.add(new Named("simplex-ridge-default",Noises.simplexRidge(12345,80,3)));
        graphs.add(new Named("billow-default",Noises.billow(12345,80,3)));
        graphs.add(new Named("cubic-default",Noises.cubic(12345,80,3)));
        graphs.add(new Named("perlin-full",Noises.perlin(Integer.MIN_VALUE,24,4,1.75F,0.625F)));
        graphs.add(new Named("perlin2-full",Noises.perlin2(Integer.MIN_VALUE,24,4,1.75F,0.625F)));
        graphs.add(new Named("simplex-full",Noises.simplex(Integer.MIN_VALUE,24,4,1.75F,0.625F)));
        graphs.add(new Named("simplex2-full",Noises.simplex2(Integer.MIN_VALUE,24,4,1.75F,0.625F)));
        graphs.add(new Named("perlin-ridge-full",Noises.perlinRidge(Integer.MIN_VALUE,24,4,1.75F,0.625F)));
        graphs.add(new Named("simplex-ridge-full",Noises.simplexRidge(Integer.MIN_VALUE,24,4,1.75F,0.625F)));
        graphs.add(new Named("billow-full",Noises.billow(Integer.MIN_VALUE,24,4,1.75F,0.625F)));
        graphs.add(new Named("cubic-full",Noises.cubic(Integer.MIN_VALUE,24,4,1.75F,0.625F)));
        graphs.add(new Named("worley-default",Noises.worley(12345,80)));
        graphs.add(new Named("edge-default",Noises.worleyEdge(12345,80)));
        graphs.add(new Named("white-default",Noises.white(12345,80)));
        graphs.add(new Named("sin-default",Noises.sin(12345,0.0125F,s())));
        graphs.add(new Named("map-constant-shortcut",Noises.map(Noises.constant(0.5F),0.5F,0.5F)));
        graphs.add(new Named("map-unit-shortcut",Noises.map(s(),0.0F,1.0F)));
        graphs.add(new Named("map-reverse",Noises.map(s(),1.0F,-1.0F)));
        graphs.add(new Named("warp-perlin",Noises.warpPerlin(s(),Integer.MAX_VALUE,80,3,100.0F)));
        graphs.add(new Named("warp-white",Noises.warpWhite(s(),Integer.MAX_VALUE,8,10.0F)));
        graphs.add(new Named("frequency",Noises.frequency(s(),-0.25F,0.75F)));
        for(ClimateSettings.BiomeNoise.EdgeType type:ClimateSettings.BiomeNoise.EdgeType.values())
            graphs.add(new Named("edge-type-"+type.getSerializedName(),new ClimateSettings.BiomeNoise(type,24,3,0.5F,2.65F,14).build(12345)));
        int[] seeds={0,1,-1,42,Integer.MIN_VALUE,Integer.MAX_VALUE};
        float[] coords={-30_000_000f,-17f,-16.25f,-16f,-1.5f,-1f,-.25f,-0f,0f,.25f,1f,15f,15.75f,16f,16.25f,17f,30_000_000f};
        for(int n=0;n<graphs.size();n++) {
            Named g=graphs.get(n);Noise noise=g.noise();
            out.append("graph/").append(n).append('\t').append(g.name()).append(',').append(noise.getClass().getSimpleName()).append(',').append(bits(noise.minValue())).append(',').append(bits(noise.maxValue())).append('\n');
            for(int seed:seeds) for(int i=0;i<coords.length;i++) for(int j=0;j<coords.length;j++)
                out.append("sample/").append(n).append('/').append(seed).append('/').append(i).append('/').append(j).append('\t').append(bits(noise.compute(coords[i],coords[j],seed))).append('\n');
        }
        Files.writeString(values,out.toString(),java.nio.charset.StandardCharsets.UTF_8);
    }
}
