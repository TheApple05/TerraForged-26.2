package raccoonman.reterraforged.world.worldgen.feature;

import com.mojang.serialization.Codec;
import com.mojang.serialization.codecs.RecordCodecBuilder;
import net.minecraft.core.BlockPos;
import net.minecraft.core.Holder;
import net.minecraft.util.RandomSource;
import net.minecraft.world.level.levelgen.feature.Feature;
import net.minecraft.world.level.levelgen.feature.ConfiguredFeature;
import net.minecraft.world.level.levelgen.feature.FeaturePlaceContext;
import net.minecraft.world.level.levelgen.feature.configurations.FeatureConfiguration;
import net.minecraft.world.level.levelgen.placement.PlacedFeature;

/** Keeps legacy patch attempt counts and random-consumption order on 26.2. */
public final class LegacyRandomPatchFeature extends Feature<LegacyRandomPatchFeature.Config> {
    public LegacyRandomPatchFeature(Codec<Config> codec) {
        super(codec);
    }

    @Override
    public boolean place(FeaturePlaceContext<Config> context) {
        Config config = context.config();
        RandomSource random = context.random();
        BlockPos.MutableBlockPos candidate = new BlockPos.MutableBlockPos();
        boolean placed = false;
        for (int attempt = 0; attempt < config.tries(); attempt++) {
            offset(candidate, context.origin(), random, config.xzSpread(), config.ySpread());
            // Do not short-circuit: all attempts must consume their random draws,
            // including the random draws made by the nested placed feature.
            boolean success = config.feature().value().place(context.level(), context.chunkGenerator(), random, candidate);
            placed |= success;
        }
        return placed;
    }

    public static void offset(BlockPos.MutableBlockPos output, BlockPos origin, RandomSource random, int xzSpread, int ySpread) {
        int x = random.nextInt(xzSpread + 1) - random.nextInt(xzSpread + 1);
        int y = random.nextInt(ySpread + 1) - random.nextInt(ySpread + 1);
        int z = random.nextInt(xzSpread + 1) - random.nextInt(xzSpread + 1);
        output.setWithOffset(origin, x, y, z);
    }

    public record Config(int tries, int xzSpread, int ySpread, Holder<PlacedFeature> feature) implements FeatureConfiguration {
        public static final Codec<Config> CODEC = RecordCodecBuilder.create(instance -> instance.group(
            Codec.intRange(0, Integer.MAX_VALUE).fieldOf("tries").forGetter(Config::tries),
            Codec.intRange(0, Integer.MAX_VALUE - 1).fieldOf("xz_spread").forGetter(Config::xzSpread),
            Codec.intRange(0, Integer.MAX_VALUE - 1).fieldOf("y_spread").forGetter(Config::ySpread),
            PlacedFeature.CODEC.fieldOf("feature").forGetter(Config::feature)
        ).apply(instance, Config::new));

        public Config {
            if (tries < 0 || xzSpread < 0 || ySpread < 0 || xzSpread == Integer.MAX_VALUE || ySpread == Integer.MAX_VALUE) {
                throw new IllegalArgumentException("Invalid legacy patch attempt count or spread");
            }
        }

        @Override
        public java.util.stream.Stream<Holder<ConfiguredFeature<?, ?>>> getSubFeatures() {
            return this.feature.value().getFeatures();
        }
    }
}
