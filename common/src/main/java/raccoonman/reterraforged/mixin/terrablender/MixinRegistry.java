package raccoonman.reterraforged.mixin.terrablender;

import com.mojang.serialization.Codec;
import com.mojang.serialization.MapCodec;
import net.minecraft.core.Registry;
import net.minecraft.core.registries.Registries;
import net.minecraft.resources.Identifier;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.ModifyVariable;
import terrablender.worldgen.surface.NamespacedSurfaceRuleSource;

/** Corrects the pinned TerraBlender release's Codec/MapCodec registration mismatch. */
@Mixin(value = Registry.class, priority = 1100)
interface MixinRegistry {
    // Target the original public static overload, independent of TerraBlender's
    // merged synthetic lambda name. Slot 2 is the erased Object value argument.
    @ModifyVariable(
        method = "register(Lnet/minecraft/core/Registry;Lnet/minecraft/resources/Identifier;Ljava/lang/Object;)Ljava/lang/Object;",
        at = @At("HEAD"), argsOnly = true, index = 2, remap = false, require = 1
    )
    private static Object reterraforged$correctMergedSurfaceCodec(
            Object value, Registry<?> registry, Identifier id, Object originalValue) {
        if (registry.key().equals(Registries.MATERIAL_RULE)
                && id.getNamespace().equals("terrablender") && id.getPath().equals("merged")
                && value instanceof Codec<?> && !(value instanceof MapCodec<?>)) {
            // 26.2 stores MapCodec identities in MATERIAL_RULE. The affected
            // release passes CODEC.codec(), a distinct full Codec wrapper.
            return NamespacedSurfaceRuleSource.CODEC;
        }
        return value;
    }
}
