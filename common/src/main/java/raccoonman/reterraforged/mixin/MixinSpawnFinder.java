package raccoonman.reterraforged.mixin;

import net.minecraft.core.BlockPos;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.ModifyArg;
import org.spongepowered.asm.mixin.injection.ModifyArgs;
import org.spongepowered.asm.mixin.injection.invoke.arg.Args;
import raccoonman.reterraforged.world.worldgen.SpawnSearchContext;

@Mixin(targets = "net.minecraft.world.level.biome.Climate$SpawnFinder")
class MixinSpawnFinder {
    @ModifyArgs(method = "<init>", require = 1, at = @At(
        value = "INVOKE",
        target = "Lnet/minecraft/world/level/biome/Climate$SpawnFinder;getSpawnPositionAndFitness(Ljava/util/List;Lnet/minecraft/world/level/biome/Climate$Sampler;II)Lnet/minecraft/world/level/biome/Climate$SpawnFinder$Result;"
    ))
    private void reterraforged$setInitialSearchCenter(Args args) {
        BlockPos center = SpawnSearchContext.getCenter();
        if (center != null) {
            args.set(2, center.getX());
            args.set(3, center.getZ());
        }
    }

    @ModifyArg(method = "getSpawnPositionAndFitness", index = 0, require = 1, at = @At(
        value = "INVOKE", target = "Lnet/minecraft/util/Mth;square(J)J", ordinal = 0
    ))
    private static long reterraforged$distanceFromCenterX(long blockX) {
        return SpawnSearchContext.relativeX(blockX);
    }

    @ModifyArg(method = "getSpawnPositionAndFitness", index = 0, require = 1, at = @At(
        value = "INVOKE", target = "Lnet/minecraft/util/Mth;square(J)J", ordinal = 1
    ))
    private static long reterraforged$distanceFromCenterZ(long blockZ) {
        return SpawnSearchContext.relativeZ(blockZ);
    }
}
