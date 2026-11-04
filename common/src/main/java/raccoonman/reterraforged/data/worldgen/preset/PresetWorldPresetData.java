package raccoonman.reterraforged.data.worldgen.preset;

import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import com.mojang.serialization.JsonOps;
import net.minecraft.core.HolderLookup;
import net.minecraft.core.registries.Registries;
import net.minecraft.data.worldgen.BootstrapContext;
import net.minecraft.world.level.levelgen.NoiseBasedChunkGenerator;
import net.minecraft.world.level.levelgen.NoiseGeneratorSettings;
import net.minecraft.world.level.levelgen.presets.WorldPreset;
import net.minecraft.world.level.dimension.LevelStem;
import raccoonman.reterraforged.data.worldgen.BootstrapContextOps;
import raccoonman.reterraforged.data.worldgen.preset.settings.Preset;
import raccoonman.reterraforged.world.worldgen.biome.ClassicBiomeSource;
import raccoonman.reterraforged.world.worldgen.biome.ClassicBiomes;

/** Retains every dimension/delegate field, including data supplied by other mods. */
public final class PresetWorldPresetData {
    private PresetWorldPresetData() {
    }

    public static void bootstrap(Preset preset, BootstrapContext<WorldPreset> ctx, HolderLookup.Provider original) {
        original.lookupOrThrow(Registries.WORLD_PRESET).listElements().forEach(holder -> {
            WorldPreset worldPreset = holder.value();
            if (worldPreset.overworld().isEmpty()
                    || !(worldPreset.overworld().get().generator() instanceof NoiseBasedChunkGenerator generator)
                    || !generator.generatorSettings().is(NoiseGeneratorSettings.OVERWORLD)) return;
            JsonElement encoded = WorldPreset.DIRECT_CODEC.encodeStart(original.createSerializationContext(JsonOps.INSTANCE), worldPreset).getOrThrow();
            JsonObject generatorJson = encoded.getAsJsonObject().getAsJsonObject("dimensions").getAsJsonObject("minecraft:overworld").getAsJsonObject("generator");
            updateSource(generatorJson, preset.miscellaneous().classicBiomeVariants);
            ctx.register(holder.key(), WorldPreset.DIRECT_CODEC.parse(BootstrapContextOps.forDecoding(ctx), encoded).getOrThrow());
        });
    }

    /** Dimension resources take precedence over world presets in WorldDimensions.bake. */
    public static void bootstrapDimensions(Preset preset, BootstrapContext<LevelStem> ctx, HolderLookup.Provider original) {
        original.lookup(Registries.LEVEL_STEM).ifPresent(dimensions -> dimensions.listElements().forEach(holder -> {
            if (!holder.is(LevelStem.OVERWORLD)
                    || !(holder.value().generator() instanceof NoiseBasedChunkGenerator generator)
                    || !generator.generatorSettings().is(NoiseGeneratorSettings.OVERWORLD)) return;
            JsonElement encoded = LevelStem.CODEC.encodeStart(original.createSerializationContext(JsonOps.INSTANCE), holder.value()).getOrThrow();
            updateSource(encoded.getAsJsonObject().getAsJsonObject("generator"), preset.miscellaneous().classicBiomeVariants);
            ctx.register(holder.key(), LevelStem.CODEC.parse(BootstrapContextOps.forDecoding(ctx), encoded).getOrThrow());
        }));
    }

    private static void updateSource(JsonObject generator, boolean enabled) {
        JsonElement source = generator.get("biome_source");
        while (source.isJsonObject() && source.getAsJsonObject().has("type")
                && source.getAsJsonObject().get("type").getAsString().equals("reterraforged:classic_variants")) {
            source = source.getAsJsonObject().get("delegate");
        }
        if (!enabled) {
            generator.add("biome_source", source);
            return;
        }
        JsonObject wrapper = new JsonObject();
        wrapper.addProperty("type", "reterraforged:classic_variants");
        wrapper.add("delegate", source);
        JsonArray variants = new JsonArray();
        ClassicBiomes.VARIANTS.forEach(variant -> variants.add(variant.key().identifier().toString()));
        wrapper.add("variants", variants);
        generator.add("biome_source", wrapper);
    }
}
