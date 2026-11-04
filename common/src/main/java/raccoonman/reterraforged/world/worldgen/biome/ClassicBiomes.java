/* Modern adaptation of TerraForged BiomeBuilders (c) 2021 TerraForged, MIT. */
package raccoonman.reterraforged.world.worldgen.biome;

import java.util.List;
import java.util.Set;
import net.minecraft.core.Holder;
import net.minecraft.core.HolderGetter;
import net.minecraft.core.HolderLookup;
import net.minecraft.core.registries.Registries;
import net.minecraft.data.worldgen.BootstrapContext;
import net.minecraft.data.worldgen.placement.VegetationPlacements;
import net.minecraft.resources.ResourceKey;
import net.minecraft.world.level.biome.Biome;
import net.minecraft.world.level.biome.BiomeGenerationSettings;
import net.minecraft.world.level.biome.Biomes;
import net.minecraft.world.level.levelgen.GenerationStep;
import net.minecraft.world.level.levelgen.feature.Feature;
import net.minecraft.world.level.levelgen.placement.PlacedFeature;
import raccoonman.reterraforged.data.worldgen.preset.PresetBiomeData;
import raccoonman.reterraforged.data.worldgen.preset.PresetPlacedFeatures;

/** Explicit modern parent mappings; no old biome-manager/Engine thresholds are inferred. */
public final class ClassicBiomes {
    public record Variant(ResourceKey<Biome> key, ResourceKey<Biome> parent, ResourceKey<Biome> tagParent,
                          boolean removeTrees, boolean removeRocks, boolean removeMushrooms,
                          boolean deadBush, boolean denseGrass, boolean ferns, Float temperature, Float downfall) {
    }

    public static final List<Variant> VARIANTS = List.of(
        variant(PresetBiomeData.BRYCE, Biomes.BADLANDS, true, false, false, true, false, false, null, null),
        variant(PresetBiomeData.COLD_STEPPE, Biomes.OLD_GROWTH_SPRUCE_TAIGA, true, true, true, true, true, true, .2F, .025F),
        variant(PresetBiomeData.COLD_MARSHLAND, Biomes.SNOWY_TAIGA, true, false, false, true, true, true, .2F, .5F),
        variant(PresetBiomeData.FIR_FOREST, Biomes.TAIGA, false, false, false, true, true, true, null, null),
        variant(PresetBiomeData.FLOWER_PLAINS, Biomes.FLOWER_FOREST, false, false, false, false, false, false, null, null),
        variant(PresetBiomeData.FROZEN_LAKE, Biomes.FROZEN_RIVER, true, false, false, false, false, false, null, null),
        variant(PresetBiomeData.FROZEN_MARSH, Biomes.SNOWY_TAIGA, true, false, false, false, false, false, .14999F, .15F),
        variant(PresetBiomeData.LAKE, Biomes.RIVER, true, false, false, false, false, false, null, null),
        variant(PresetBiomeData.MARSHLAND, Biomes.SWAMP, true, false, false, true, true, true, .7F, null),
        variant(PresetBiomeData.SAVANNA_SCRUB, Biomes.SAVANNA, true, false, false, true, true, false, null, null),
        variant(PresetBiomeData.SHATTERED_SAVANNA_SCRUB, Biomes.WINDSWEPT_SAVANNA, false, false, false, true, true, false, null, null),
        variant(PresetBiomeData.SNOWY_FIR_FOREST, Biomes.SNOWY_TAIGA, false, false, false, false, false, true, null, null),
        variant(PresetBiomeData.SNOWY_TAIGA_SCRUB, Biomes.SNOWY_TAIGA, true, false, false, false, false, true, null, null),
        new Variant(PresetBiomeData.STEPPE, Biomes.OLD_GROWTH_SPRUCE_TAIGA, Biomes.WINDSWEPT_SAVANNA, true, true, true, true, true, false, 1.2F, .05F),
        variant(PresetBiomeData.STONE_FOREST, Biomes.JUNGLE, false, false, false, false, false, false, null, null),
        variant(PresetBiomeData.TAIGA_SCRUB, Biomes.TAIGA, true, false, true, true, true, true, null, null),
        new Variant(PresetBiomeData.WARM_BEACH, Biomes.WARM_OCEAN, Biomes.BEACH, true, false, false, false, false, false, 1F, null)
    );

    private ClassicBiomes() {
    }

