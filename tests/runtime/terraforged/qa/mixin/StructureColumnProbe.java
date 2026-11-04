package terraforged.qa.mixin;

import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Unique;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;
import net.minecraft.world.level.LevelHeightAccessor;
import net.minecraft.world.level.NoiseColumn;
import net.minecraft.world.level.levelgen.NoiseBasedChunkGenerator;
import net.minecraft.world.level.levelgen.RandomState;

/** Optional exact-coordinate telemetry; contained only in the separate QA JAR. */
@Mixin(NoiseBasedChunkGenerator.class)
public abstract class StructureColumnProbe {
    @Unique private static final java.util.concurrent.atomic.AtomicLong terraforged$qa$columnIds = new java.util.concurrent.atomic.AtomicLong();
    @Unique private static final ThreadLocal<long[]> terraforged$qa$column = new ThreadLocal<>();

    @Inject(method = "getBaseColumn", at = @At("HEAD"), require = 1)
    private void terraforged$qa$columnStart(int x, int z, LevelHeightAccessor heights, RandomState state, CallbackInfoReturnable<NoiseColumn> callback) {
        if (!Boolean.getBoolean("terraforged.qa.profileRuinedPortal")) return;
        terraforged$qa$column.remove();
        boolean portal = StackWalker.getInstance().walk(frames -> frames.anyMatch(frame ->
            frame.getClassName().equals("net.minecraft.world.level.levelgen.structure.structures.RuinedPortalStructure")
                && frame.getMethodName().equals("findSuitableY")));
        if (!portal) return;
        long id = terraforged$qa$columnIds.incrementAndGet();
        terraforged$qa$column.set(new long[] {id, x, z, System.nanoTime()});
        System.out.println("TERRAFORGED_RUINED_PORTAL_COLUMN_START_JSON={\"id\":" + id + ",\"x\":" + x + ",\"z\":" + z + "}");
    }

    @Inject(method = "getBaseColumn", at = @At("RETURN"), require = 1)
    private void terraforged$qa$columnEnd(int x, int z, LevelHeightAccessor heights, RandomState state, CallbackInfoReturnable<NoiseColumn> callback) {
        if (!Boolean.getBoolean("terraforged.qa.profileRuinedPortal")) return;
        long[] measurement = terraforged$qa$column.get();
        if (measurement == null) return;
        terraforged$qa$column.remove();
        System.out.println("TERRAFORGED_RUINED_PORTAL_COLUMN_END_JSON={\"id\":" + measurement[0] + ",\"x\":" + measurement[1]
            + ",\"z\":" + measurement[2] + ",\"elapsedNanos\":" + (System.nanoTime() - measurement[3]) + "}");
    }
}
