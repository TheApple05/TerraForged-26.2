import java.util.HashSet;
import java.util.Set;
import com.google.gson.JsonObject;
import com.mojang.serialization.JsonOps;
import net.minecraft.core.registries.Registries;
import net.minecraft.resources.Identifier;
import net.minecraft.resources.ResourceKey;
import net.minecraft.world.level.biome.Biome;
import net.minecraft.world.level.biome.Biomes;
import raccoonman.reterraforged.data.worldgen.preset.PresetBiomeData;
import raccoonman.reterraforged.world.worldgen.biome.ClassicBiomeSource;
import raccoonman.reterraforged.world.worldgen.biome.ClassicBiomes;
import raccoonman.reterraforged.world.worldgen.cell.Cell;
import raccoonman.reterraforged.world.worldgen.cell.terrain.Terrain;
import raccoonman.reterraforged.world.worldgen.cell.terrain.TerrainType;

public class ClassicBiomePolicyProbe {
    private static int assertions;
    private static final Set<ResourceKey<Biome>> covered = new HashSet<>();

    public static void main(String[] args) {
        net.minecraft.SharedConstants.tryDetectVersion();
        net.minecraft.server.Bootstrap.bootStrap();
        check(ClassicBiomes.VARIANTS.size() == 17, "inventory count");
        check(ClassicBiomes.VARIANTS.stream().map(ClassicBiomes.Variant::key).distinct().count() == 17, "unique inventory");
        select(Biomes.BADLANDS, TerrainType.BADLANDS, .5F, .1F, PresetBiomeData.BRYCE);
        select(Biomes.OLD_GROWTH_SPRUCE_TAIGA, TerrainType.FLATS, .3F, .1F, PresetBiomeData.COLD_STEPPE);
        select(Biomes.TAIGA, TerrainType.WETLAND, .3F, .1F, PresetBiomeData.COLD_MARSHLAND);
        select(Biomes.TAIGA, TerrainType.FLATS, .5F, .1F, PresetBiomeData.FIR_FOREST);
        select(Biomes.FLOWER_FOREST, TerrainType.FLATS, .5F, .1F, PresetBiomeData.FLOWER_PLAINS);
        select(Biomes.SNOWY_PLAINS, TerrainType.LAKE, .1F, .1F, PresetBiomeData.FROZEN_LAKE);
        select(Biomes.TAIGA, TerrainType.WETLAND, .1F, .1F, PresetBiomeData.FROZEN_MARSH);
        select(Biomes.PLAINS, TerrainType.LAKE, .5F, .1F, PresetBiomeData.LAKE);
        select(Biomes.PLAINS, TerrainType.WETLAND, .5F, .1F, PresetBiomeData.MARSHLAND);
        select(Biomes.SAVANNA, TerrainType.FLATS, .8F, .1F, PresetBiomeData.SAVANNA_SCRUB);
        select(Biomes.WINDSWEPT_SAVANNA, TerrainType.HILLS, .8F, .1F, PresetBiomeData.SHATTERED_SAVANNA_SCRUB);
        select(Biomes.SNOWY_TAIGA, TerrainType.FLATS, .1F, .1F, PresetBiomeData.SNOWY_FIR_FOREST);
        select(Biomes.SNOWY_TAIGA, TerrainType.FLATS, .1F, .3F, PresetBiomeData.SNOWY_TAIGA_SCRUB);
        select(Biomes.SAVANNA_PLATEAU, TerrainType.PLATEAU, .8F, .1F, PresetBiomeData.STEPPE);
        select(Biomes.JUNGLE, TerrainType.HILLS, .8F, .1F, PresetBiomeData.STONE_FOREST);
        select(Biomes.TAIGA, TerrainType.FLATS, .5F, .3F, PresetBiomeData.TAIGA_SCRUB);
        select(Biomes.BEACH, TerrainType.BEACH, .8F, .1F, PresetBiomeData.WARM_BEACH);
        check(covered.size() == 17, "all 17 synthetic policy paths");

        for (var untouched : Set.of(Biomes.LUSH_CAVES, Biomes.DRIPSTONE_CAVES, Biomes.DEEP_DARK, Biomes.MUSHROOM_FIELDS,
                Biomes.MANGROVE_SWAMP, Biomes.CHERRY_GROVE, Biomes.PALE_GARDEN, Biomes.FROZEN_PEAKS,
                Biomes.JAGGED_PEAKS, Biomes.STONY_PEAKS, Biomes.GROVE, Biomes.SNOWY_SLOPES, Biomes.WARM_OCEAN,
                ResourceKey.create(Registries.BIOME, Identifier.parse("biomesoplenty:overgrown_greens")),
                ResourceKey.create(Registries.BIOME, Identifier.parse("minecraft:future_surface_family")))) {
            for (var terrain : Set.of(TerrainType.LAKE, TerrainType.WETLAND, TerrainType.BEACH, TerrainType.FLATS)) {
                select(untouched, terrain, .1F, .1F, untouched);
            }
        }
        select(Biomes.PLAINS, TerrainType.LAKE, .25F, .1F, PresetBiomeData.LAKE);
        select(Biomes.PLAINS, TerrainType.WETLAND, .2F, .1F, PresetBiomeData.COLD_MARSHLAND);
        select(Biomes.PLAINS, TerrainType.WETLAND, .4F, .1F, PresetBiomeData.MARSHLAND);
        select(Biomes.BEACH, TerrainType.BEACH, .75F, .1F, Biomes.BEACH);
        bracket(Biomes.TAIGA, .25F, PresetBiomeData.FIR_FOREST, PresetBiomeData.TAIGA_SCRUB);
        bracket(Biomes.TAIGA, .55F, PresetBiomeData.TAIGA_SCRUB, Biomes.TAIGA);
        bracket(Biomes.BADLANDS, .35F, PresetBiomeData.BRYCE, Biomes.BADLANDS);
        bracket(Biomes.JUNGLE, 1F / 6F, PresetBiomeData.STONE_FOREST, Biomes.JUNGLE);
        for (float id : new float[] {0F, .1F, .25F, .5F, .75F, 1F, Float.MIN_VALUE, Math.nextDown(1F)}) {
            float choice = ClassicBiomeSource.regionChoice(id);
            check(choice >= 0F && choice < 1F, "finite hash domain");
            check(Float.floatToRawIntBits(choice) == Float.floatToRawIntBits(ClassicBiomeSource.regionChoice(Float.intBitsToFloat(Float.floatToRawIntBits(id)))), "raw-bit hash stability");
            Cell first = new Cell(); first.biomeRegionId = id; first.terrain = TerrainType.FLATS;
            Cell neighbor = new Cell(); neighbor.biomeRegionId = id; neighbor.terrain = TerrainType.FLATS;
            neighbor.height = .9F; neighbor.biomeRegionEdge = .01F; neighbor.continentId = .987F; neighbor.macroBiomeId = .876F;
            check(ClassicBiomeSource.selectKey(Biomes.TAIGA, first).equals(ClassicBiomeSource.selectKey(Biomes.TAIGA, neighbor)), "same raw region retains choice across non-selection field changes");
        }
        for (int[] vector : new int[][] {
            {0x00000000, 0x3e90425e},
            {0x3dcccccd, 0x3e03f480},
            {0x3e800000, 0x3f52f19e},
            {0x3f000000, 0x3f12af91},
            {0x3f400000, 0x3f06888f},
            {0x3f800000, 0x3ea8857a},
            {0x00000001, 0x3f7783c6},
            {0x3f7fffff, 0x3f49998f},
        }) {
            check(Float.floatToRawIntBits(ClassicBiomeSource.regionChoice(Float.intBitsToFloat(vector[0]))) == vector[1], "frozen region-choice hash bits");
        }
        for (float invalid : new float[] {Float.NaN, Float.POSITIVE_INFINITY, Float.NEGATIVE_INFINITY}) {
            Cell cell = new Cell(); cell.biomeRegionId = invalid; cell.terrain = TerrainType.LAKE;
            check(ClassicBiomeSource.selectKey(Biomes.PLAINS, cell).equals(Biomes.PLAINS), "nonfinite raw region retains delegate");
            cell.biomeRegionId = .5F; cell.regionTemperature = invalid;
            check(ClassicBiomeSource.selectKey(Biomes.PLAINS, cell).equals(Biomes.PLAINS), "nonfinite regional temperature retains delegate");
        }

        verifyNativeFeatureAliasOrdering();

        var current = new raccoonman.reterraforged.data.worldgen.preset.settings.MiscellaneousSettings(true, 600, true, true, true, false, true, true, true, true, true, .4F, .4F);
        check(current.classicBiomeVariants, "new current preset enables classic variants");
        JsonObject json = raccoonman.reterraforged.data.worldgen.preset.settings.MiscellaneousSettings.CODEC.encodeStart(JsonOps.INSTANCE, current).getOrThrow().getAsJsonObject();
        check(json.get("classicBiomeVariants").getAsBoolean(), "new option serialized");
        json.remove("classicBiomeVariants");
        var oldCurrent = raccoonman.reterraforged.data.worldgen.preset.settings.MiscellaneousSettings.CODEC.parse(JsonOps.INSTANCE, json).getOrThrow();
        check(!oldCurrent.classicBiomeVariants, "missing current field preserves old selection");
        check(!oldCurrent.copy().classicBiomeVariants, "current copy preserves false");
        var oldLegacy = raccoonman.reterraforged.data.worldgen.preset.MiscellaneousSettings.CODEC.parse(JsonOps.INSTANCE, json).getOrThrow();
        check(!oldLegacy.classicBiomeVariants, "missing legacy field preserves old selection");
        check(!oldLegacy.copy().classicBiomeVariants, "legacy copy preserves false");
        current.classicBiomeVariants = false;
        current.customBiomeFeatures = true;
        check(!current.copy().classicBiomeVariants && current.copy().customBiomeFeatures, "selection independent from decoration");
        System.out.println("CLASSIC_BIOME_POLICY_JSON={\"status\":\"pass\",\"assertions\":" + assertions + ",\"syntheticVariantsCovered\":17}");
    }

