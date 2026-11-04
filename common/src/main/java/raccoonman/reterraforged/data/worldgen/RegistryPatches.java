package raccoonman.reterraforged.data.worldgen;

import net.minecraft.core.Cloner;
import net.minecraft.core.HolderLookup;
import net.minecraft.core.RegistryAccess;
import net.minecraft.core.RegistrySetBuilder;
import net.minecraft.core.registries.BuiltInRegistries;
import net.minecraft.resources.RegistryDataLoader;
import raccoonman.reterraforged.data.worldgen.preset.settings.Preset;
import raccoonman.reterraforged.registries.RTFRegistries;
import raccoonman.reterraforged.world.worldgen.biome.modifier.BiomeModifier;
import raccoonman.reterraforged.world.worldgen.noise.module.Noise;
import raccoonman.reterraforged.world.worldgen.structure.rule.StructureRule;

/** Rebinds references in patched registries to the complete 26.2 registry view. */
public final class RegistryPatches {
    private RegistryPatches() {
    }

    public static HolderLookup.Provider build(RegistrySetBuilder builder, HolderLookup.Provider original) {
        Cloner.Factory cloners = new Cloner.Factory();
        RegistryDataLoader.WORLDGEN_REGISTRIES.forEach(data -> data.runWithArguments(cloners::addCodec));
        RegistryDataLoader.DIMENSION_REGISTRIES.forEach(data -> data.runWithArguments(cloners::addCodec));
        cloners.addCodec(RTFRegistries.PRESET, Preset.DIRECT_CODEC);
        cloners.addCodec(RTFRegistries.NOISE, Noise.DIRECT_CODEC);
        cloners.addCodec(RTFRegistries.BIOME_MODIFIER, BiomeModifier.CODEC);
        cloners.addCodec(RTFRegistries.STRUCTURE_RULE, StructureRule.DIRECT_CODEC);
        return builder.buildPatch(RegistryAccess.fromRegistryOfRegistries(BuiltInRegistries.REGISTRY), original, cloners).full();
    }
}
