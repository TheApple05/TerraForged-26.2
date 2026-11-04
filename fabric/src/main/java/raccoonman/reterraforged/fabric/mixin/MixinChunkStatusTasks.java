package raccoonman.reterraforged.fabric.mixin;

import java.util.concurrent.CompletableFuture;

import net.minecraft.server.level.GenerationChunkHolder;
import net.minecraft.util.StaticCache2D;
import net.minecraft.world.level.ChunkPos;
import net.minecraft.world.level.chunk.ChunkAccess;
import net.minecraft.world.level.chunk.status.ChunkStatusTasks;
import net.minecraft.world.level.chunk.status.ChunkStep;
import net.minecraft.world.level.chunk.status.WorldGenContext;
import net.minecraft.world.level.levelgen.RandomState;
import org.jetbrains.annotations.Nullable;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Unique;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;
import raccoonman.reterraforged.world.worldgen.GeneratorContext;
import raccoonman.reterraforged.world.worldgen.RTFRandomState;
import raccoonman.reterraforged.world.worldgen.densityfunction.tile.TileCache;

@Mixin(ChunkStatusTasks.class)
public class MixinChunkStatusTasks {
    @Inject(method = "generateBiomes", at = @At("HEAD"))
    private static void reterraforged$queueTileBeforeBiomes(
            WorldGenContext worldGenContext, ChunkStep step,
            StaticCache2D<GenerationChunkHolder> chunks, ChunkAccess chunk,
            CallbackInfoReturnable<CompletableFuture<ChunkAccess>> callback) {
        TileCache cache = reterraforged$getTileCache(worldGenContext);
        if (cache != null) {
            ChunkPos pos = chunk.getPos();
            cache.queueAtChunk(pos.x(), pos.z());
        }
    }

    @Inject(method = "generateFeatures", at = @At("RETURN"), cancellable = true)
    private static void reterraforged$dropTileAfterFeaturesComplete(
            WorldGenContext worldGenContext, ChunkStep step,
            StaticCache2D<GenerationChunkHolder> chunks, ChunkAccess chunk,
            CallbackInfoReturnable<CompletableFuture<ChunkAccess>> callback) {
        TileCache cache = reterraforged$getTileCache(worldGenContext);
        if (cache != null) {
            ChunkPos pos = chunk.getPos();
            // TileCache counts completed chunks, not active per-stage references.
            // Decoration still consumes this tile, so drop only after its future.
            callback.setReturnValue(callback.getReturnValue().whenComplete((generatedChunk, failure) -> {
                cache.dropAtChunk(pos.x(), pos.z());
            }));
        }
    }

    @Nullable
    @Unique
    private static TileCache reterraforged$getTileCache(WorldGenContext context) {
        RandomState randomState = context.level().getChunkSource().randomState();
        if ((Object) randomState instanceof RTFRandomState rtfRandomState) {
            GeneratorContext generatorContext = rtfRandomState.generatorContext();
            if (generatorContext != null) {
                return generatorContext.cache;
            }
        }
        return null;
    }
}