    private static Variant variant(ResourceKey<Biome> key, ResourceKey<Biome> parent, boolean trees,
            boolean rocks, boolean mushrooms, boolean bushes, boolean grass, boolean ferns, Float temperature, Float downfall) {
        return new Variant(key, parent, parent, trees, rocks, mushrooms, bushes, grass, ferns, temperature, downfall);
    }

    public static void bootstrap(BootstrapContext<Biome> ctx, HolderLookup.Provider original) {
        HolderGetter<Biome> biomes = original.lookupOrThrow(Registries.BIOME);
        HolderGetter<PlacedFeature> features = ctx.lookup(Registries.PLACED_FEATURE);
        for (Variant variant : VARIANTS) {
            Biome parent = biomes.getOrThrow(variant.parent()).value();
            BiomeGenerationSettings.PlainBuilder generation = new BiomeGenerationSettings.PlainBuilder();
            parent.getGenerationSettings().getCarvers().forEach(generation::addCarver);
            for (int step = 0; step < parent.getGenerationSettings().features().size(); step++) {
                for (Holder<PlacedFeature> feature : parent.getGenerationSettings().features().get(step)) {
                    if (!remove(variant, feature)) generation.addFeature(step, feature);
                }
            }
            // Rebind all inherited nested references through the patch context,
            // including inline features contributed to a vanilla parent by a pack.
            var encodedGeneration = BiomeGenerationSettings.CODEC.codec().encodeStart(original.createSerializationContext(com.mojang.serialization.JsonOps.INSTANCE), generation.build()).getOrThrow();
            var rebound = BiomeGenerationSettings.CODEC.codec().parse(raccoonman.reterraforged.data.worldgen.BootstrapContextOps.forDecoding(ctx), encodedGeneration).getOrThrow();
            BiomeGenerationSettings.PlainBuilder finalGeneration = new BiomeGenerationSettings.PlainBuilder();
            rebound.getCarvers().forEach(finalGeneration::addCarver);
            for (int step = 0; step < rebound.features().size(); step++) {
                for (var feature : rebound.features().get(step)) finalGeneration.addFeature(step, feature);
            }
            if (variant.deadBush()) addAlias(finalGeneration, features, DEAD_BUSH);
            if (variant.denseGrass()) {
                addAlias(finalGeneration, features, GRASS_PLAIN);
                addAlias(finalGeneration, features, GRASS_NORMAL);
                addAlias(finalGeneration, features, GRASS_SAVANNA);
            }
            if (variant.ferns()) addAlias(finalGeneration, features, FERNS);
            if (variant.key().equals(PresetBiomeData.BRYCE)) {
                finalGeneration.addFeature(GenerationStep.Decoration.RAW_GENERATION, features.getOrThrow(PresetPlacedFeatures.BRYCE_SURFACE));
            } else if (variant.key().equals(PresetBiomeData.STONE_FOREST)) {
                finalGeneration.addFeature(GenerationStep.Decoration.RAW_GENERATION, features.getOrThrow(PresetPlacedFeatures.STONE_FOREST_SURFACE));
            }
            if (variant.key().equals(PresetBiomeData.MARSHLAND)) {
                finalGeneration.addFeature(GenerationStep.Decoration.RAW_GENERATION, features.getOrThrow(MARSH_SURFACE));
            }
            Biome ambience = variant.key().equals(PresetBiomeData.STEPPE) ? biomes.getOrThrow(Biomes.SAVANNA).value() : parent;
            ctx.register(variant.key(), new Biome.BiomeBuilder()
                .hasPrecipitation(!variant.key().equals(PresetBiomeData.STEPPE) && parent.hasPrecipitation())
                .temperature(variant.temperature() == null ? parent.getBaseTemperature() : variant.temperature())
                .downfall(variant.downfall() == null ? downfall(parent, original) : variant.downfall())
                .putAttributes(ambience.getAttributes()).specialEffects(ambience.getSpecialEffects())
                .mobSpawnSettings(parent.getMobSettings()).generationSettings(finalGeneration.build()).build());
        }
    }

    private static boolean remove(Variant variant, Holder<PlacedFeature> placed) {
        // These are applied from the current preset's modifier registry. Do not
        // retain a previous editor application and append them a second time.
        if (placed.is(PresetPlacedFeatures.ERODE) || placed.is(PresetPlacedFeatures.DECORATE_SNOW)
                || placed.is(PresetPlacedFeatures.SWAMP_SURFACE)) return true;
        if (variant.removeTrees() && placed.value().getFeatures().anyMatch(feature -> isTree(feature.value()))) return true;
        if (variant.removeRocks() && placed.unwrapKey().map(key -> key.identifier().getPath().equals("forest_rock")).orElse(false)) return true;
        return variant.removeMushrooms() && placed.unwrapKey().map(key -> Set.of("brown_mushroom_taiga", "red_mushroom_taiga", "brown_mushroom_old_growth", "red_mushroom_old_growth").contains(key.identifier().getPath())).orElse(false);
    }

