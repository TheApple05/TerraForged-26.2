package raccoonman.reterraforged.fabric.mixin;

import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.gen.Accessor;

/** Connects ordered datapack mutations to Fabric's native cache invalidation. */
@Mixin(targets = "net.fabricmc.fabric.impl.biome.modification.BiomeModificationContextImpl$GenerationSettingsContextImpl", remap = false)
public interface MixinGenerationSettingsContext {
    @Accessor("rebuildFeatures")
    void terraforged$setRebuildFeatures(boolean rebuildFeatures);
}
