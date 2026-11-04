package raccoonman.reterraforged.world.worldgen.noise.module;
import java.util.*;
import java.nio.file.*;
import com.google.gson.*;
import com.mojang.serialization.JsonOps;
import net.minecraft.resources.RegistryOps;
import net.minecraft.server.MinecraftServer;
import raccoonman.reterraforged.registries.RTFRegistries;
/** Actual registry and codec observations; explicit nonfinite classifications. */
public final class NativeNoiseRegistryProbe {
    private static String bits(float f) {
        if(Float.isNaN(f)) return "NaN";
        if(f==Float.POSITIVE_INFINITY) return "+Infinity";
        if(f==Float.NEGATIVE_INFINITY) return "-Infinity";
        return String.format("%08x",Float.floatToRawIntBits(f));
    }
    public static void capture(MinecraftServer server,Path values,Path inputFile) throws Exception {
        var ops=RegistryOps.create(JsonOps.INSTANCE,server.registryAccess());
        var registry=server.registryAccess().lookupOrThrow(RTFRegistries.NOISE);
        TreeMap<String,String> inputs=new TreeMap<>();
        TreeMap<String,Noise> noises=new TreeMap<>();
        for(var entry:registry.entrySet()) {
            String id=entry.getKey().identifier().toString();
            inputs.put(id,Noise.DIRECT_CODEC.encodeStart(ops,entry.getValue()).getOrThrow().toString());
            noises.put(id,entry.getValue());
        }
        if(noises.size()!=26) throw new IllegalStateException("Expected actual 26-entry custom registry, got "+noises.size());
        inputs.put("qa:constant","{\"type\":\"reterraforged:constant\",\"value\":0.5}");
        inputs.put("qa:white","{\"type\":\"reterraforged:white\",\"frequency\":0.0125}");
        inputs.put("qa:perlin","{\"type\":\"reterraforged:perlin\",\"frequency\":0.0125,\"octaves\":3,\"lacunarity\":2,\"gain\":0.5,\"interpolation\":\"CURVE3\",\"seed\":12345}");
        inputs.put("qa:perlin2","{\"type\":\"reterraforged:perlin2\",\"frequency\":0.0125,\"octaves\":3,\"lacunarity\":2,\"gain\":0.5,\"interpolation\":\"CURVE3\",\"seed\":12345}");
        inputs.put("qa:simplex","{\"type\":\"reterraforged:simplex\",\"frequency\":0.0125,\"octaves\":3,\"lacunarity\":2,\"gain\":0.5,\"interpolation\":\"CURVE3\"}");
        inputs.put("qa:simplex2","{\"type\":\"reterraforged:simplex2\",\"frequency\":0.0125,\"octaves\":3,\"lacunarity\":2,\"gain\":0.5,\"interpolation\":\"CURVE3\"}");
        inputs.put("qa:perlin_ridge","{\"type\":\"reterraforged:perlin_ridge\",\"frequency\":0.0125,\"octaves\":3,\"lacunarity\":2,\"gain\":0.5,\"interpolation\":\"CURVE3\"}");
        inputs.put("qa:simplex_ridge","{\"type\":\"reterraforged:simplex_ridge\",\"frequency\":0.0125,\"octaves\":3,\"lacunarity\":2,\"gain\":0.5,\"interpolation\":\"CURVE3\"}");
        inputs.put("qa:billow","{\"type\":\"reterraforged:billow\",\"frequency\":0.0125,\"octaves\":3,\"lacunarity\":2,\"gain\":0.5,\"interpolation\":\"CURVE3\"}");
        inputs.put("qa:cubic","{\"type\":\"reterraforged:cubic\",\"frequency\":0.0125,\"octaves\":3,\"lacunarity\":2,\"gain\":0.5}");
        inputs.put("qa:sin","{\"type\":\"reterraforged:sin\",\"frequency\":0.0125,\"alpha\":{\"type\":\"reterraforged:simplex\",\"frequency\":0.0125,\"octaves\":3,\"lacunarity\":2,\"gain\":0.5,\"interpolation\":\"CURVE3\"}}");
        inputs.put("qa:worley","{\"type\":\"reterraforged:worley\",\"frequency\":0.0125,\"distance\":1,\"cell_function\":\"NOISE_LOOKUP\",\"distance_function\":\"NATURAL\",\"lookup\":{\"type\":\"reterraforged:add\",\"input1\":{\"type\":\"reterraforged:simplex\",\"frequency\":0.0125,\"octaves\":3,\"lacunarity\":2,\"gain\":0.5,\"interpolation\":\"CURVE3\"},\"input2\":0.125}}");
        inputs.put("qa:worley_edge","{\"type\":\"reterraforged:worley_edge\",\"frequency\":0.0125,\"distance\":1,\"edge_function\":\"DISTANCE_2_ADD\",\"distance_function\":\"EUCLIDEAN\"}");
        inputs.put("qa:line","{\"type\":\"reterraforged:line\",\"x1\":-4.125,\"z1\":2.25,\"x2\":6.375,\"z2\":-7.75,\"radiusSq\":20,\"feather\":0.5}");
        inputs.put("qa:shift","{\"type\":\"reterraforged:shift\",\"input\":{\"type\":\"reterraforged:add\",\"input1\":{\"type\":\"reterraforged:simplex\",\"frequency\":0.0125,\"octaves\":3,\"lacunarity\":2,\"gain\":0.5,\"interpolation\":\"CURVE3\"},\"input2\":0.125},\"shift\":12345}");
        inputs.put("qa:frequency","{\"type\":\"reterraforged:frequency\",\"input\":{\"type\":\"reterraforged:add\",\"input1\":{\"type\":\"reterraforged:simplex\",\"frequency\":0.0125,\"octaves\":3,\"lacunarity\":2,\"gain\":0.5,\"interpolation\":\"CURVE3\"},\"input2\":0.125},\"x_freq\":{\"type\":\"reterraforged:multiply\",\"input1\":{\"type\":\"reterraforged:simplex\",\"frequency\":0.0125,\"octaves\":3,\"lacunarity\":2,\"gain\":0.5,\"interpolation\":\"CURVE3\"},\"input2\":0.5},\"z_freq\":{\"type\":\"reterraforged:simplex\",\"frequency\":0.0125,\"octaves\":3,\"lacunarity\":2,\"gain\":0.5,\"interpolation\":\"CURVE3\"}}");
        inputs.put("qa:add","{\"type\":\"reterraforged:add\",\"input1\":{\"type\":\"reterraforged:add\",\"input1\":{\"type\":\"reterraforged:simplex\",\"frequency\":0.0125,\"octaves\":3,\"lacunarity\":2,\"gain\":0.5,\"interpolation\":\"CURVE3\"},\"input2\":0.125},\"input2\":{\"type\":\"reterraforged:multiply\",\"input1\":{\"type\":\"reterraforged:simplex\",\"frequency\":0.0125,\"octaves\":3,\"lacunarity\":2,\"gain\":0.5,\"interpolation\":\"CURVE3\"},\"input2\":0.5}}");
        inputs.put("qa:multiply","{\"type\":\"reterraforged:multiply\",\"input1\":{\"type\":\"reterraforged:add\",\"input1\":{\"type\":\"reterraforged:simplex\",\"frequency\":0.0125,\"octaves\":3,\"lacunarity\":2,\"gain\":0.5,\"interpolation\":\"CURVE3\"},\"input2\":0.125},\"input2\":{\"type\":\"reterraforged:multiply\",\"input1\":{\"type\":\"reterraforged:simplex\",\"frequency\":0.0125,\"octaves\":3,\"lacunarity\":2,\"gain\":0.5,\"interpolation\":\"CURVE3\"},\"input2\":0.5}}");
        inputs.put("qa:min","{\"type\":\"reterraforged:min\",\"input1\":{\"type\":\"reterraforged:add\",\"input1\":{\"type\":\"reterraforged:simplex\",\"frequency\":0.0125,\"octaves\":3,\"lacunarity\":2,\"gain\":0.5,\"interpolation\":\"CURVE3\"},\"input2\":0.125},\"input2\":{\"type\":\"reterraforged:multiply\",\"input1\":{\"type\":\"reterraforged:simplex\",\"frequency\":0.0125,\"octaves\":3,\"lacunarity\":2,\"gain\":0.5,\"interpolation\":\"CURVE3\"},\"input2\":0.5}}");
        inputs.put("qa:max","{\"type\":\"reterraforged:max\",\"input1\":{\"type\":\"reterraforged:add\",\"input1\":{\"type\":\"reterraforged:simplex\",\"frequency\":0.0125,\"octaves\":3,\"lacunarity\":2,\"gain\":0.5,\"interpolation\":\"CURVE3\"},\"input2\":0.125},\"input2\":{\"type\":\"reterraforged:multiply\",\"input1\":{\"type\":\"reterraforged:simplex\",\"frequency\":0.0125,\"octaves\":3,\"lacunarity\":2,\"gain\":0.5,\"interpolation\":\"CURVE3\"},\"input2\":0.5}}");
        inputs.put("qa:power","{\"type\":\"reterraforged:power\",\"input\":{\"type\":\"reterraforged:add\",\"input1\":{\"type\":\"reterraforged:simplex\",\"frequency\":0.0125,\"octaves\":3,\"lacunarity\":2,\"gain\":0.5,\"interpolation\":\"CURVE3\"},\"input2\":0.125},\"power\":1.5}");
        inputs.put("qa:power_curve","{\"type\":\"reterraforged:power_curve\",\"input\":{\"type\":\"reterraforged:add\",\"input1\":{\"type\":\"reterraforged:simplex\",\"frequency\":0.0125,\"octaves\":3,\"lacunarity\":2,\"gain\":0.5,\"interpolation\":\"CURVE3\"},\"input2\":0.125},\"power\":1.5}");
        inputs.put("qa:curve","{\"type\":\"reterraforged:curve\",\"input\":{\"type\":\"reterraforged:add\",\"input1\":{\"type\":\"reterraforged:simplex\",\"frequency\":0.0125,\"octaves\":3,\"lacunarity\":2,\"gain\":0.5,\"interpolation\":\"CURVE3\"},\"input2\":0.125},\"curve_function\":{\"type\":\"reterraforged:scurve\",\"lower\":2,\"upper\":-0.5}}");
        inputs.put("qa:gradient","{\"type\":\"reterraforged:gradient\",\"input\":{\"type\":\"reterraforged:add\",\"input1\":{\"type\":\"reterraforged:simplex\",\"frequency\":0.0125,\"octaves\":3,\"lacunarity\":2,\"gain\":0.5,\"interpolation\":\"CURVE3\"},\"input2\":0.125},\"lower\":0.25,\"upper\":0.75,\"strength\":{\"type\":\"reterraforged:multiply\",\"input1\":{\"type\":\"reterraforged:simplex\",\"frequency\":0.0125,\"octaves\":3,\"lacunarity\":2,\"gain\":0.5,\"interpolation\":\"CURVE3\"},\"input2\":0.5}}");
        inputs.put("qa:terrace","{\"type\":\"reterraforged:terrace\",\"input\":{\"type\":\"reterraforged:add\",\"input1\":{\"type\":\"reterraforged:simplex\",\"frequency\":0.0125,\"octaves\":3,\"lacunarity\":2,\"gain\":0.5,\"interpolation\":\"CURVE3\"},\"input2\":0.125},\"ramp\":{\"type\":\"reterraforged:multiply\",\"input1\":{\"type\":\"reterraforged:simplex\",\"frequency\":0.0125,\"octaves\":3,\"lacunarity\":2,\"gain\":0.5,\"interpolation\":\"CURVE3\"},\"input2\":0.5},\"cliff\":{\"type\":\"reterraforged:simplex\",\"frequency\":0.0125,\"octaves\":3,\"lacunarity\":2,\"gain\":0.5,\"interpolation\":\"CURVE3\"},\"ramp_height\":0.75,\"blend_range\":0.25,\"steps\":6}");
        inputs.put("qa:advanced_terrace","{\"type\":\"reterraforged:advanced_terrace\",\"source\":{\"type\":\"reterraforged:add\",\"input1\":{\"type\":\"reterraforged:simplex\",\"frequency\":0.0125,\"octaves\":3,\"lacunarity\":2,\"gain\":0.5,\"interpolation\":\"CURVE3\"},\"input2\":0.125},\"modulation\":{\"type\":\"reterraforged:multiply\",\"input1\":{\"type\":\"reterraforged:simplex\",\"frequency\":0.0125,\"octaves\":3,\"lacunarity\":2,\"gain\":0.5,\"interpolation\":\"CURVE3\"},\"input2\":0.5},\"mask\":{\"type\":\"reterraforged:simplex\",\"frequency\":0.0125,\"octaves\":3,\"lacunarity\":2,\"gain\":0.5,\"interpolation\":\"CURVE3\"},\"slope\":0.75,\"blend_min\":0.1,\"blend_max\":0.8,\"steps\":6,\"octaves\":2}");
        inputs.put("qa:invert","{\"type\":\"reterraforged:invert\",\"input\":{\"type\":\"reterraforged:add\",\"input1\":{\"type\":\"reterraforged:simplex\",\"frequency\":0.0125,\"octaves\":3,\"lacunarity\":2,\"gain\":0.5,\"interpolation\":\"CURVE3\"},\"input2\":0.125}}");
        inputs.put("qa:abs","{\"type\":\"reterraforged:abs\",\"input\":{\"type\":\"reterraforged:add\",\"input1\":{\"type\":\"reterraforged:simplex\",\"frequency\":0.0125,\"octaves\":3,\"lacunarity\":2,\"gain\":0.5,\"interpolation\":\"CURVE3\"},\"input2\":0.125}}");
        inputs.put("qa:blend","{\"type\":\"reterraforged:blend\",\"alpha\":{\"type\":\"reterraforged:add\",\"input1\":{\"type\":\"reterraforged:simplex\",\"frequency\":0.0125,\"octaves\":3,\"lacunarity\":2,\"gain\":0.5,\"interpolation\":\"CURVE3\"},\"input2\":0.125},\"lower\":{\"type\":\"reterraforged:multiply\",\"input1\":{\"type\":\"reterraforged:simplex\",\"frequency\":0.0125,\"octaves\":3,\"lacunarity\":2,\"gain\":0.5,\"interpolation\":\"CURVE3\"},\"input2\":0.5},\"upper\":{\"type\":\"reterraforged:simplex\",\"frequency\":0.0125,\"octaves\":3,\"lacunarity\":2,\"gain\":0.5,\"interpolation\":\"CURVE3\"},\"mid\":0.5,\"range\":0.75,\"interpolation\":\"CURVE3\"}");
        inputs.put("qa:alpha","{\"type\":\"reterraforged:alpha\",\"input\":{\"type\":\"reterraforged:add\",\"input1\":{\"type\":\"reterraforged:simplex\",\"frequency\":0.0125,\"octaves\":3,\"lacunarity\":2,\"gain\":0.5,\"interpolation\":\"CURVE3\"},\"input2\":0.125},\"alpha\":{\"type\":\"reterraforged:multiply\",\"input1\":{\"type\":\"reterraforged:simplex\",\"frequency\":0.0125,\"octaves\":3,\"lacunarity\":2,\"gain\":0.5,\"interpolation\":\"CURVE3\"},\"input2\":0.5}}");
        inputs.put("qa:boost","{\"type\":\"reterraforged:boost\",\"input\":{\"type\":\"reterraforged:add\",\"input1\":{\"type\":\"reterraforged:simplex\",\"frequency\":0.0125,\"octaves\":3,\"lacunarity\":2,\"gain\":0.5,\"interpolation\":\"CURVE3\"},\"input2\":0.125},\"iterations\":2}");
        inputs.put("qa:steps","{\"type\":\"reterraforged:steps\",\"input\":{\"type\":\"reterraforged:add\",\"input1\":{\"type\":\"reterraforged:simplex\",\"frequency\":0.0125,\"octaves\":3,\"lacunarity\":2,\"gain\":0.5,\"interpolation\":\"CURVE3\"},\"input2\":0.125},\"steps\":4,\"slope_min\":0.1,\"slope_max\":0.9,\"slope_curve\":{\"type\":\"reterraforged:interpolation\",\"value\":\"CURVE3\"}}");
        inputs.put("qa:map","{\"type\":\"reterraforged:map\",\"alpha\":{\"type\":\"reterraforged:add\",\"input1\":{\"type\":\"reterraforged:simplex\",\"frequency\":0.0125,\"octaves\":3,\"lacunarity\":2,\"gain\":0.5,\"interpolation\":\"CURVE3\"},\"input2\":0.125},\"from\":{\"type\":\"reterraforged:multiply\",\"input1\":{\"type\":\"reterraforged:simplex\",\"frequency\":0.0125,\"octaves\":3,\"lacunarity\":2,\"gain\":0.5,\"interpolation\":\"CURVE3\"},\"input2\":0.5},\"to\":2}");
        inputs.put("qa:clamp","{\"type\":\"reterraforged:clamp\",\"input\":{\"type\":\"reterraforged:add\",\"input1\":{\"type\":\"reterraforged:simplex\",\"frequency\":0.0125,\"octaves\":3,\"lacunarity\":2,\"gain\":0.5,\"interpolation\":\"CURVE3\"},\"input2\":0.125},\"min\":{\"type\":\"reterraforged:multiply\",\"input1\":{\"type\":\"reterraforged:simplex\",\"frequency\":0.0125,\"octaves\":3,\"lacunarity\":2,\"gain\":0.5,\"interpolation\":\"CURVE3\"},\"input2\":0.5},\"max\":0.9}");
        inputs.put("qa:threshold","{\"type\":\"reterraforged:threshold\",\"input\":{\"type\":\"reterraforged:add\",\"input1\":{\"type\":\"reterraforged:simplex\",\"frequency\":0.0125,\"octaves\":3,\"lacunarity\":2,\"gain\":0.5,\"interpolation\":\"CURVE3\"},\"input2\":0.125},\"lower\":{\"type\":\"reterraforged:multiply\",\"input1\":{\"type\":\"reterraforged:simplex\",\"frequency\":0.0125,\"octaves\":3,\"lacunarity\":2,\"gain\":0.5,\"interpolation\":\"CURVE3\"},\"input2\":0.5},\"upper\":{\"type\":\"reterraforged:simplex\",\"frequency\":0.0125,\"octaves\":3,\"lacunarity\":2,\"gain\":0.5,\"interpolation\":\"CURVE3\"},\"threshold\":0.5}");
        inputs.put("qa:warp","{\"type\":\"reterraforged:warp\",\"input\":{\"type\":\"reterraforged:add\",\"input1\":{\"type\":\"reterraforged:simplex\",\"frequency\":0.0125,\"octaves\":3,\"lacunarity\":2,\"gain\":0.5,\"interpolation\":\"CURVE3\"},\"input2\":0.125},\"domain\":{\"type\":\"reterraforged:domain\",\"x\":{\"type\":\"reterraforged:multiply\",\"input1\":{\"type\":\"reterraforged:simplex\",\"frequency\":0.0125,\"octaves\":3,\"lacunarity\":2,\"gain\":0.5,\"interpolation\":\"CURVE3\"},\"input2\":0.5},\"z\":{\"type\":\"reterraforged:simplex\",\"frequency\":0.0125,\"octaves\":3,\"lacunarity\":2,\"gain\":0.5,\"interpolation\":\"CURVE3\"},\"distance\":16}}");
        inputs.put("qa:erosion","{\"type\":\"reterraforged:erosion\",\"input\":{\"type\":\"reterraforged:add\",\"input1\":{\"type\":\"reterraforged:simplex\",\"frequency\":0.0125,\"octaves\":3,\"lacunarity\":2,\"gain\":0.5,\"interpolation\":\"CURVE3\"},\"input2\":0.125},\"seed\":42,\"octaves\":1,\"strength\":0.6,\"grid_size\":80,\"amplitude\":0.5,\"lacunarity\":2,\"distance_falloff\":0.5,\"blend_mode\":\"constant\"}");
        inputs.put("qa:linear_spline","{\"type\":\"reterraforged:linear_spline\",\"input\":{\"type\":\"reterraforged:add\",\"input1\":{\"type\":\"reterraforged:simplex\",\"frequency\":0.0125,\"octaves\":3,\"lacunarity\":2,\"gain\":0.5,\"interpolation\":\"CURVE3\"},\"input2\":0.125},\"points\":[[0,{\"type\":\"reterraforged:multiply\",\"input1\":{\"type\":\"reterraforged:simplex\",\"frequency\":0.0125,\"octaves\":3,\"lacunarity\":2,\"gain\":0.5,\"interpolation\":\"CURVE3\"},\"input2\":0.5}],[0.5,{\"type\":\"reterraforged:abs\",\"input\":{\"type\":\"reterraforged:simplex\",\"frequency\":0.0125,\"octaves\":3,\"lacunarity\":2,\"gain\":0.5,\"interpolation\":\"CURVE3\"}}],[1,{\"type\":\"reterraforged:add\",\"input1\":{\"type\":\"reterraforged:simplex\",\"frequency\":0.0125,\"octaves\":3,\"lacunarity\":2,\"gain\":0.5,\"interpolation\":\"CURVE3\"},\"input2\":0.125}]]}");
        inputs.put("qa:cache","{\"type\":\"reterraforged:cache\",\"noise\":{\"type\":\"reterraforged:add\",\"input1\":{\"type\":\"reterraforged:simplex\",\"frequency\":0.0125,\"octaves\":3,\"lacunarity\":2,\"gain\":0.5,\"interpolation\":\"CURVE3\"},\"input2\":0.125}}");
        inputs.put("qa:legacy_temperature","{\"type\":\"reterraforged:legacy_temperature\",\"frequency\":0.0125,\"power\":3}");
        inputs.put("qa:legacy_moisture","{\"type\":\"reterraforged:legacy_moisture\",\"source\":{\"type\":\"reterraforged:add\",\"input1\":{\"type\":\"reterraforged:simplex\",\"frequency\":0.0125,\"octaves\":3,\"lacunarity\":2,\"gain\":0.5,\"interpolation\":\"CURVE3\"},\"input2\":0.125},\"power\":3}");
        inputs.put("qa:reference","{\"type\":\"reterraforged:add\",\"input1\":\"reterraforged:terrain/ground\",\"input2\":{\"type\":\"reterraforged:simplex\",\"frequency\":0.0125,\"octaves\":3,\"lacunarity\":2,\"gain\":0.5,\"interpolation\":\"CURVE3\"}}");
        inputs.put("qa:high-perlin","{\"type\":\"reterraforged:perlin\",\"seed\":42,\"frequency\":1e-14,\"octaves\":256,\"lacunarity\":1.1,\"gain\":0.5,\"interpolation\":\"CURVE3\"}");
        inputs.put("qa:high-perlin2","{\"type\":\"reterraforged:perlin2\",\"seed\":42,\"frequency\":1e-14,\"octaves\":256,\"lacunarity\":1.1,\"gain\":0.5,\"interpolation\":\"CURVE3\"}");
        String[] decimals={"-0","0.0","1.0000000596046447753906250000000000001","1.000000059604644775390625","-1.0000000596046447753906250000000000001","1.401298464324817e-45"};
        for(int i=0;i<decimals.length;i++) inputs.put("qa:round/"+i,decimals[i]);
        // Source Codec.pair(Codec.FLOAT, Noise...) feeds an empty remainder to Noise.
        // Observe its actual rejected formats instead of inventing a working wire schema.
        inputs.put("qa:linear_spline_numeric","{\"type\":\"reterraforged:linear_spline\",\"input\":0.5,\"points\":[0,0.5,1]}");
        inputs.put("qa:linear_spline_empty","{\"type\":\"reterraforged:linear_spline\",\"input\":0.5,\"points\":[]}");
        TreeMap<String,Boolean> rejections=new TreeMap<>();
        for(var entry:inputs.entrySet()) if(entry.getKey().startsWith("qa:")) {
            var decoded=Noise.DIRECT_CODEC.parse(ops,JsonParser.parseString(entry.getValue()));
            if(entry.getKey().startsWith("qa:linear_spline")) {
                if(decoded.error().isEmpty()) throw new IllegalStateException("Source spline wire unexpectedly accepted "+entry.getKey());
                rejections.put(entry.getKey(),true);
            } else noises.put(entry.getKey(),decoded.getOrThrow());
        }
        Files.writeString(inputFile,new GsonBuilder().setPrettyPrinting().create().toJson(inputs)+"\n",java.nio.charset.StandardCharsets.UTF_8);
        int[] seeds={0,1,-1,42,Integer.MIN_VALUE,Integer.MAX_VALUE};
        float[] coords={-30_000_000f,-17f,-16.25f,-16f,-1.5f,-1f,-.25f,-0f,0f,.25f,1f,15f,15.75f,16f,16.25f,17f,30_000_000f};
        StringBuilder out=new StringBuilder();
        for(var rejected:rejections.entrySet()) out.append("reject/").append(rejected.getKey()).append("\ttrue\n");
        for(var entry:noises.entrySet()) {
            String id=entry.getKey();Noise noise=entry.getValue();
            out.append("graph/").append(id).append('\t').append(noise.getClass().getSimpleName()).append(',').append(bits(noise.minValue())).append(',').append(bits(noise.maxValue())).append('\n');
            for(int seed:seeds) for(int i=0;i<coords.length;i++) for(int j=0;j<coords.length;j++)
                out.append("sample/").append(id).append('/').append(seed).append('/').append(i).append('/').append(j).append('\t').append(bits(noise.compute(coords[i],coords[j],seed))).append('\n');
        }
        Files.writeString(values,out.toString(),java.nio.charset.StandardCharsets.UTF_8);
    }
}
