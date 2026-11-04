package raccoonman.reterraforged.world.worldgen.biome.modifier.fabric;

import net.fabricmc.fabric.api.biome.v1.BiomeModificationContext;
import net.fabricmc.fabric.api.biome.v1.BiomeSelectionContext;
import raccoonman.reterraforged.fabric.mixin.MixinGenerationSettingsContext;
import raccoonman.reterraforged.world.worldgen.biome.modifier.BiomeModifier;

public interface FabricBiomeModifier extends BiomeModifier {
    void apply(BiomeSelectionContext selectionContext, BiomeModificationContext modificationContext);

    default void markFeaturesChanged(BiomeModificationContext modificationContext) {
        // Preserve custom ordering while using Fabric's native freeze/rebuild lifecycle.
        // It invalidates biome caches and already-validated chunk-generator feature sorters.
        ((MixinGenerationSettingsContext) modificationContext.getGenerationSettings())
            .terraforged$setRebuildFeatures(true);
    }
}