    private static void select(ResourceKey<Biome> original, Terrain terrain, float temperature, float region, ResourceKey<Biome> expected) {
        Cell cell = new Cell();
        cell.terrain = terrain;
        cell.regionTemperature = temperature;
        cell.biomeRegionId = rawIdForChoice(region);
        var result = ClassicBiomeSource.selectKey(original, cell);
        check(result.equals(expected), "selection " + original + " expected " + expected + " got " + result);
        if (result.identifier().getNamespace().equals("reterraforged")) covered.add(result);
    }

    private static void verifyNativeFeatureAliasOrdering() {
        net.minecraft.core.Holder<net.minecraft.world.level.levelgen.feature.ConfiguredFeature<?, ?>> configured = net.minecraft.core.Holder.direct(new net.minecraft.world.level.levelgen.feature.ConfiguredFeature<>(
            net.minecraft.world.level.levelgen.feature.Feature.NO_OP, net.minecraft.world.level.levelgen.feature.configurations.FeatureConfiguration.NONE));
        var base = new net.minecraft.world.level.levelgen.placement.PlacedFeature(configured, java.util.List.of(net.minecraft.world.level.levelgen.placement.CountPlacement.of(2)));
        var original = new net.minecraft.world.level.levelgen.placement.PlacedFeature(configured, java.util.List.of());
        var equalAlias = new net.minecraft.world.level.levelgen.placement.PlacedFeature(configured, java.util.List.of());
        var marked = new net.minecraft.world.level.levelgen.placement.PlacedFeature(configured, java.util.List.of(net.minecraft.world.level.levelgen.placement.CountPlacement.of(1)));
        check(original.equals(equalAlias), "different aliases with identical placement values collapse structurally");
        check(!original.equals(marked), "fixed count-one alias stays structurally distinct");
        java.util.List<net.minecraft.core.HolderSet<net.minecraft.world.level.levelgen.placement.PlacedFeature>> parent = java.util.List.of(net.minecraft.core.HolderSet.direct(net.minecraft.core.Holder.direct(base), net.minecraft.core.Holder.direct(original)));
        java.util.List<net.minecraft.core.HolderSet<net.minecraft.world.level.levelgen.placement.PlacedFeature>> broken = java.util.List.of(net.minecraft.core.HolderSet.direct(net.minecraft.core.Holder.direct(equalAlias), net.minecraft.core.Holder.direct(base)));
        boolean cycle = false;
        try {
            net.minecraft.world.level.biome.FeatureSorter.buildFeaturesPerStep(java.util.List.of(parent, broken), step -> step, true);
        } catch (IllegalStateException failure) {
            cycle = failure.getMessage().contains("Feature order cycle");
        }
        check(cycle, "native FeatureSorter reproduces the equal-alias order cycle");
        java.util.List<net.minecraft.core.HolderSet<net.minecraft.world.level.levelgen.placement.PlacedFeature>> fixed = java.util.List.of(net.minecraft.core.HolderSet.direct(net.minecraft.core.Holder.direct(marked), net.minecraft.core.Holder.direct(base)));
        var sorted = net.minecraft.world.level.biome.FeatureSorter.buildFeaturesPerStep(java.util.List.of(parent, fixed), step -> step, true);
        check(sorted.size() == 1 && sorted.getFirst().features().size() == 3, "native FeatureSorter accepts three distinct fixed vertices");
        check(sorted.getFirst().indexMapping().applyAsInt(marked) >= 0, "fixed alias has a native decoration index");
        for (long seed : new long[] {0, 1, -1, 42, 8675309, 123456789, Long.MIN_VALUE, Long.MAX_VALUE}) {
            var actual = net.minecraft.util.RandomSource.create(seed);
            var expected = net.minecraft.util.RandomSource.create(seed);
            var positions = net.minecraft.world.level.levelgen.placement.CountPlacement.of(1).getPositions(null, actual, net.minecraft.core.BlockPos.ZERO).toList();
            check(positions.equals(java.util.List.of(net.minecraft.core.BlockPos.ZERO)), "count-one alias preserves the position");
            check(actual.nextLong() == expected.nextLong(), "count-one alias consumes no random values");
        }
    }

