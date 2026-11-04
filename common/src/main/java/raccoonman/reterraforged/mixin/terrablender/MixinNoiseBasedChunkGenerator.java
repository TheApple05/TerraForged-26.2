package raccoonman.reterraforged.mixin.terrablender;

import com.mojang.datafixers.util.Either;
import net.minecraft.core.Holder;
import net.minecraft.world.level.biome.Biome;
import net.minecraft.world.level.biome.BiomeResolver;
import net.minecraft.world.level.biome.MultiNoiseBiomeSource;
import net.minecraft.world.level.levelgen.NoiseBasedChunkGenerator;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.ModifyArg;
import raccoonman.reterraforged.world.worldgen.biome.ClassicBiomeSource;
import terrablender.mixin.MultiNoiseBiomeSourceAccess;
import terrablender.worldgen.IExtendedMultiNoiseBiomeSource;
import terrablender.worldgen.IExtendedParameterList;

/** Preserve native TerraBlender per-chunk uniqueness isolation inside the classic wrapper. */
@Mixin(NoiseBasedChunkGenerator.class)
class MixinNoiseBasedChunkGenerator {
    @ModifyArg(method = "doCreateBiomes", index = 0, require = 1, at = @At(value = "INVOKE",
        target = "Lnet/minecraft/world/level/chunk/ChunkAccess;fillBiomesFromNoise(Lnet/minecraft/world/level/biome/BiomeResolver;Lnet/minecraft/world/level/biome/Climate$Sampler;)V"))
    private BiomeResolver reterraforged$forkDelegate(BiomeResolver resolver) {
        if (!(resolver instanceof ClassicBiomeSource classic)
                || !(classic.delegate() instanceof MultiNoiseBiomeSource multi)) return resolver;
        var parameters = (IExtendedParameterList<Holder<Biome>>) ((MultiNoiseBiomeSourceInvoker) multi).reterraforged$parameters();
        if (!parameters.isInitialized()) return resolver;
        var isolated = parameters.clone();
        ((IExtendedParameterList<Holder<Biome>>) isolated).recreateUniqueness();
        var source = ((IExtendedMultiNoiseBiomeSource) multi).clone();
        ((MultiNoiseBiomeSourceAccess) source).setParameters(Either.left(isolated));
        return classic.withDelegate(source);
    }
}
