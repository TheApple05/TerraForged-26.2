import java.util.List;
import java.util.concurrent.CountDownLatch;
import java.util.concurrent.Executors;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.atomic.AtomicInteger;
import raccoonman.reterraforged.concurrent.cache.Cache;
import raccoonman.reterraforged.concurrent.cache.CacheManager;
import raccoonman.reterraforged.concurrent.cache.ExpiringEntry;
import raccoonman.reterraforged.concurrent.cache.map.StampedLongMap;

public final class CacheLifecycleProbe {
    private static int assertions;
    private static void check(boolean condition, String message) {
        assertions++; if (!condition) throw new AssertionError(message);
    }
    private record Entry(AtomicInteger closed) implements ExpiringEntry {
        public long getTimestamp() { return System.currentTimeMillis(); }
        public void close() { closed.incrementAndGet(); }
    }
    public static void main(String[] args) throws Exception {
        try {
            int initial = CacheManager.activeCacheCount();
            int scheduled = Cache.SCHEDULER.getQueue().size();
            for (int cycle = 0; cycle < 500; cycle++) {
                Cache<Entry> cache = cycle % 2 == 0 ? CacheManager.createCache(8, 60, 20, TimeUnit.SECONDS)
                    : new Cache<>(8, 60, 20, TimeUnit.SECONDS, StampedLongMap::new);
                AtomicInteger valueClosed = new AtomicInteger();
                Entry heldByReader = cache.computeIfAbsent(1, key -> new Entry(valueClosed));
                check(CacheManager.activeCacheCount() == initial + 1, "Direct or factory cache not registered once");
                check(cache.size() == 1, "Entry not stored");
                cache.close(); cache.close();
                check(cache.isClosed() && cache.size() == 0, "Closed storage retained");
                check(CacheManager.activeCacheCount() == initial, "Manager retained closed cache");
                check(Cache.SCHEDULER.getQueue().size() == scheduled, "Cancelled scheduler task retained");
                check(valueClosed.get() == 0 && heldByReader.closed() == valueClosed, "In-flight value prematurely closed");
                boolean rejected = false;
                try { cache.computeIfAbsent(2, key -> new Entry(valueClosed)); } catch (IllegalStateException expected) { rejected = true; }
                check(rejected && cache.get(1) == null, "Closed cache resumed allocation");
            }
            Cache<Entry> unrelated = CacheManager.createCache(8, 60, 20, TimeUnit.SECONDS);
            var captured = CacheManager.capture(() -> {
                var a = CacheManager.<Entry>createCache(8, 60, 20, TimeUnit.SECONDS);
                var nested = CacheManager.capture(() -> new Cache<Entry>(8, 60, 20, TimeUnit.SECONDS, StampedLongMap::new));
                return List.of(a, nested.value());
            });
            check(captured.caches().size() == 2, "Nested/direct caches not captured by their context");
            captured.caches().forEach(Cache::close);
            check(CacheManager.activeCacheCount() == initial + 1 && !unrelated.isClosed(), "Selective close touched another context");
            int beforeFailure = CacheManager.activeCacheCount();
            try { CacheManager.capture(() -> { CacheManager.createCache(8, 60, 20, TimeUnit.SECONDS); throw new IllegalStateException("expected construction failure"); }); }
            catch (IllegalStateException expected) { check(expected.getMessage().contains("expected"), "Wrong construction failure"); }
            check(CacheManager.activeCacheCount() == beforeFailure, "Failed generator construction leaked a cache");
            unrelated.close();

            // Reproduce the disputed factory/close interleaving with the real
            // Stamped map: close must wait for insertion, then clear ownership.
            var racing = CacheManager.<Entry>createCache(8, 60, 20, TimeUnit.SECONDS);
            var factoryEntered = new CountDownLatch(1);
            var allowFactory = new CountDownLatch(1);
            var raceWorkers = Executors.newFixedThreadPool(2);
            try {
                var computed = raceWorkers.submit(() -> racing.computeIfAbsent(1, key -> {
                    factoryEntered.countDown();
                    try { if (!allowFactory.await(5, TimeUnit.SECONDS)) throw new AssertionError("Factory release timeout"); }
                    catch (InterruptedException failure) { throw new RuntimeException(failure); }
                    return new Entry(new AtomicInteger());
                }));
                check(factoryEntered.await(5, TimeUnit.SECONDS), "Factory was not entered");
                var closed = raceWorkers.submit(racing::close);
                long deadline = System.nanoTime() + TimeUnit.SECONDS.toNanos(5);
                while (!racing.isClosed() && System.nanoTime() < deadline) Thread.onSpinWait();
                check(racing.isClosed(), "Close did not publish its state");
                allowFactory.countDown();
                Entry held = computed.get(5, TimeUnit.SECONDS);
                closed.get(5, TimeUnit.SECONDS);
                check(racing.size() == 0, "Factory inserted into storage after close completed");
                check(held.closed().get() == 0, "Close invalidated the reader's completed value");
                check(CacheManager.activeCacheCount() == initial, "Factory/close race leaked an owner");
            } finally { allowFactory.countDown(); racing.close(); raceWorkers.shutdownNow(); }

            var executor = Executors.newFixedThreadPool(4);
            var begin = new CountDownLatch(1);
            try {
                var tasks = new java.util.ArrayList<java.util.concurrent.Future<?>>();
                for (int worker = 0; worker < 4; worker++) tasks.add(executor.submit(() -> {
                    try { begin.await(); for (int n = 0; n < 100; n++) {
                        var cache = CacheManager.createCache(8, 60, 20, TimeUnit.SECONDS);
                        if ((n & 1) == 0) CacheManager.clear();
                        cache.close();
                    }} catch (Exception failure) { throw new RuntimeException(failure); }
                }));
                begin.countDown();
                for (var task : tasks) task.get(10, TimeUnit.SECONDS);
            } finally { executor.shutdownNow(); }
            CacheManager.clear();
            check(CacheManager.activeCacheCount() == 0, "Concurrent create/clear left strong owners");
            check(Cache.SCHEDULER.getQueue().isEmpty(), "Concurrent close retained scheduled tasks");
            System.out.println("assertions=" + assertions + ",repeatedCycles=500,concurrentCycles=400,retainedOwners=0,queuedTasks=0");
        } finally { CacheManager.clear(); Cache.SCHEDULER.shutdownNow(); }
    }
}
