package raccoonman.reterraforged.mixin.terrablender;

import java.util.ArrayList;
import net.minecraft.core.Holder;
import net.minecraft.core.RegistryAccess;
import net.minecraft.resources.ResourceKey;
import net.minecraft.world.level.biome.BiomeSource;
import net.minecraft.world.level.biome.MultiNoiseBiomeSource;
import net.minecraft.world.level.chunk.ChunkGenerator;
import net.minecraft.world.level.dimension.DimensionType;
import net.minecraft.world.level.dimension.LevelStem;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.ModifyVariable;
import org.spongepowered.asm.mixin.injection.Redirect;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;
import raccoonman.reterraforged.world.worldgen.biome.ClassicBiomeSource;
import terrablender.util.LevelUtils;
import terrablender.worldgen.IExtendedBiomeSource;
import terrablender.worldgen.IExtendedParameterList;

/** Let TerraBlender initialize its real delegate and publish the same biome set on our wrapper. */
@Mixin(value = LevelUtils.class, remap = false)
class MixinLevelUtils {
    @ModifyVariable(method = "shouldApplyToBiomeSource", at = @At("HEAD"), argsOnly = true, ordinal = 0, require = 1)
    private static BiomeSource reterraforged$testDelegate(BiomeSource source) {
        return reterraforged$unwrap(source);
    }

    @Redirect(method = "initializeBiomes", at = @At(value = "INVOKE",
        target = "Lnet/minecraft/world/level/chunk/ChunkGenerator;getBiomeSource()Lnet/minecraft/world/level/biome/BiomeSource;"), require = 1)
    private static BiomeSource reterraforged$initializeDelegate(ChunkGenerator generator) {
        return reterraforged$unwrap(generator.getBiomeSource());
    }

    @Inject(method = "initializeBiomes", at = @At("TAIL"), require = 1)
    private static void reterraforged$publishDelegateBiomes(RegistryAccess registries, Holder<DimensionType> type,
            ResourceKey<LevelStem> key, ChunkGenerator generator, long seed, CallbackInfo callback) {
        if (generator.getBiomeSource() instanceof ClassicBiomeSource classic
                && classic.delegate() instanceof MultiNoiseBiomeSource multi
                && ((IExtendedParameterList<?>) ((MultiNoiseBiomeSourceInvoker) multi).reterraforged$parameters()).isInitialized()) {
            // The wrapper may already have memoized its initial set during validation.
            // Use TerraBlender's own publication API, preserving all existing holders.
            ((IExtendedBiomeSource) (Object) classic).appendDeferredBiomesList(new ArrayList<>(multi.possibleBiomes()));
        }
    }

    private static BiomeSource reterraforged$unwrap(BiomeSource source) {
        while (source instanceof ClassicBiomeSource classic) source = classic.delegate();
        return source;
    }
}
