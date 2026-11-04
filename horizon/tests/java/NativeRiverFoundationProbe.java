package terraforged.qa;
import java.nio.file.*;
import java.lang.reflect.*;
import java.util.*;
import com.google.gson.*;
import com.mojang.serialization.JsonOps;
import raccoonman.reterraforged.data.worldgen.preset.settings.RiverSettings;
import raccoonman.reterraforged.world.worldgen.cell.heightmap.Levels;
import raccoonman.reterraforged.world.worldgen.cell.rivermap.river.*;
import raccoonman.reterraforged.world.worldgen.cell.rivermap.lake.LakeConfig;
import raccoonman.reterraforged.world.worldgen.cell.rivermap.wetland.WetlandConfig;
import raccoonman.reterraforged.world.worldgen.cell.rivermap.gen.GenWarp;
import raccoonman.reterraforged.world.worldgen.noise.NoiseUtil.Vec2i;
import raccoonman.reterraforged.world.worldgen.util.PosUtil;
import raccoonman.reterraforged.world.worldgen.util.Variance;

/** Genuine river configurations, geometry, variance, warp and domain observations. */
public final class NativeRiverFoundationProbe {
    private static String bits(float f) {return Float.isNaN(f)?"NaN":String.format("%08x",Float.floatToRawIntBits(f));}
    private static String pair(long p) {return bits(PosUtil.unpackLeftf(p))+","+bits(PosUtil.unpackRightf(p));}
    private static void fields(String key,Object object,StringBuilder out) throws Exception {
        var fields=Arrays.stream(object.getClass().getFields()).filter(f->!Modifier.isStatic(f.getModifiers())).sorted(Comparator.comparing(Field::getName)).toArray(Field[]::new);
        out.append(key).append('\t');for(int i=0;i<fields.length;i++){if(i>0)out.append(',');Object v=fields[i].get(object);out.append(v instanceof Float f?bits(f):v);}if(object instanceof River r)out.append(',').append(bits(r.getAngle()));out.append('\n');
    }
    private static String warp(RiverWarp w) throws Exception {
        var out=new StringJoiner(",");var seed=RiverWarp.class.getDeclaredField("seed");seed.setAccessible(true);out.add(Integer.toString(seed.getInt(w)));
        for(String name:List.of("lower","upper","lowerRange","upperRange","frequency","scale")){var f=RiverWarp.class.getDeclaredField(name);f.setAccessible(true);out.add(bits(f.getFloat(w)));}return out.toString();
    }
    public static void capture(Path values,Path inputFile) throws Exception {
        String input;try(var stream=NativeRiverFoundationProbe.class.getResourceAsStream("/terraforged-qa-river-foundations.json")){if(stream==null)throw new IllegalStateException("Explicit river inputs missing");input=new String(stream.readAllBytes(),java.nio.charset.StandardCharsets.UTF_8);}
        Files.writeString(inputFile,input,java.nio.charset.StandardCharsets.UTF_8);var root=JsonParser.parseString(input).getAsJsonObject();StringBuilder out=new StringBuilder();
        for(var h:root.getAsJsonArray("heights"))for(var sea:root.getAsJsonArray("seas")){
            int height=h.getAsInt(),water=sea.getAsInt();Levels levels=new Levels(height,water);String prefix=height+"/"+water;fields("config-default/"+prefix,RiverConfig.builder(levels).build(),out);
            var cases=root.getAsJsonArray("builderCases");for(int i=0;i<cases.size();i++){
                var v=cases.get(i).getAsJsonObject();RiverConfig c=RiverConfig.builder(levels).order(v.get("order").getAsInt()).main(v.get("main").getAsBoolean()).bedWidth(v.get("bedWidth").getAsInt()).bankWidth(v.get("bankWidth").getAsInt()).bedDepth(v.get("bedDepth").getAsInt()).bankHeight(v.get("bankMin").getAsInt(),v.get("bankMax").getAsInt()).length(v.get("length").getAsInt()).fade(v.get("fade").getAsFloat()).build();
                fields("config/"+prefix+"/"+i,c,out);for(var width:root.getAsJsonArray("connectBits")){String b=width.getAsString();fields("fork/"+prefix+"/"+i+"/"+b,c.createFork(Float.intBitsToFloat(Integer.parseUnsignedInt(b,16)),levels),out);}
                fields("fork-dimensions/"+prefix+"/"+i,c.createFork(Integer.MIN_VALUE,-1,Integer.MAX_VALUE,levels),out);
            }
            var lakes=root.getAsJsonArray("lakeCases");for(int i=0;i<lakes.size();i++)fields("lake/"+prefix+"/"+i,LakeConfig.of(RiverSettings.Lake.CODEC.parse(JsonOps.INSTANCE,lakes.get(i)).getOrThrow(),levels),out);
        }
        fields("lake-builder/default",new LakeConfig.Builder(),out);
        var wetlands=root.getAsJsonArray("wetlandCases");for(int i=0;i<wetlands.size();i++){
            var w=new WetlandConfig(RiverSettings.Wetland.CODEC.parse(JsonOps.INSTANCE,wetlands.get(i)).getOrThrow());out.append("wetland/").append(i).append('\t').append(w.skipSize).append(',').append(bits(w.length.min())).append(',').append(bits(w.length.range())).append(',').append(bits(w.width.min())).append(',').append(bits(w.width.range())).append('\n');
        }
        var variances=root.getAsJsonArray("variances");for(int i=0;i<variances.size();i++){
            var v=variances.get(i).getAsJsonArray();Variance a=Variance.of(v.get(0).getAsFloat(),v.get(1).getAsFloat());
            out.append("variance-meta/").append(i).append('\t').append(bits(a.min())).append(',').append(bits(a.range())).append(',').append(bits(Variance.min(a.min()).range())).append(',').append(bits(Variance.range(a.range()).min())).append('\n');
            int j=0;for(var value:root.getAsJsonArray("varianceValues"))out.append("variance/").append(i).append('/').append(j++).append('\t').append(bits(a.apply(value.getAsFloat()))).append(',').append(bits(a.apply(value.getAsFloat(),2.5F))).append('\n');
            for(var seed:root.getAsJsonArray("randomSeeds")){Random random=new Random(seed.getAsLong());for(int n=0;n<32;n++)out.append("variance-rng/").append(i).append('/').append(seed.getAsLong()).append('/').append(n).append('\t').append(bits(a.next(random))).append(',').append(bits(a.next(random,2.5F))).append(',').append(random.nextInt()).append('\n');}
        }
        List<River> rivers=new ArrayList<>();var geometries=root.getAsJsonArray("geometries");var radii=root.getAsJsonArray("radii");
        for(int g=0;g<geometries.size();g++){var a=geometries.get(g).getAsJsonArray();for(var radius:radii)rivers.add(new River(a.get(0).getAsFloat(),a.get(1).getAsFloat(),a.get(2).getAsFloat(),a.get(3).getAsFloat(),radius.getAsFloat()));fields("nodes/"+g,River.fromNodes(new Vec2i((int)a.get(0).getAsFloat(),(int)a.get(1).getAsFloat()),new Vec2i((int)a.get(2).getAsFloat(),(int)a.get(3).getAsFloat())),out);}
        for(int i=0;i<rivers.size();i++){
            River r=rivers.get(i);fields("river/"+i,r,out);for(var distance:root.getAsJsonArray("shorten"))fields("shorten/"+i+"/"+distance.getAsInt(),r.shorten(distance.getAsInt()),out);
            var coords=root.getAsJsonArray("coordinates");for(int x=0;x<coords.size();x++)for(int z=0;z<coords.size();z++){float px=coords.get(x).getAsFloat(),pz=coords.get(z).getAsFloat();out.append("bounds/").append(i).append('/').append(x).append('/').append(z).append('\t').append(r.contains(px,pz)).append(',').append(r.overlaps(px,pz,px+1,pz+1)).append(',').append(r.overlaps(new raccoonman.reterraforged.world.worldgen.noise.NoiseUtil.Vec2f(px,pz),275)).append('\n');}
            for(int j=0;j<rivers.size();j++){River other=rivers.get(j);out.append("pair/").append(i).append('/').append(j).append('\t').append(r.intersects(other)).append(',').append(r.overlaps(other)).append('\n');int e=0;for(var extend:root.getAsJsonArray("extend"))out.append("extend/").append(i).append('/').append(j).append('/').append(e++).append('\t').append(r.intersects(other,extend.getAsFloat())).append('\n');}
        }
        List<RiverWarp> warps=new ArrayList<>(List.of(RiverWarp.NONE,new RiverWarp(0,0,0,0,0),new RiverWarp(12345,.1F,.9F,.001F,150),new RiverWarp(Integer.MAX_VALUE,1,0,-.25F,-100)));
        for(var seed:root.getAsJsonArray("randomSeeds")){Random random=new Random(seed.getAsLong());warps.add(RiverWarp.create(.1F,random));warps.add(warps.get(warps.size()-1).createChild(.2F,.8F,.5F,random));warps.add(RiverWarp.NONE.createChild(0,1,0,random));out.append("warp-rng/").append(seed.getAsLong()).append('\t').append(random.nextInt()).append('\n');}
        for(int w=0;w<warps.size();w++){
            RiverWarp warp=warps.get(w);out.append("warp/").append(w).append('\t').append(warp(warp)).append('\n');
            for(int i=0;i<rivers.size();i++)for(var value:root.getAsJsonArray("tBits")){String key=value.getAsString();float t=Float.intBitsToFloat(Integer.parseUnsignedInt(key,16));long pos=rivers.get(i).pos(t);out.append("warp-sample/").append(w).append('/').append(i).append('/').append(key).append('\t').append(warp.test(t)).append(',').append(pair(pos)).append(',').append(pair(warp.getOffset(PosUtil.unpackLeftf(pos),PosUtil.unpackRightf(pos),t,rivers.get(i)))).append(',').append(pair(rivers.get(i).pos(t,warp))).append('\n');}
        }
        for(var seed:root.getAsJsonArray("randomSeeds"))for(var scale:root.getAsJsonArray("genScales")){
            var gen=GenWarp.make((int)seed.getAsLong(),scale.getAsInt());var coords=root.getAsJsonArray("coordinates");for(int i=0;i<coords.size();i++)for(int j=0;j<coords.size();j++){float x=coords.get(i).getAsFloat(),z=coords.get(j).getAsFloat();out.append("gen/").append(seed.getAsLong()).append('/').append(scale.getAsInt()).append('/').append(i).append('/').append(j).append('\t');int n=0;for(var d:List.of(gen.lake(),gen.river(),GenWarp.EMPTY.lake(),GenWarp.EMPTY.river())){if(n++>0)out.append(',');out.append(bits(d.getX(x,z,0))).append(',').append(bits(d.getZ(x,z,0))).append(',').append(bits(d.getOffsetX(x,z,0))).append(',').append(bits(d.getOffsetZ(x,z,0)));}out.append('\n');}
        }
        Files.writeString(values,out.toString(),java.nio.charset.StandardCharsets.UTF_8);
    }
}
