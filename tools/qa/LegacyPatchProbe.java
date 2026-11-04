import java.util.*;
import java.lang.reflect.*;
import java.nio.ByteBuffer;
import java.security.MessageDigest;
import net.minecraft.SharedConstants;
import net.minecraft.server.Bootstrap;
import net.minecraft.core.BlockPos;
import net.minecraft.core.Holder;
import net.minecraft.util.RandomSource;
import net.minecraft.world.level.WorldGenLevel;
import net.minecraft.world.level.levelgen.DebugLevelSource;
import net.minecraft.world.level.levelgen.feature.*;
import net.minecraft.world.level.levelgen.feature.configurations.*;
import net.minecraft.world.level.levelgen.placement.PlacedFeature;
import com.mojang.serialization.Codec;

public class LegacyPatchProbe {
  static MessageDigest digest;
  static int placements;
  static class Recording extends Feature<NoneFeatureConfiguration> {
    Recording() { super(NoneFeatureConfiguration.CODEC); }
    public boolean place(FeaturePlaceContext<NoneFeatureConfiguration> c) {
      int nestedDraw=c.random().nextInt(7);
      digest.update(ByteBuffer.allocate(16).putInt(c.origin().getX()).putInt(c.origin().getY()).putInt(c.origin().getZ()).putInt(nestedDraw).array());
      placements++;
      return nestedDraw%3==0;
    }
  }
  @SuppressWarnings({"rawtypes","unchecked"})
  public static void main(String[] args) throws Exception {
    SharedConstants.tryDetectVersion(); Bootstrap.bootStrap();
    digest=MessageDigest.getInstance("SHA-256");
    boolean baseline=args.length>0 && args[0].equals("baseline");
    Class<?> configClass=Class.forName(baseline?"net.minecraft.world.level.levelgen.feature.configurations.RandomPatchConfiguration":"raccoonman.reterraforged.world.worldgen.feature.LegacyRandomPatchFeature$Config");
    Class<?> featureClass=Class.forName(baseline?"net.minecraft.world.level.levelgen.feature.RandomPatchFeature":"raccoonman.reterraforged.world.worldgen.feature.LegacyRandomPatchFeature");
    Feature feature=(Feature)featureClass.getConstructor(Codec.class).newInstance(configClass.getField("CODEC").get(null));
    Holder<PlacedFeature> child=Holder.direct(new PlacedFeature(Holder.direct(new ConfiguredFeature<>(new Recording(),NoneFeatureConfiguration.INSTANCE)),List.of()));
    DebugLevelSource generator=new DebugLevelSource(null);
    WorldGenLevel world=(WorldGenLevel)Proxy.newProxyInstance(LegacyPatchProbe.class.getClassLoader(),new Class<?>[]{WorldGenLevel.class},(proxy,method,values)->switch(method.getName()) {
      case "ensureCanWrite" -> true;
      case "getMinY","getMinBuildHeight" -> -64;
      case "getHeight" -> 384;
      default -> throw new AssertionError("Unexpected world operation: "+method);
    });
    int cases=0;
    for(long seed:new long[]{0,1,-1,Long.MIN_VALUE,Long.MAX_VALUE,123456789})
      for(BlockPos origin:List.of(new BlockPos(0,64,0),new BlockPos(-32768,-20,32768)))
        for(int tries:new int[]{0,1,24,48,56,64})
          for(int[] spread:new int[][]{{7,3},{0,0},{1,9}}) {
            RandomSource random=RandomSource.create(seed);
            FeatureConfiguration config=(FeatureConfiguration)configClass.getConstructor(int.class,int.class,int.class,Holder.class).newInstance(tries,spread[0],spread[1],child);
            boolean success=feature.place(new FeaturePlaceContext(Optional.empty(),world,generator,random,origin,config));
            digest.update((byte)(success?1:0));
            digest.update(ByteBuffer.allocate(8).putLong(random.nextLong()).array());
            cases++;
          }
    System.out.println("PATCH_PROBE_JSON={\"cases\":"+cases+",\"placements\":"+placements+",\"sha256\":\""+HexFormat.of().formatHex(digest.digest())+"\"}");
  }
}
