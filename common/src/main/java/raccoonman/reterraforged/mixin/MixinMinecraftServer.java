package raccoonman.reterraforged.mixin;

import net.minecraft.core.BlockPos;
import net.minecraft.server.MinecraftServer;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.server.level.progress.LevelLoadListener;
import net.minecraft.world.level.biome.Climate;
import net.minecraft.world.level.levelgen.RandomState;
import net.minecraft.world.level.storage.ServerLevelData;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Redirect;
import raccoonman.reterraforged.world.worldgen.GeneratorContext;
import raccoonman.reterraforged.world.worldgen.RTFRandomState;
import raccoonman.reterraforged.world.worldgen.SpawnSearchContext;

@Mixin(MinecraftServer.class)
class MixinMinecraftServer {
    @Redirect(method = "setInitialSpawn", require = 1, at = @At(
        value = "INVOKE",
        target = "Lnet/minecraft/world/level/biome/Climate$Sampler;findSpawnPosition()Lnet/minecraft/core/BlockPos;"
    ))
    private static BlockPos reterraforged$findConfiguredSpawn(
            Climate.Sampler sampler, ServerLevel level, ServerLevelData levelData,
            boolean spawnBonusChest, boolean isDebug, LevelLoadListener levelLoadListener) {
        RandomState randomState = level.getChunkSource().randomState();
        if ((Object) randomState instanceof RTFRandomState rtfRandomState) {
            GeneratorContext context = rtfRandomState.generatorContext();
            if (context != null) {
                BlockPos center = context.preset.world().properties.spawnType.getSearchCenter(context);
                if (sampler.spawnTarget().isEmpty()) return center;
                return SpawnSearchContext.withCenter(center, sampler::findSpawnPosition);
            }
        }
        return sampler.findSpawnPosition();
    }
}
