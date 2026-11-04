/* MIT License. Copyright (c) 2021 TerraForged. See ClassicPillarMath.java and LICENSE. */
package raccoonman.reterraforged.world.worldgen.feature;

import java.util.Random;

import com.mojang.serialization.Codec;
import com.mojang.serialization.codecs.RecordCodecBuilder;

import net.minecraft.core.BlockPos;
import net.minecraft.core.Holder;
import net.minecraft.core.QuartPos;
import net.minecraft.resources.ResourceKey;
import net.minecraft.tags.BlockTags;
import net.minecraft.world.level.ChunkPos;
import net.minecraft.world.level.WorldGenLevel;
import net.minecraft.world.level.biome.Biome;
import net.minecraft.world.level.block.Blocks;
import net.minecraft.world.level.block.state.BlockState;
import net.minecraft.world.level.chunk.ChunkAccess;
import net.minecraft.world.level.levelgen.Heightmap;
import net.minecraft.world.level.levelgen.feature.Feature;
import net.minecraft.world.level.levelgen.feature.FeaturePlaceContext;
import net.minecraft.world.level.levelgen.feature.configurations.FeatureConfiguration;
import raccoonman.reterraforged.data.worldgen.preset.PresetBiomeData;
import raccoonman.reterraforged.world.worldgen.GeneratorContext;
import raccoonman.reterraforged.world.worldgen.RTFRandomState;
import raccoonman.reterraforged.world.worldgen.cell.Cell;
import raccoonman.reterraforged.world.worldgen.densityfunction.tile.Tile;
import raccoonman.reterraforged.world.worldgen.noise.module.Noise;
import raccoonman.reterraforged.world.worldgen.noise.module.Noises;

/** Classic pillar surfaces adapted to modern chunk height, biome boundaries and structure ownership. */
public final class ClassicPillarFeature extends Feature<ClassicPillarFeature.Config> {
    private static final Noise BRYCE = ridge(12341342, 60, 0.8F, 0.95F, 0.999F, false);
    private static final Noise STONE_A = ridge(-2345768, 50, 0.7F, 0.95F, 1, true);
    private static final Noise STONE_B = ridge(-2345767, 55, 0.7F, 0.95F, 1, true);

    public ClassicPillarFeature(Codec<Config> codec) { super(codec); }

    @Override
    public boolean place(FeaturePlaceContext<Config> placement) {
        WorldGenLevel level = placement.level();
        var state = level.getLevel().getChunkSource().randomState();
        GeneratorContext context = ((Object) state instanceof RTFRandomState rtf) ? rtf.generatorContext() : null;
        if (context == null || !context.preset.miscellaneous().classicBiomeVariants) return false;
        boolean stone = placement.config().stoneForest();
        ResourceKey<Biome> key = stone ? PresetBiomeData.STONE_FOREST : PresetBiomeData.BRYCE;
        ChunkPos chunkPos = ChunkPos.containing(placement.origin());
        ChunkAccess chunk = level.getChunk(chunkPos.x(), chunkPos.z());
        Tile.Chunk tile = context.cache.provideAtChunk(chunkPos.x(), chunkPos.z()).getChunkReader(chunkPos.x(), chunkPos.z());
        int seed = context.seed.root();
        Palette[] palettes = stone ? null : palettes(seed);
        // This baseline's Perlin stores its seed internally; unlike PerlinRidge,
        // it does not use compute's seed argument.
        Noise detail = stone ? null : Noises.alpha(Noises.perlin(seed + 12341343, 4, 1), 0.05F);
        Noise bands = stone ? null : Noises.perlin(seed + 12341345, 100, 1);
        BlockPos.MutableBlockPos pos = new BlockPos.MutableBlockPos();
        boolean changed = false;
        for (int z = 0; z < 16; z++) {
            for (int x = 0; x < 16; x++) {
                int worldX = chunkPos.getBlockX(x), worldZ = chunkPos.getBlockZ(z);
                int surfaceY = chunk.getHeight(Heightmap.Types.OCEAN_FLOOR, x, z);
                if (surfaceY <= placement.chunkGenerator().getSeaLevel() || surfaceY >= level.getMaxY()) continue;
                BlockState base = chunk.getBlockState(pos.set(worldX, surfaceY, worldZ));
                if (!base.is(BlockTags.BASE_STONE_OVERWORLD) && !base.is(BlockTags.DIRT)
                        && !base.is(BlockTags.SAND) && !base.is(BlockTags.TERRACOTTA)) continue;
                Holder<Biome> biome = level.getNoiseBiome(QuartPos.fromBlock(worldX), QuartPos.fromBlock(surfaceY), QuartPos.fromBlock(worldZ));
                if (!biome.is(key)) continue;
                Cell cell = tile.getCell(x, z);
                float mask = ClassicPillarMath.mask(cell.biomeRegionEdge, cell.riverMask, cell.gradient);
                mask *= boundaryMask(level, worldX, surfaceY, worldZ, key);
                float shape = stone
                    ? Math.max(ClassicPillarMath.terrace(STONE_A.compute(worldX, worldZ, seed), 1), ClassicPillarMath.terrace(STONE_B.compute(worldX, worldZ, seed), 1))
                    : ClassicPillarMath.terrace(BRYCE.compute(worldX, worldZ, seed), 0.999F) * detail.compute(worldX, worldZ, seed);
                int height = ClassicPillarMath.addedHeight(shape, mask, stone);
                int top = Math.min(surfaceY + height, level.getMaxY());
                if (top <= surfaceY) continue;
                // Structures have already placed their blocks. Do not overwrite them or fluids.
                boolean empty = true;
                for (int y = surfaceY + 1; y <= top; y++) {
                    if (!chunk.getBlockState(pos.set(worldX, y, worldZ)).isAir()) { empty = false; break; }
                }
                if (!empty) continue;
                Palette palette = stone ? null : palettes[Math.min(8, (int) (Math.max(0, Math.min(0.999999F, cell.biomeRegionId)) * 9))];
                int shift = stone ? 0 : (int) (bands.compute(worldX, worldZ, seed) * 4);
                for (int y = surfaceY + 1; y <= top; y++) {
                    BlockState material = stone ? (y == top ? Blocks.GRASS_BLOCK.defaultBlockState() : y == top - 1 ? Blocks.DIRT.defaultBlockState() : Blocks.STONE.defaultBlockState())
                        : palette.at(y - surfaceY + shift, Math.max(1, height));
                    chunk.setBlockState(pos.set(worldX, y, worldZ), material);
                }
                changed = true;
            }
        }
        return changed;
    }

