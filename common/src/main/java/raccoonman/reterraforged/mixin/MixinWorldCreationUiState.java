package raccoonman.reterraforged.mixin;

import net.minecraft.client.gui.screens.worldselection.WorldCreationContext;
import net.minecraft.client.gui.screens.worldselection.WorldCreationUiState;
import net.minecraft.core.HolderSet;
import net.minecraft.core.registries.Registries;
import net.minecraft.world.level.levelgen.NoiseBasedChunkGenerator;
import net.minecraft.world.level.levelgen.NoiseGeneratorSettings;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.ModifyVariable;
import raccoonman.reterraforged.data.worldgen.preset.settings.Preset;
import raccoonman.reterraforged.registries.RTFRegistries;
import raccoonman.reterraforged.world.worldgen.biome.ClassicBiomeSource;
import raccoonman.reterraforged.world.worldgen.biome.ClassicBiomes;

/** Vanilla keeps customized dimensions through datapack reload; update their biome source too. */
@Mixin(WorldCreationUiState.class)
class MixinWorldCreationUiState {
    @ModifyVariable(method = "setSettings", at = @At("HEAD"), argsOnly = true, ordinal = 0, require = 1)
    private WorldCreationContext reterraforged$applyClassicSelection(WorldCreationContext settings) {
        var registries = settings.worldgenLoadContext();
        var preset = registries.lookup(RTFRegistries.PRESET).flatMap(lookup -> lookup.get(Preset.KEY));
        if (preset.isEmpty() || !(settings.selectedDimensions().overworld() instanceof NoiseBasedChunkGenerator generator)
                || !generator.generatorSettings().is(NoiseGeneratorSettings.OVERWORLD)) return settings;
        var source = generator.getBiomeSource();
        while (source instanceof ClassicBiomeSource classic) source = classic.delegate();
        if (preset.get().value().miscellaneous().classicBiomeVariants) {
            var biomes = registries.lookupOrThrow(Registries.BIOME);
            source = new ClassicBiomeSource(source, HolderSet.direct(ClassicBiomes.VARIANTS.stream()
                .map(variant -> biomes.getOrThrow(variant.key())).toList()));
        }
        if (source == generator.getBiomeSource()) return settings;
        var replacement = new NoiseBasedChunkGenerator(source, generator.generatorSettings());
        return settings.withDimensions((lookup, dimensions) -> dimensions.replaceOverworldGenerator(lookup, replacement));
    }
}
