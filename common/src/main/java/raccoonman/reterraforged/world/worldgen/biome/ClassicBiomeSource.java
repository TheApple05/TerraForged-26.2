package raccoonman.reterraforged.world.worldgen.biome;

import java.util.Map;
import java.util.stream.Collectors;
import java.util.stream.Stream;
import com.mojang.serialization.MapCodec;
import com.mojang.serialization.codecs.RecordCodecBuilder;
import net.minecraft.core.Holder;
import net.minecraft.core.HolderSet;
import net.minecraft.core.QuartPos;
import net.minecraft.resources.ResourceKey;
import net.minecraft.world.level.biome.Biome;
import net.minecraft.world.level.biome.BiomeSource;
import net.minecraft.world.level.biome.Biomes;
import net.minecraft.world.level.biome.Climate;
import raccoonman.reterraforged.concurrent.Resource;
import raccoonman.reterraforged.data.worldgen.preset.PresetBiomeData;
import raccoonman.reterraforged.world.worldgen.GeneratorContext;
import raccoonman.reterraforged.world.worldgen.cell.Cell;
import raccoonman.reterraforged.world.worldgen.cell.terrain.TerrainType;

/** Adds regional classic variants after the complete modern/modded resolver. */
public final class ClassicBiomeSource extends BiomeSource {
    public static final MapCodec<ClassicBiomeSource> CODEC = RecordCodecBuilder.mapCodec(instance -> instance.group(
        BiomeSource.CODEC.fieldOf("delegate").forGetter(source -> source.delegate),
        Biome.LIST_CODEC.fieldOf("variants").forGetter(source -> source.variants)
    ).apply(instance, ClassicBiomeSource::new));

    private final BiomeSource delegate;
    private final HolderSet<Biome> variants;
    private final Map<ResourceKey<Biome>, Holder<Biome>> byKey;

    public ClassicBiomeSource(BiomeSource delegate, HolderSet<Biome> variants) {
        this.delegate = delegate;
        this.variants = variants;
        this.byKey = variants.stream().collect(Collectors.toUnmodifiableMap(holder -> holder.unwrapKey().orElseThrow(), holder -> holder));
    }

    public BiomeSource delegate() {
        return this.delegate;
    }

    public ClassicBiomeSource withDelegate(BiomeSource replacement) {
        return new ClassicBiomeSource(replacement, this.variants);
    }

    @Override
    protected MapCodec<? extends BiomeSource> codec() {
        return CODEC;
    }

    @Override
    protected Stream<Holder<Biome>> collectPossibleBiomes() {
        return Stream.concat(this.delegate.possibleBiomes().stream(), this.variants.stream());
    }

    @Override
    public Holder<Biome> getNoiseBiome(int quartX, int quartY, int quartZ, Climate.Sampler sampler) {
        if (!((Object) sampler instanceof RTFClimateSampler climateSampler)) return this.delegate.getNoiseBiome(quartX, quartY, quartZ, sampler);
        GeneratorContext context = climateSampler.getGeneratorContext();
        if (context == null || !context.preset.miscellaneous().classicBiomeVariants) return this.delegate.getNoiseBiome(quartX, quartY, quartZ, sampler);
        try (Resource<Cell> resource = Cell.getResource()) {
            Cell cell = resource.get();
            // Resolve both the parent and variant from the completed filtered
            // terrain. A cold cache must not change a saved biome query.
            context.lookup.applyCell(cell, QuartPos.toBlock(quartX), QuartPos.toBlock(quartZ), true, true);
            Holder<Biome> original = raccoonman.reterraforged.world.worldgen.densityfunction.CellSampler.withAccurateSampling(
                () -> this.delegate.getNoiseBiome(quartX, quartY, quartZ, sampler));
            if (quartY < 0) return original;
            ResourceKey<Biome> originalKey = original.unwrapKey().orElse(null);
            if (originalKey == null || !eligible(originalKey)) return original;
            ResourceKey<Biome> replacement = selectKey(originalKey, cell);
            return this.byKey.getOrDefault(replacement, original);
        }
    }

