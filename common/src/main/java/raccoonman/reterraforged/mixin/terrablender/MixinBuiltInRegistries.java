package raccoonman.reterraforged.mixin.terrablender;

import com.mojang.serialization.Codec;
import com.mojang.serialization.MapCodec;
import net.minecraft.core.Registry;
import net.minecraft.core.registries.BuiltInRegistries;
import net.minecraft.core.registries.Registries;
import net.minecraft.resources.Identifier;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.ModifyArgs;
import org.spongepowered.asm.mixin.injection.invoke.arg.Args;
import terrablender.worldgen.surface.NamespacedSurfaceRuleSource;

/** Corrects the pinned TerraBlender release's Codec/MapCodec registration mismatch. */
@Mixin(value = BuiltInRegistries.class, priority = 1100)
class MixinBuiltInRegistries {
    // TerraBlender contributes a synthetic lambda while merging its mixin. Select
    // the exact invocation across merged methods, then guard all three arguments.
    // The priority must exceed TerraBlender's 1000 to inject into its merged code.
    @ModifyArgs(method = "*", remap = false, require = 1, at = @At(
        value = "INVOKE",
        target = "Lnet/minecraft/core/Registry;register(Lnet/minecraft/core/Registry;Lnet/minecraft/resources/Identifier;Ljava/lang/Object;)Ljava/lang/Object;",
        remap = false
    ))
    private static void reterraforged$correctMergedSurfaceCodec(Args arguments) {
        Object value = arguments.get(2);
        if (arguments.get(0) instanceof Registry<?> registry
                && registry.key().equals(Registries.MATERIAL_RULE)
                && arguments.get(1) instanceof Identifier id
                && id.getNamespace().equals("terrablender") && id.getPath().equals("merged")
                && value instanceof Codec<?> && !(value instanceof MapCodec<?>)) {
            // 26.2 stores MapCodec identities in MATERIAL_RULE. The affected
            // release passes CODEC.codec(), a distinct full Codec wrapper.
            arguments.set(2, NamespacedSurfaceRuleSource.CODEC);
        }
    }
}
