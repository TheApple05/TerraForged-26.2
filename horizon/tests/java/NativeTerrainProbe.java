import java.util.*;
import raccoonman.reterraforged.world.worldgen.cell.terrain.*;

/** Observe genuine source registry identity, inheritance and composite quirks. */
public final class NativeTerrainProbe {
    private static String bits(float f) {return String.format("%08x",Float.floatToRawIntBits(f));}
    private static void snapshot(String key,Terrain t) {
        System.out.println(key+"\t"+t.getId()+","+t.getName()+","+t.getCategory().name()+","+bits(t.erosionModifier())+","+bits(t.getRenderHue())+","
            +t.isFlat()+","+t.isRiver()+","+t.isShallowOcean()+","+t.isDeepOcean()+","+t.isCoast()+","+t.isSubmerged()+","+t.isOverground()+","
            +t.overridesRiver()+","+t.overridesCoast()+","+t.isLake()+","+t.isWetland()+","+t.isMountain()+","+t.isVolcano());
    }
    public static void main(String[] args) {
        List<Terrain> initial=new ArrayList<>();TerrainType.forEach(initial::add);
        if(initial.size()!=18) throw new IllegalStateException("Unexpected initial terrain registry");
        for(Terrain t:initial) snapshot("builtin/"+t.getId(),t);
        snapshot("alias/plain",TerrainType.getOrCreate("qa_plain",TerrainType.FLATS));
        snapshot("alias/case-collision",TerrainType.getOrCreate("QA_PLAIN",TerrainType.MOUNTAINS_1));
        snapshot("alias/mountain",TerrainType.getOrCreate("qa_mountain",TerrainType.MOUNTAINS_1));
        snapshot("alias/badlands",TerrainType.getOrCreate("qa_badlands",TerrainType.BADLANDS));
        snapshot("alias/volcano",TerrainType.getOrCreate("qa_volcano",TerrainType.VOLCANO));
        snapshot("alias/none",TerrainType.getOrCreate("qa_none",TerrainType.NONE));
        snapshot("alias/null",TerrainType.getOrCreate("qa_null",null));
        snapshot("alias/existing-builtin",TerrainType.getOrCreate("FLATS",TerrainType.VOLCANO));
        // A preexisting alias shadows the same future composite name in the source registry.
        snapshot("alias/shadow",TerrainType.getOrCreate("flats-hills",TerrainType.VOLCANO));
        List<Terrain> inputs=new ArrayList<>();TerrainType.forEach(inputs::add);
        for(int i=0;i<inputs.size();i++) for(int j=0;j<inputs.size();j++)
            snapshot("composite/"+i+"/"+j,TerrainType.registerComposite(inputs.get(i),inputs.get(j)));
        Terrain a=TerrainType.registerComposite(TerrainType.FLATS,TerrainType.MOUNTAINS_1);
        Terrain b=TerrainType.registerComposite(TerrainType.BADLANDS,TerrainType.VOLCANO);
        snapshot("nested/a",a);snapshot("nested/b",b);snapshot("nested/mix",TerrainType.registerComposite(a,b));
        snapshot("nested/alias",TerrainType.getOrCreate("qa_inherited_mix",TerrainType.registerComposite(a,b)));
        for(int id:new int[]{Integer.MIN_VALUE,-1,0,8,17,18,100,Integer.MAX_VALUE}) snapshot("get/"+id,TerrainType.get(id));
        for(String name:new String[]{"qa_plain","QA_PLAIN","FLATS-HILLS","qa_inherited_mix","not_present"}) {
            Terrain found=TerrainType.get(name);System.out.println("find/"+name+"\t"+(found==null?"missing":found.getId()));
        }
        for(TerrainCategory x:TerrainCategory.values()) for(TerrainCategory z:TerrainCategory.values())
            System.out.println("category/"+x.ordinal()+"/"+z.ordinal()+"\t"+x.getDominant(z).name());
        System.out.println("final-count\t"+TerrainType.find(t->t.getName().equals("qa_inherited_mix")).orElseThrow().getId());
    }
}