    /** Deliberate 26.2 selection policy, not reconstructed unavailable Engine code. */
    public static ResourceKey<Biome> selectKey(ResourceKey<Biome> original, Cell cell) {
        if (!eligible(original) || !Float.isFinite(cell.biomeRegionId) || !Float.isFinite(cell.regionTemperature)) return original;
        float region = regionChoice(cell.biomeRegionId);
        float temperature = cell.regionTemperature;
        if (cell.terrain == TerrainType.LAKE) return temperature < .25F ? PresetBiomeData.FROZEN_LAKE : PresetBiomeData.LAKE;
        if (cell.terrain.isWetland()) {
            if (temperature < .2F) return PresetBiomeData.FROZEN_MARSH;
            if (temperature < .4F) return PresetBiomeData.COLD_MARSHLAND;
            return PresetBiomeData.MARSHLAND;
        }
        if (cell.terrain.getCategory() == raccoonman.reterraforged.world.worldgen.cell.terrain.TerrainCategory.BEACH && temperature > .75F) return PresetBiomeData.WARM_BEACH;
        if (original.equals(Biomes.BADLANDS) || original.equals(Biomes.ERODED_BADLANDS)) return region < .35F ? PresetBiomeData.BRYCE : original;
        if (original.equals(Biomes.OLD_GROWTH_SPRUCE_TAIGA)) return region < .35F ? PresetBiomeData.COLD_STEPPE : original;
        if (original.equals(Biomes.TAIGA)) {
            if (region < .25F) return PresetBiomeData.FIR_FOREST;
            if (region < .55F) return PresetBiomeData.TAIGA_SCRUB;
        }
        if (original.equals(Biomes.SNOWY_TAIGA)) {
            if (region < .25F) return PresetBiomeData.SNOWY_FIR_FOREST;
            if (region < .55F) return PresetBiomeData.SNOWY_TAIGA_SCRUB;
        }
        if (original.equals(Biomes.FLOWER_FOREST) && region < .35F) return PresetBiomeData.FLOWER_PLAINS;
        if (original.equals(Biomes.SWAMP) && region < .35F) return PresetBiomeData.MARSHLAND;
        if (original.equals(Biomes.SAVANNA) && region < .35F) return PresetBiomeData.SAVANNA_SCRUB;
        if (original.equals(Biomes.SAVANNA_PLATEAU) && region < .35F) return PresetBiomeData.STEPPE;
        if (original.equals(Biomes.WINDSWEPT_SAVANNA) && region < .35F) return PresetBiomeData.SHATTERED_SAVANNA_SCRUB;
        if (original.equals(Biomes.JUNGLE) && region < 1F / 6F) return PresetBiomeData.STONE_FOREST;
        return original;
    }

    /** A separate deterministic selector domain; the raw ID already controls climate interpolation. */
    public static float regionChoice(float regionId) {
        if (!Float.isFinite(regionId)) return 1F;
        int hash = Float.floatToRawIntBits(regionId) ^ 0x632BE59B;
        hash ^= hash >>> 16;
        hash *= 0x7feb352d;
        hash ^= hash >>> 15;
        hash *= 0x846ca68b;
        hash ^= hash >>> 16;
        return (hash >>> 8) * 0x1.0p-24F;
    }

    private static final java.util.Set<ResourceKey<Biome>> ELIGIBLE = java.util.Set.of(Biomes.BADLANDS, Biomes.ERODED_BADLANDS, Biomes.OLD_GROWTH_SPRUCE_TAIGA,
            Biomes.TAIGA, Biomes.SNOWY_TAIGA, Biomes.FLOWER_FOREST, Biomes.SWAMP, Biomes.SAVANNA,
            Biomes.SAVANNA_PLATEAU, Biomes.WINDSWEPT_SAVANNA, Biomes.JUNGLE, Biomes.PLAINS,
            Biomes.SUNFLOWER_PLAINS, Biomes.SNOWY_PLAINS, Biomes.BEACH, Biomes.SNOWY_BEACH,
            Biomes.RIVER, Biomes.FROZEN_RIVER);

    private static boolean eligible(ResourceKey<Biome> key) {
        return ELIGIBLE.contains(key);
    }
}
