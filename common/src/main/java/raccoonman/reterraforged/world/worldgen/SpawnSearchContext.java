package raccoonman.reterraforged.world.worldgen;

import java.util.Objects;
import java.util.function.Supplier;
import net.minecraft.core.BlockPos;
import org.jetbrains.annotations.Nullable;

/** Initial climate search configuration, scoped to the synchronous vanilla search. */
public final class SpawnSearchContext {
    private static final ThreadLocal<BlockPos> CENTER = new ThreadLocal<>();

    private SpawnSearchContext() {}

    public static <T> T withCenter(BlockPos center, Supplier<T> search) {
        BlockPos previous = CENTER.get();
        CENTER.set(Objects.requireNonNull(center).immutable());
        try {
            return search.get();
        } finally {
            if (previous == null) CENTER.remove();
            else CENTER.set(previous);
        }
    }

    @Nullable
    public static BlockPos getCenter() { return CENTER.get(); }

    public static long relativeX(long blockX) {
        BlockPos center = CENTER.get();
        return center == null ? blockX : blockX - center.getX();
    }

    public static long relativeZ(long blockZ) {
        BlockPos center = CENTER.get();
        return center == null ? blockZ : blockZ - center.getZ();
    }
}