    public static boolean isTree(net.minecraft.world.level.levelgen.feature.ConfiguredFeature<?, ?> feature) {
        return feature.feature() == Feature.TREE || (feature.config() instanceof raccoonman.reterraforged.world.worldgen.feature.template.TemplateFeature.Config<?> template
            && !template.templates().isEmpty() && template.templates().stream().allMatch(id -> id.getPath().startsWith("structures/trees/")));
    }

    public static final ResourceKey<PlacedFeature> MARSH_SURFACE = featureKey("classic_marsh_surface");
    public static final ResourceKey<PlacedFeature> DEAD_BUSH = featureKey("classic_dead_bush");
    public static final ResourceKey<PlacedFeature> GRASS_PLAIN = featureKey("classic_grass_plain");
    public static final ResourceKey<PlacedFeature> GRASS_NORMAL = featureKey("classic_grass_normal");
    public static final ResourceKey<PlacedFeature> GRASS_SAVANNA = featureKey("classic_grass_savanna");
    public static final ResourceKey<PlacedFeature> FERNS = featureKey("classic_ferns");

    public static void bootstrapPlacedFeatures(BootstrapContext<PlacedFeature> ctx, HolderLookup.Provider original) {
        ctx.register(MARSH_SURFACE, new PlacedFeature(ctx.lookup(Registries.CONFIGURED_FEATURE).getOrThrow(raccoonman.reterraforged.data.worldgen.preset.PresetConfiguredFeatures.SWAMP_SURFACE), List.of(net.minecraft.world.level.levelgen.placement.CountPlacement.of(1))));
        var aliases = java.util.Map.of(DEAD_BUSH, VegetationPlacements.PATCH_DEAD_BUSH_2,
            GRASS_PLAIN, VegetationPlacements.PATCH_GRASS_PLAIN, GRASS_NORMAL, VegetationPlacements.PATCH_GRASS_NORMAL,
            GRASS_SAVANNA, VegetationPlacements.PATCH_GRASS_SAVANNA, FERNS, VegetationPlacements.PATCH_LARGE_FERN);
        aliases.forEach((target, source) -> {
            var encoded = PlacedFeature.DIRECT_CODEC.encodeStart(original.createSerializationContext(com.mojang.serialization.JsonOps.INSTANCE), original.lookupOrThrow(Registries.PLACED_FEATURE).getOrThrow(source).value()).getOrThrow();
            var rebound = PlacedFeature.DIRECT_CODEC.parse(raccoonman.reterraforged.data.worldgen.BootstrapContextOps.forDecoding(ctx), encoded).getOrThrow();
            var placement = new java.util.ArrayList<>(rebound.placement());
            // PlacedFeature is a record and FeatureSorter keys by equals, so a
            // new registry ID alone does not distinguish an exact alias. Fixed
            // count 1 changes equality while preserving positions and RNG use.
            placement.add(net.minecraft.world.level.levelgen.placement.CountPlacement.of(1));
            ctx.register(target, new PlacedFeature(rebound.feature(), List.copyOf(placement)));
        });
    }

    private static ResourceKey<PlacedFeature> featureKey(String name) {
        return ResourceKey.create(Registries.PLACED_FEATURE, raccoonman.reterraforged.RTFCommon.location(name));
    }

    private static void addAlias(BiomeGenerationSettings.PlainBuilder builder, HolderGetter<PlacedFeature> features, ResourceKey<PlacedFeature> key) {
        // Registered aliases keep every step a homogeneous reference-holder list
        // and avoid contradictory ordering for shared vanilla placed features.
        builder.addFeature(GenerationStep.Decoration.VEGETAL_DECORATION, features.getOrThrow(key));
    }

    private static float downfall(Biome biome, HolderLookup.Provider original) {
        // The public API no longer exposes downfall; preserve the exact climate
        // codec value, rather than reverse-engineering it from grass colour.
        return Biome.DIRECT_CODEC.encodeStart(original.createSerializationContext(com.mojang.serialization.JsonOps.INSTANCE), biome).getOrThrow().getAsJsonObject().get("downfall").getAsFloat();
    }
}
