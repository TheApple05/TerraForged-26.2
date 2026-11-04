package raccoonman.reterraforged.mixin;

import java.util.function.Predicate;
import com.mojang.datafixers.util.Pair;
import net.minecraft.core.BlockPos;
import net.minecraft.core.Holder;
import net.minecraft.util.RandomSource;
import net.minecraft.world.level.biome.Biome;
import net.minecraft.world.level.biome.BiomeSource;
import net.minecraft.world.level.biome.Climate;
import net.minecraft.world.level.chunk.ChunkGeneratorStructureState;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Redirect;
import raccoonman.reterraforged.world.worldgen.biome.ClassicBiomeSource;

/** Ring refinement is a coarse parent-biome heuristic, not a request to generate 320 remote regions. */
@Mixin(ChunkGeneratorStructureState.class)
class MixinChunkGeneratorStructureState {
    @Redirect(method = "lambda$generateRingPositions$0", require = 1, at = @At(value = "INVOKE", target =
        "Lnet/minecraft/world/level/biome/BiomeSource;findBiomeHorizontal(IIIILjava/util/function/Predicate;Lnet/minecraft/util/RandomSource;Lnet/minecraft/world/level/biome/Climate$Sampler;)Lcom/mojang/datafixers/util/Pair;"))
    private Pair<BlockPos, Holder<Biome>> reterraforged$searchParentForRing(BiomeSource source, int x, int y, int z,
            int radius, Predicate<Holder<Biome>> predicate, RandomSource random, Climate.Sampler sampler) {
        while (source instanceof ClassicBiomeSource classic) source = classic.delegate();
        return source.findBiomeHorizontal(x, y, z, radius, predicate, random, sampler);
    }
}