    private static Noise ridge(int offset, int scale, float lower, float upper, float maximum, boolean power) {
        Noise value = Noises.map(Noises.clamp(Noises.perlinRidge(offset, scale, 4), lower, upper), 0, maximum);
        return power ? Noises.pow(value, 1.5F) : value;
    }

    private static float boundaryMask(WorldGenLevel level, int x, int y, int z, ResourceKey<Biome> key) {
        float mask = 1;
        int quartY = QuartPos.fromBlock(y);
        for (int distance = 4; distance <= 16; distance += 4) {
            // Exact quart lookups stay inside the FEATURES 3x3 region. BiomeManager's
            // fuzzy block lookup can reach an extra chunk beyond a +/-16-block probe.
            if (!level.getNoiseBiome(QuartPos.fromBlock(x + distance), quartY, QuartPos.fromBlock(z)).is(key)
                || !level.getNoiseBiome(QuartPos.fromBlock(x - distance), quartY, QuartPos.fromBlock(z)).is(key)
                || !level.getNoiseBiome(QuartPos.fromBlock(x), quartY, QuartPos.fromBlock(z + distance)).is(key)
                || !level.getNoiseBiome(QuartPos.fromBlock(x), quartY, QuartPos.fromBlock(z - distance)).is(key)) {
                mask = Math.min(mask, distance / 16.0F);
                break;
            }
        }
        return mask;
    }

    private static Palette[] palettes(int seed) {
        Random random = new Random(seed + 12341341);
        Palette[] palettes = new Palette[9];
        for (int i = 0; i < palettes.length; i++) {
            BlockState[] materials = {Blocks.DYED_TERRACOTTA.orange().defaultBlockState(), Blocks.TERRACOTTA.defaultBlockState(),
                Blocks.DYED_TERRACOTTA.orange().defaultBlockState(), (i % 3 == 2 ? Blocks.DYED_TERRACOTTA.white() : i % 3 == 1 ? Blocks.DYED_TERRACOTTA.yellow() : Blocks.DYED_TERRACOTTA.brown()).defaultBlockState(),
                (i % 3 == 0 ? Blocks.DYED_TERRACOTTA.yellow() : Blocks.DYED_TERRACOTTA.lightGray()).defaultBlockState(), Blocks.TERRACOTTA.defaultBlockState()};
            float[] widths = new float[6];
            float total = 0;
            for (int band = 0; band < widths.length; band++) {
                total += widths[band] = (band == 2 || band == 5 ? 0.2F : 0.1F) + random.nextFloat() * 0.1F;
            }
            for (int band = 1; band < widths.length; band++) widths[band] += widths[band - 1];
            for (int band = 0; band < widths.length; band++) widths[band] /= total;
            palettes[i] = new Palette(materials, widths);
        }
        return palettes;
    }

    private record Palette(BlockState[] materials, float[] ends) {
        BlockState at(int y, int height) {
            float alpha = (y % height) / (float) height;
            for (int i = 0; i < ends.length; i++) if (alpha < ends[i]) return materials[i];
            return materials[materials.length - 1];
        }
    }

    public record Config(boolean stoneForest) implements FeatureConfiguration {
        public static final Codec<Config> CODEC = RecordCodecBuilder.create(instance -> instance.group(
            Codec.BOOL.fieldOf("stone_forest").forGetter(Config::stoneForest)).apply(instance, Config::new));
    }
}
