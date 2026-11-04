import com.google.gson.Gson;
import java.util.ArrayList;
import java.util.List;
import java.util.concurrent.CountDownLatch;
import java.util.concurrent.Executors;
import java.util.concurrent.Future;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.atomic.AtomicInteger;
import net.minecraft.SharedConstants;
import net.minecraft.core.BlockPos;
import net.minecraft.server.Bootstrap;
import raccoonman.reterraforged.world.worldgen.SpawnSearchContext;

public final class SpawnScopeProbe {
    private static final AtomicInteger ASSERTIONS = new AtomicInteger();

    private static void check(boolean condition, String message) {
        ASSERTIONS.incrementAndGet();
        if (!condition) throw new AssertionError(message);
    }

    public static void main(String[] args) throws Exception {
        SharedConstants.tryDetectVersion();
        Bootstrap.bootStrap();
        check(SharedConstants.getCurrentVersion().id().equals("26.2"), "Wrong game version");
        check(SpawnSearchContext.getCenter() == null, "Unscoped search has no configured center");
        check(SpawnSearchContext.relativeX(-17) == -17 && SpawnSearchContext.relativeZ(16) == 16,
              "Unscoped searches retain vanilla distance inputs");

        BlockPos.MutableBlockPos mutable = new BlockPos.MutableBlockPos(12345, 0, -67890);
        int result = SpawnSearchContext.withCenter(mutable, () -> {
            mutable.set(9, 9, 9);
            check(SpawnSearchContext.getCenter().equals(new BlockPos(12345, 0, -67890)),
                  "Scope must snapshot a mutable input position");
            check(SpawnSearchContext.relativeX(12345 + 17) == 17, "X distance must be relative to chosen center");
            check(SpawnSearchContext.relativeZ(-67890 - 16) == -16, "Z distance must preserve sign");
            RuntimeException sentinel = new IllegalStateException("intentional nested search failure");
            try {
                SpawnSearchContext.withCenter(new BlockPos(-1, 0, 1), () -> {
                    check(SpawnSearchContext.relativeX(-17) == -16, "Nested X center");
                    check(SpawnSearchContext.relativeZ(17) == 16, "Nested Z center");
                    throw sentinel;
                });
                throw new AssertionError("Expected nested exception");
            } catch (RuntimeException failure) {
                check(failure == sentinel, "Original failure must propagate unchanged");
            }
            check(SpawnSearchContext.getCenter().equals(new BlockPos(12345, 0, -67890)),
                  "Exception must restore enclosing scope");
            return 42;
        });
        check(result == 42 && SpawnSearchContext.getCenter() == null, "Successful outer scope must clean up and retain result");

        RuntimeException outerFailure = new IllegalArgumentException("intentional outer failure");
        try {
            SpawnSearchContext.withCenter(BlockPos.ZERO, () -> { throw outerFailure; });
        } catch (RuntimeException failure) {
            check(failure == outerFailure, "Outer exception identity");
        }
        check(SpawnSearchContext.getCenter() == null, "Failed outer scope must remove thread-local state");
        SpawnSearchContext.withCenter(new BlockPos(Integer.MAX_VALUE, 0, Integer.MIN_VALUE), () -> {
            check(SpawnSearchContext.relativeX(Integer.MIN_VALUE) == -4294967295L, "X subtraction must not overflow int");
            check(SpawnSearchContext.relativeZ(Integer.MAX_VALUE) == 4294967295L, "Z subtraction must not overflow int");
            return null;
        });

        try (var executor = Executors.newFixedThreadPool(4)) {
            CountDownLatch entered = new CountDownLatch(4);
            List<Future<?>> futures = new ArrayList<>();
            for (int i = 0; i < 4; i++) {
                final int worker = i;
                futures.add(executor.submit(() -> {
                    check(SpawnSearchContext.getCenter() == null, "Fresh worker starts unscoped");
                    SpawnSearchContext.withCenter(new BlockPos(worker * 1000, 0, worker * -1000), () -> {
                        entered.countDown();
                        try {
                            check(entered.await(10, TimeUnit.SECONDS), "Concurrent scope barrier timed out");
                        } catch (InterruptedException e) {
                            Thread.currentThread().interrupt();
                            throw new AssertionError(e);
                        }
                        check(SpawnSearchContext.relativeX(worker * 1000 + 3) == 3,
                              "Concurrent X centers leaked between threads");
                        check(SpawnSearchContext.relativeZ(worker * -1000 - 4) == -4,
                              "Concurrent Z centers leaked between threads");
                        return null;
                    });
                    check(SpawnSearchContext.getCenter() == null, "Worker scope must clear before reuse");
                }));
            }
            for (Future<?> future : futures) future.get(15, TimeUnit.SECONDS);
        }
        check(SpawnSearchContext.getCenter() == null, "Worker centers must not leak to caller");
        var report = java.util.Map.of(
            "status", "pass", "assertions", ASSERTIONS.get(), "minecraft", SharedConstants.getCurrentVersion().id(),
            "scenarios", List.of("vanilla_unscoped_distance", "immutable_center_snapshot", "nested_exception_restore",
                                 "outer_success_and_failure_cleanup", "long_coordinate_subtraction", "concurrent_thread_isolation"),
            "scope", "Actual SpawnSearchContext helper with real Minecraft BlockPos; does not apply or validate runtime mixins");
        Bootstrap.realStdoutPrintln("SPAWN_QA_JSON=" + new Gson().toJson(report));
        Bootstrap.shutdownStdout();
    }
}
