package terraforged.qa;
import java.nio.file.*;
import java.lang.reflect.*;
import java.util.*;
import com.google.gson.*;
import com.mojang.serialization.JsonOps;
import raccoonman.reterraforged.data.worldgen.preset.settings.RiverSettings;
import raccoonman.reterraforged.world.worldgen.cell.Cell;
import raccoonman.reterraforged.world.worldgen.cell.biome.type.BiomeType;
import raccoonman.reterraforged.world.worldgen.cell.heightmap.Levels;
import raccoonman.reterraforged.world.worldgen.cell.terrain.*;
import raccoonman.reterraforged.world.worldgen.cell.rivermap.river.*;
import raccoonman.reterraforged.world.worldgen.cell.rivermap.lake.*;
import raccoonman.reterraforged.world.worldgen.cell.rivermap.wetland.Wetland;
import raccoonman.reterraforged.world.worldgen.noise.NoiseUtil.Vec2f;
import raccoonman.reterraforged.world.worldgen.noise.function.CurveFunctions;
import raccoonman.reterraforged.world.worldgen.util.Boundsf;

/** Original river/lake/wetland mutations, bounds and double-round observations. */
public final class NativeRiverCarverProbe {
    private static final Field[] FIELDS=Arrays.stream(Cell.class.getFields()).filter(f->!Modifier.isStatic(f.getModifiers())).sorted(Comparator.comparing(Field::getName)).toArray(Field[]::new);
    private static String bits(float f){return Float.isNaN(f)?"NaN":String.format("%08x",Float.floatToRawIntBits(f));}
    private static Cell cell(JsonObject v)throws Exception{Cell c=new Cell();for(Field f:FIELDS){if(f.getType()==float.class)f.setFloat(c,-3.25F);else if(f.getType()==int.class)f.setInt(c,-123456);}c.height=v.get("height").getAsFloat();c.riverMask=v.get("mask").getAsFloat();c.terrain=TerrainType.get(v.get("terrain").getAsInt());c.continentEdge=v.get("edge").getAsFloat();return c;}
    private static void snapshot(String key,Cell c,StringBuilder out)throws Exception{out.append(key).append('\t');for(int i=0;i<FIELDS.length;i++){if(i>0)out.append(',');Object v=FIELDS[i].get(c);out.append(v instanceof Float f?bits(f):v instanceof Terrain t?t.getId():v instanceof BiomeType b?b.ordinal():v);}out.append('\n');}
    private static RiverConfig config(JsonObject v,Levels l){return RiverConfig.builder(l).order(v.get("order").getAsInt()).main(v.get("main").getAsBoolean()).bedWidth(v.get("bedWidth").getAsInt()).bankWidth(v.get("bankWidth").getAsInt()).bedDepth(v.get("bedDepth").getAsInt()).bankHeight(v.get("bankMin").getAsInt(),v.get("bankMax").getAsInt()).length(v.get("length").getAsInt()).fade(v.get("fade").getAsFloat()).build();}
    private static void configRow(String key,RiverConfig c,StringBuilder out){out.append(key).append('\t').append(c.bankWidth).append(',').append(bits(c.bedHeight)).append(',').append(c.bedWidth).append(',').append(bits(c.fade)).append(',').append(c.length).append(',').append(c.length2).append(',').append(c.main).append(',').append(bits(c.maxBankHeight)).append(',').append(bits(c.minBankHeight)).append(',').append(c.order).append('\n');}
    private static void bounds(String key,Boundsf b,StringBuilder out){out.append(key).append('\t').append(bits(b.minX())).append(',').append(bits(b.minY())).append(',').append(bits(b.maxX())).append(',').append(bits(b.maxY())).append('\n');}
    private static void carve(String prefix,RiverCarver c,Levels levels,JsonObject root,StringBuilder out)throws Exception{
        for(var value:root.getAsJsonArray("tBits")){String key=value.getAsString();float t=Float.intBitsToFloat(Integer.parseUnsignedInt(key,16));configRow("fork/"+prefix+"/"+key,c.createForkConfig(t,levels),out);int o=0;
            for(var offset:root.getAsJsonArray("offsets")){float distance=offset.getAsFloat();float x=c.river.x1+c.river.dx*t+c.river.normX*distance,z=c.river.z1+c.river.dz*t+c.river.normZ*distance;int v=0;
                for(var input:root.getAsJsonArray("cellCases")){Cell cell=cell(input.getAsJsonObject());c.carve(cell,x+3,z-5,t+.01F,x,z,t);snapshot("carve/"+prefix+"/"+key+"/"+o+"/"+v++,cell,out);}o++;
            }
        }
    }
    public static void capture(Path values,Path inputFile)throws Exception{
        String input;try(var stream=NativeRiverCarverProbe.class.getResourceAsStream("/terraforged-qa-river-carvers.json")){if(stream==null)throw new IllegalStateException("Explicit carving inputs missing");input=new String(stream.readAllBytes(),java.nio.charset.StandardCharsets.UTF_8);}
        Files.writeString(inputFile,input,java.nio.charset.StandardCharsets.UTF_8);var root=JsonParser.parseString(input).getAsJsonObject();StringBuilder out=new StringBuilder();
        var levelInputs=root.getAsJsonArray("levels");for(int l=0;l<levelInputs.size();l++){var lv=levelInputs.get(l).getAsJsonArray();Levels levels=new Levels(lv.get(0).getAsInt(),lv.get(1).getAsInt());
            var gs=root.getAsJsonArray("geometries");var cs=root.getAsJsonArray("profiles");var ss=root.getAsJsonArray("carverSettings");
            for(int g=0;g<gs.size();g++){var a=gs.get(g).getAsJsonArray();River river=new River(a.get(0).getAsFloat(),a.get(1).getAsFloat(),a.get(2).getAsFloat(),a.get(3).getAsFloat());
                for(int c=0;c<cs.size();c++)for(int s=0;s<ss.size();s++){var v=ss.get(s).getAsJsonObject();var settings=new RiverCarver.Settings();settings.valleySize=v.get("valleySize").getAsFloat();settings.fadeIn=v.get("fadeIn").getAsFloat();settings.connecting=v.get("connecting").getAsBoolean();var curve=v.getAsJsonArray("curve");settings.valleyCurve=CurveFunctions.scurve(curve.get(0).getAsFloat(),curve.get(1).getAsFloat());carve("explicit/"+l+"/"+g+"/"+c+"/"+s,new RiverCarver(river,RiverWarp.NONE,config(cs.get(c).getAsJsonObject(),levels),settings,levels),levels,root,out);}
            }
            for(var seed:root.getAsJsonArray("randomSeeds")){Random r=new Random(seed.getAsLong());var c=RiverCarver.create(0,0,1000,250,config(cs.get(0).getAsJsonObject(),levels),levels,r);out.append("create-rng/").append(l).append('/').append(seed.getAsLong()).append('\t').append(r.nextInt()).append('\n');carve("random/"+l+"/"+seed.getAsLong(),c,levels,root,out);}
            var centers=root.getAsJsonArray("lakeCenters");var shapes=root.getAsJsonArray("lakeShapes");var lakes=root.getAsJsonArray("lakeCases");var grid=root.getAsJsonArray("grid");
            for(int c=0;c<centers.size();c++)for(int s=0;s<shapes.size();s++)for(int cfg=0;cfg<lakes.size();cfg++){var center=centers.get(c).getAsJsonArray();float cx=center.get(0).getAsFloat(),cz=center.get(1).getAsFloat();var shape=shapes.get(s).getAsJsonArray();var lake=new Lake(new Vec2f(cx,cz),shape.get(0).getAsFloat(),shape.get(1).getAsFloat(),LakeConfig.of(RiverSettings.Lake.CODEC.parse(JsonOps.INSTANCE,lakes.get(cfg)).getOrThrow(),levels));String prefix=l+"/"+c+"/"+s+"/"+cfg;var builder=Boundsf.builder();lake.recordBounds(builder);var box=builder.build();bounds("lake-bounds/"+prefix,box,out);
                for(int x=0;x<grid.size();x++)for(int z=0;z<grid.size();z++){float px=cx+grid.get(x).getAsFloat(),pz=cz+grid.get(z).getAsFloat();out.append("lake-geometry/").append(prefix).append('/').append(x).append('/').append(z).append('\t').append(lake.overlaps(px,pz,900)).append(',').append(box.contains(px,pz)).append('\n');int v=0;for(var inputCase:root.getAsJsonArray("cellCases")){Cell cell=cell(inputCase.getAsJsonObject());lake.apply(cell,px,pz);snapshot("lake/"+prefix+"/"+x+"/"+z+"/"+v++,cell,out);}}
            }
            var radii=root.getAsJsonArray("wetlandRadii");for(var seed:root.getAsJsonArray("randomSeeds"))for(int g=0;g<gs.size();g++)for(int radius=0;radius<radii.size();radius++){var a=gs.get(g).getAsJsonArray();float cx=a.get(0).getAsFloat(),cz=a.get(1).getAsFloat();var wet=new Wetland((int)seed.getAsLong(),new Vec2f(cx,cz),new Vec2f(a.get(2).getAsFloat(),a.get(3).getAsFloat()),radii.get(radius).getAsFloat(),levels);String prefix=l+"/"+seed.getAsLong()+"/"+g+"/"+radius;var builder=Boundsf.builder();wet.recordBounds(builder);bounds("wetland-bounds/"+prefix,builder.build(),out);
                for(int x=0;x<grid.size();x++)for(int z=0;z<grid.size();z++){float px=cx+grid.get(x).getAsFloat(),pz=cz+grid.get(z).getAsFloat();int v=0;for(var inputCase:root.getAsJsonArray("cellCases")){Cell cell=cell(inputCase.getAsJsonObject());wet.apply(cell,px+3,pz-5,px,pz);snapshot("wetland/"+prefix+"/"+x+"/"+z+"/"+v++,cell,out);}}
            }
        }
        for(var seed:root.getAsJsonArray("randomSeeds")){Random random=new Random(seed.getAsLong());for(int index=0;index<64;index++){var curve=RiverCarver.getValleyType(random);out.append("valley-rng/").append(seed.getAsLong()).append('/').append(index).append('\t');int n=0;for(var value:root.getAsJsonArray("curveValues")){if(n++>0)out.append(',');out.append(bits(curve.apply(value.getAsFloat())));}out.append(',').append(random.nextInt()).append('\n');}}
        bounds("bounds-empty",Boundsf.builder().build(),out);bounds("bounds-none",Boundsf.NONE,out);
        var negative=Boundsf.builder();negative.record(-1000,-2000);negative.record(-5,-2);bounds("bounds-negative",negative.build(),out);
        for(var b:root.getAsJsonArray("doubleRoundBits")){String key=b.getAsString();out.append("round-double/").append(key).append('\t').append(Math.round(Double.longBitsToDouble(Long.parseUnsignedLong(key,16)))).append('\n');}
        Files.writeString(values,out.toString(),java.nio.charset.StandardCharsets.UTF_8);
    }
}
