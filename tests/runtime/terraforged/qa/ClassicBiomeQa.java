package terraforged.qa;

import java.util.ArrayList;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.Set;
import java.util.stream.Collectors;
import com.mojang.serialization.JsonOps;
import net.minecraft.core.Holder;
import net.minecraft.core.QuartPos;
import net.minecraft.core.registries.Registries;
import net.minecraft.server.MinecraftServer;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.world.level.biome.Biome;
import net.minecraft.world.level.biome.BiomeSource;
import net.minecraft.world.level.levelgen.feature.Feature;
import raccoonman.reterraforged.world.worldgen.RTFRandomState;
import raccoonman.reterraforged.world.worldgen.biome.ClassicBiomeSource;
import raccoonman.reterraforged.world.worldgen.biome.ClassicBiomes;
import raccoonman.reterraforged.world.worldgen.cell.Cell;

/** Registry/codec/actual-sampler evidence, kept outside the production packages. */
public final class ClassicBiomeQa {
    private ClassicBiomeQa() {
    }

    public static Map<String, Object> verify(MinecraftServer server, ServerLevel level) {
        Map<String, Object> result = new LinkedHashMap<>();
        var randomState = level.getChunkSource().randomState();
        var state = (RTFRandomState) (Object) randomState;
        if (state.generatorContext() == null) {
            result.put("activeTerraForgedContext", false);
            return result;
        }
        var context = state.generatorContext();
        var source = level.getChunkSource().getGenerator().getBiomeSource();
        boolean enabled = context.preset.miscellaneous().classicBiomeVariants;
        result.put("activeTerraForgedContext", true);
        result.put("classicBiomeVariants", enabled);
        result.put("sourceClass", source.getClass().getName());
        if (enabled != (source instanceof ClassicBiomeSource)) throw new IllegalStateException("Classic biome source/setting disagree");
        var registry = level.registryAccess().lookupOrThrow(Registries.BIOME);
        List<Map<String, Object>> definitions = new ArrayList<>();
        for (var variant : ClassicBiomes.VARIANTS) {
            Holder<Biome> holder = registry.getOrThrow(variant.key());
            Map<String, Object> definition = new LinkedHashMap<>();
            definition.put("id", variant.key().identifier().toString());
            definition.put("parent", variant.parent().identifier().toString());
            definition.put("possible", source.possibleBiomes().contains(holder));
            definition.put("temperature", holder.value().getBaseTemperature());
            definition.put("hasPrecipitation", holder.value().hasPrecipitation());
            long trees = holder.value().getGenerationSettings().features().stream().flatMap(set -> set.stream())
                .filter(feature -> feature.value().getFeatures().anyMatch(configured -> ClassicBiomes.isTree(configured.value()))).count();
            definition.put("recognizedTreeFeatures", trees);
            definition.put("tags", holder.tags().map(tag -> tag.location().toString()).sorted().toList());
            var parentTags = registry.getOrThrow(variant.tagParent()).tags().collect(Collectors.toSet());
            var actualTags = holder.tags().collect(Collectors.toSet());
            var missingTags = parentTags.stream().filter(tag -> !actualTags.contains(tag)).map(tag -> tag.location().toString()).sorted().toList();
            definition.put("missingInheritedParentTags", missingTags);
            if (!missingTags.isEmpty()) throw new IllegalStateException("Missing inherited biome tags for " + variant.key() + ": " + missingTags);
            if (!holder.is(net.minecraft.tags.TagKey.create(Registries.BIOME, net.minecraft.resources.Identifier.parse("reterraforged:classic_variants")))) throw new IllegalStateException("Missing classic variant tag " + variant.key());
            if (variant.removeTrees() && trees != 0) throw new IllegalStateException("Unexpected parent trees in " + variant.key());
            if (enabled && !source.possibleBiomes().contains(holder)) throw new IllegalStateException("Missing possible biome " + variant.key());
            definitions.add(definition);
        }
        result.put("definitions", definitions);
        result.put("registeredVariants", definitions.size());
        var placedFeatures = level.registryAccess().lookupOrThrow(Registries.PLACED_FEATURE);
        var aliases = Map.of(ClassicBiomes.DEAD_BUSH, net.minecraft.data.worldgen.placement.VegetationPlacements.PATCH_DEAD_BUSH_2,
            ClassicBiomes.GRASS_PLAIN, net.minecraft.data.worldgen.placement.VegetationPlacements.PATCH_GRASS_PLAIN,
            ClassicBiomes.GRASS_NORMAL, net.minecraft.data.worldgen.placement.VegetationPlacements.PATCH_GRASS_NORMAL,
            ClassicBiomes.GRASS_SAVANNA, net.minecraft.data.worldgen.placement.VegetationPlacements.PATCH_GRASS_SAVANNA,
            ClassicBiomes.FERNS, net.minecraft.data.worldgen.placement.VegetationPlacements.PATCH_LARGE_FERN,
            ClassicBiomes.MARSH_SURFACE, raccoonman.reterraforged.data.worldgen.preset.PresetPlacedFeatures.SWAMP_SURFACE);
        for (var alias : aliases.entrySet()) {
            if (placedFeatures.getOrThrow(alias.getKey()).value().equals(placedFeatures.getOrThrow(alias.getValue()).value())) throw new IllegalStateException("FeatureSorter would collapse classic placed alias " + alias.getKey());
        }
        result.put("structurallyDistinctPlacedAliases", aliases.size());
        var ops = level.registryAccess().createSerializationContext(JsonOps.INSTANCE);
        var json = BiomeSource.CODEC.encodeStart(ops, source).getOrThrow();
        BiomeSource decoded = BiomeSource.CODEC.parse(ops, json).getOrThrow();
        boolean codecEqual = json.equals(BiomeSource.CODEC.encodeStart(ops, decoded).getOrThrow());
        if (raccoonman.reterraforged.world.worldgen.terrablender.TBCompat.isEnabled()) {
            if (!(level.getChunkSource().getGenerator() instanceof net.minecraft.world.level.levelgen.NoiseBasedChunkGenerator generator)) throw new IllegalStateException("Expected native noise generator for TerraBlender codec lifecycle QA");
            int beforeNativeInit = decoded.possibleBiomes().size();
            var temporary = new net.minecraft.world.level.levelgen.NoiseBasedChunkGenerator(decoded, generator.generatorSettings());
            try {
                var initialize = Class.forName("terrablender.util.LevelUtils").getMethod("initializeBiomes", net.minecraft.core.RegistryAccess.class,
                    net.minecraft.core.Holder.class, net.minecraft.resources.ResourceKey.class, net.minecraft.world.level.chunk.ChunkGenerator.class, long.class);
                initialize.invoke(null, level.registryAccess(), level.dimensionTypeRegistration(), Registries.levelToLevelStem(level.dimension()), temporary, level.getSeed());
            } catch (ReflectiveOperationException failure) {
                throw new IllegalStateException("Decoded biome source native TerraBlender initialization failed", failure);
            }
            decoded = temporary.getBiomeSource();
            result.put("decodedSourceNativeTerraBlenderInitialization", Map.of("method", "LevelUtils.initializeBiomes", "dimension", level.dimension().identifier().toString(),
                "seed", level.getSeed(), "possibleBiomesBefore", beforeNativeInit, "possibleBiomesAfter", decoded.possibleBiomes().size(),
                "scope", "Decode source, verify serialized roundtrip, then run the real native initialization on a temporary generator before comparing holder sets"));
        }
        if (!codecEqual || !decoded.possibleBiomes().equals(source.possibleBiomes())) throw new IllegalStateException("Classic biome source codec roundtrip/lifecycle failed: original=" + source.possibleBiomes().size() + ", decoded=" + decoded.possibleBiomes().size());
        result.put("sourceCodecRoundTrip", true);
        result.put("possibleBiomes", source.possibleBiomes().size());
        if (!enabled || !Boolean.getBoolean("terraforged.qa.classicBiomeSearch")) return result;

        int limit = Integer.getInteger("terraforged.qa.classicBiomeSearchSamples", 65536);
        int extent = Integer.getInteger("terraforged.qa.classicBiomeSearchExtent", 131072);
        Set<String> wanted = ClassicBiomes.VARIANTS.stream().map(v -> v.key().identifier().toString()).collect(Collectors.toSet());
        Map<String, Object> found = new LinkedHashMap<>();
        Map<String, Integer> observed = new LinkedHashMap<>();
        Cell cell = new Cell();
        int checked = 0;
        long started = System.nanoTime();
        for (int index = 0; index < limit && found.size() < wanted.size(); index++) {
            // Deterministic low-discrepancy regional survey. Direct heightmap and
            // real source/sampler calls; no fabricated cell or forced biome IDs.
            int x = Math.floorDiv((int) ((radicalInverse(index + 1, 2) * 2 - 1) * extent), 4) * 4;
            int z = Math.floorDiv((int) ((radicalInverse(index + 1, 3) * 2 - 1) * extent), 4) * 4;
            context.lookup.getHeightmap().apply(cell.reset(), x, z, true);
            int y = Math.max(context.preset.world().properties.seaLevel, context.levels.scale(cell.height));
            Holder<Biome> biome = source.getNoiseBiome(QuartPos.fromBlock(x), QuartPos.fromBlock(y), QuartPos.fromBlock(z), randomState.sampler());
            String id = biome.unwrapKey().orElseThrow().identifier().toString();
            observed.merge(id, 1, Integer::sum);
            checked++;
            if (wanted.contains(id) && !found.containsKey(id)) {
                var repeated = decoded.getNoiseBiome(QuartPos.fromBlock(x), QuartPos.fromBlock(y), QuartPos.fromBlock(z), randomState.sampler());
                if (!biome.equals(repeated)) throw new IllegalStateException("Decoded source changed actual sample");
                found.put(id, Map.of("x", x, "y", y, "z", z, "terrain", cell.terrain.getName(), "temperature", cell.regionTemperature,
                    "moisture", cell.regionMoisture, "region", cell.biomeRegionId));
            }
        }
        List<String> missing = wanted.stream().filter(id -> !found.containsKey(id)).sorted().toList();
        result.put("surveySamples", checked);
        result.put("surveyExtent", extent);
        result.put("surveyMilliseconds", (System.nanoTime() - started) / 1_000_000.0);
        result.put("actualSamplerLocations", found);
        result.put("actualObservedDistribution", observed);
        result.put("missingInSurvey", missing);
        result.put("all17Reached", missing.isEmpty());
        try {
            var output = java.nio.file.Path.of(System.getProperty("terraforged.qa.output", "qa-output"));
            java.nio.file.Files.createDirectories(output);
            java.nio.file.Files.writeString(output.resolve("classic-biome-survey.json"), new com.google.gson.GsonBuilder().setPrettyPrinting().create().toJson(result) + "\n");
        } catch (java.io.IOException failure) {
            throw new IllegalStateException("Cannot retain classic biome survey", failure);
        }
        // In a biome-mod stack vanilla climate parents can intentionally be
        // replaced. Record that scope instead of forcing over its holders.
        if (Boolean.getBoolean("terraforged.qa.requireAllClassicBiomes") && !missing.isEmpty()) throw new IllegalStateException("Classic variants not reached: " + missing);
        return result;
    }

    private static double radicalInverse(int value, int base) {
        double valueInverse = 0;
        double factor = 1.0 / base;
        while (value != 0) {
            valueInverse += (value % base) * factor;
            value /= base;
            factor /= base;
        }
        return valueInverse;
    }
}