    private static float rawIdForChoice(float wanted) {
        for (int i = 0; i < 100000; i++) {
            float id = Float.intBitsToFloat(0x3f000000 + i);
            if (Math.abs(ClassicBiomeSource.regionChoice(id) - wanted) < .001F) return id;
        }
        throw new AssertionError("No synthetic raw region near choice " + wanted);
    }

    private static void bracket(ResourceKey<Biome> original, float threshold, ResourceKey<Biome> below, ResourceKey<Biome> above) {
        float lower = 0, upper = 0, lowerGap = 1, upperGap = 1;
        for (int i = 0; i < 100000; i++) {
            float id = Float.intBitsToFloat(0x3f000000 + i);
            float gap = ClassicBiomeSource.regionChoice(id) - threshold;
            if (gap < 0 && -gap < lowerGap) { lowerGap = -gap; lower = id; }
            if (gap >= 0 && gap < upperGap) { upperGap = gap; upper = id; }
        }
        check(lowerGap < .0001F && upperGap < .0001F, "threshold bracket resolved in transformed domain");
        Cell cell = new Cell(); cell.terrain = TerrainType.FLATS;
        cell.biomeRegionId = lower;
        check(ClassicBiomeSource.selectKey(original, cell).equals(below), "transformed selector below boundary " + threshold);
        cell.biomeRegionId = upper;
        check(ClassicBiomeSource.selectKey(original, cell).equals(above), "transformed selector at/above boundary " + threshold);
    }

    private static void check(boolean condition, String message) {
        assertions++;
        if (!condition) throw new AssertionError(message);
    }
}
