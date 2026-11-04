import net.minecraft.SharedConstants;
import net.minecraft.server.Bootstrap;
import net.minecraft.core.*;
import net.minecraft.data.worldgen.BootstrapContext;
import net.minecraft.data.registries.VanillaRegistries;
import net.minecraft.resources.*;
import net.minecraft.world.level.dimension.*;
import com.mojang.serialization.*;
import raccoonman.reterraforged.data.worldgen.OverworldDimensionData;
public class DimensionDataProbe {
 public static void main(String[] a) {
  SharedConstants.tryDetectVersion();Bootstrap.bootStrap();
  HolderLookup.Provider provider=VanillaRegistries.createLookup();
  BootstrapContext<DimensionType> ctx=new BootstrapContext<>() {
   public Holder.Reference<DimensionType> register(ResourceKey<DimensionType> key,DimensionType val,Lifecycle lifecycle){throw new AssertionError("No registry write expected");}
   public <S> HolderGetter<S> lookup(ResourceKey<? extends Registry<? extends S>> key){return provider.lookupOrThrow(key);}
  };
  var ops=provider.createSerializationContext(JsonOps.INSTANCE);
  var expected=DimensionType.DIRECT_CODEC.encodeStart(ops,provider.lookupOrThrow(net.minecraft.core.registries.Registries.DIMENSION_TYPE).getOrThrow(BuiltinDimensionTypes.OVERWORLD).value()).getOrThrow();
  var actual=DimensionType.DIRECT_CODEC.encodeStart(ops,OverworldDimensionData.create(ctx,64,320)).getOrThrow();
  if(!expected.equals(actual))throw new AssertionError("Default overworld differs");
  int cases=0;
  for(int depth:new int[]{0,64,128})for(int height:new int[]{256,320,512}) {
   DimensionType type=OverworldDimensionData.create(ctx,depth,height);
   if(type.minY()!=-depth || type.height()!=depth+height || type.logicalHeight()!=depth+height)throw new AssertionError("Bounds not preserved");

   var encoded=DimensionType.DIRECT_CODEC.encodeStart(ops,type).getOrThrow();
   var object=encoded.getAsJsonObject();
   if(object.get("min_y").getAsInt()!=-depth || object.get("height").getAsInt()!=depth+height || object.get("logical_height").getAsInt()!=depth+height)throw new AssertionError("Encoded bounds drift");
   cases++;
  }
  System.out.println("DIMENSION_PROBE_JSON={\"defaultOverworldMatchesVanilla\":true,\"heightCases\":"+cases+",\"encodedHeightCases\":"+cases+"}");
 }
}
