import java.lang.reflect.Field;
import java.util.Arrays;
import java.util.Comparator;
import raccoonman.reterraforged.world.worldgen.cell.Cell;
import raccoonman.reterraforged.world.worldgen.cell.biome.type.BiomeType;
import raccoonman.reterraforged.world.worldgen.cell.terrain.TerrainType;
import raccoonman.reterraforged.world.worldgen.cell.terrain.Terrain;
import raccoonman.reterraforged.world.worldgen.cell.heightmap.Levels;
import raccoonman.reterraforged.world.worldgen.densityfunction.tile.Size;

/** Genuine Java helper/value observations; no native expected values. */
public final class NativeCellFoundationProbe {
    private static String raw(float f) {return String.format("%08x",Float.floatToRawIntBits(f));}
    private static String bits(float f) {return Float.isNaN(f)?"NaN":raw(f);}
    private static final int[] INTS={Integer.MIN_VALUE,-1024,-17,-16,-1,0,1,15,16,17,63,256,Integer.MAX_VALUE};
    private static final int[] FLOATS={0,0x80000000,1,0x7f7fffff,0xbf800000,0x7fc01234};
    private static Field[] fields() {
        Field[] fields=Arrays.stream(Cell.class.getFields()).filter(f->!java.lang.reflect.Modifier.isStatic(f.getModifiers())).toArray(Field[]::new);
        Arrays.sort(fields,Comparator.comparing(Field::getName));
        if(fields.length!=24) throw new IllegalStateException("Unexpected Cell schema");
        return fields;
    }
    private static void snapshot(String name,Cell cell) throws Exception {
        for(Field f:fields()) {
            Object value=f.get(cell);
            String text=value instanceof Float x?raw(x):value instanceof Terrain t?Integer.toString(t.getId()):value instanceof BiomeType b?Integer.toString(b.ordinal()):String.valueOf(value);
            System.out.println("cell/"+name+"/"+f.getName()+"\t"+text);
        }
    }
    private static void size(String name,Size size) {
        System.out.println("size/meta/"+name+"\t"+size.size()+","+size.mask()+","+size.border()+","+size.total()+","+size.lowerBorder()+","+size.upperBorder()+","+size.arraySize());
        for(int i=0;i<INTS.length;i++) for(int j=0;j<INTS.length;j++)
            System.out.println("size/index/"+name+"/"+i+"/"+j+"\t"+size.indexOf(INTS[i],INTS[j])+","+size.mask(INTS[i])+","+size.mask(INTS[j]));
    }
    public static void main(String[] args) throws Exception {
        int[] heights={Integer.MIN_VALUE,-1,0,1,2,256,384,1024,Integer.MAX_VALUE};
        int[] seas={Integer.MIN_VALUE,-1,0,1,63,255,256,Integer.MAX_VALUE};
        float[] values={-Float.MAX_VALUE,-1.5F,-1.0F,-Float.MIN_VALUE,-0.0F,0.0F,Float.MIN_VALUE,0.25F,0.5F,1.0F,1.5F,Float.MAX_VALUE,Float.POSITIVE_INFINITY,Float.NaN};
        for(int h=0;h<heights.length;h++) for(int s=0;s<seas.length;s++) {
            Levels l=new Levels(heights[h],seas[s]);String key=h+"/"+s;
            System.out.println("levels/meta/"+key+"\t"+l.worldHeight+","+bits(l.unit)+","+l.waterY+","+l.groundLevel+","+l.waterLevel+","+bits(l.ground)+","+bits(l.water));
            for(int i=0;i<INTS.length;i++) {int y=INTS[i];System.out.println("levels/int/"+key+"/"+i+"\t"+bits(l.scale(y))+","+bits(l.elevation(y))+","+bits(l.water(y))+","+bits(l.ground(y)));}
            for(int i=0;i<values.length;i++) {float f=values[i];System.out.println("levels/float/"+key+"/"+i+"\t"+l.scale(f)+","+bits(l.elevation(f)));}
        }
        int[] factors={-32,-1,0,1,2,3,4,30,31,32,33};
        int[] borders={-1,0,1,2,Integer.MIN_VALUE,Integer.MAX_VALUE};
        int[] sizes={-1,0,1,3,128,Integer.MAX_VALUE};
        for(int i=0;i<sizes.length;i++) for(int j=0;j<borders.length;j++) size("make/"+i+"/"+j,Size.make(sizes[i],borders[j]));
        for(int i=0;i<factors.length;i++) for(int j=0;j<borders.length;j++) {size("chunks/"+i+"/"+j,Size.chunks(factors[i],borders[j]));size("blocks/"+i+"/"+j,Size.blocks(factors[i],borders[j]));}
        for(int i=0;i<INTS.length;i++) {
            System.out.println("size/convert/"+i+"\t"+Size.chunkToBlock(INTS[i])+","+Size.blockToChunk(INTS[i]));
            for(int j=0;j<INTS.length;j++) System.out.println("size/count/"+i+"/"+j+"\t"+Size.count(INTS[i],INTS[j],INTS[j],INTS[i]));
        }
        snapshot("default",new Cell());
        for(int pattern=0;pattern<2;pattern++) {
            Cell source=new Cell();Field[] fields=fields();
            for(int i=0;i<fields.length;i++) {Field f=fields[i];
                if(f.getType()==float.class) f.setFloat(source,Float.intBitsToFloat(FLOATS[(i+pattern)%FLOATS.length]));
                else if(f.getType()==int.class) f.setInt(source,Integer.MIN_VALUE+i+pattern*100);
                else if(f.getType()==boolean.class) f.setBoolean(source,true);
                else if(f.getType()==Terrain.class) f.set(source,TerrainType.FLATS);
                else if(f.getType()==BiomeType.class) f.set(source,BiomeType.TAIGA);
            }
            snapshot(pattern+"/filled",source);Cell copy=new Cell();copy.copyFrom(source);snapshot(pattern+"/copy",copy);
            source.height=-999.0F;snapshot(pattern+"/detached",copy);source.reset();snapshot(pattern+"/reset",source);
        }
    }
}
