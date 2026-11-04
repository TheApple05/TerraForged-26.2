import java.util.ArrayList;
import java.util.List;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.CountDownLatch;
import java.util.concurrent.ExecutorService;
import java.util.concurrent.Executors;
import java.util.concurrent.Future;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.TimeoutException;
import java.util.concurrent.atomic.AtomicBoolean;
import java.util.concurrent.atomic.AtomicInteger;
import java.util.concurrent.atomic.AtomicReference;
import raccoonman.reterraforged.concurrent.cache.CacheEntry;
import raccoonman.reterraforged.concurrent.cache.SafeCloseable;

/** Exercises the production cache implementation with ordinary JDK futures. */
public final class CacheEntryProbe {
    private static int assertions;

    private static void check(boolean condition, String message) {
        assertions++;
        if (!condition) throw new AssertionError(message);
    }

    private static ExecutorService workers(int count) {
        return Executors.newFixedThreadPool(count, runnable -> {
            Thread thread = new Thread(runnable, "cache-regression");
            thread.setDaemon(true); // A deadlocked old implementation must not strand the test JVM.
            return thread;
        });
    }

    private static void reentry(boolean baseline) throws Exception {
        Object expected = new Object();
        AtomicReference<CacheEntry<Object>> reference = new AtomicReference<>();
        AtomicBoolean nested = new AtomicBoolean();
        AtomicInteger reads = new AtomicInteger();
        CompletableFuture<Object> task = new CompletableFuture<>() {
            @Override
            public Object get() throws java.util.concurrent.ExecutionException, InterruptedException {
                reads.incrementAndGet();
                if (nested.compareAndSet(false, true)) {
                    // Future waits may execute other ready work (as observed in
                    // the actual ForkJoin stack). Its result is already ready
                    // when that work recursively requests this cache entry.
                    complete(expected);
                    return reference.get().get();
                }
                return super.get();
            }
        };
        CacheEntry<Object> entry = CacheEntry.supply(task);
        reference.set(entry);
        ExecutorService executor = workers(1);
        try {
            Future<Object> read = executor.submit(() -> { return entry.get(); });
            try {
                Object actual = read.get(2, TimeUnit.SECONDS);
                if (baseline) throw new AssertionError("Pinned baseline did not reproduce the observed reentry deadlock");
                check(actual == expected, "Reentry returned the wrong object");
                check(reads.get() == 2, "The nested future read was not exercised");
                check(entry.get() == expected && reads.get() == 2, "Published value was not reused");
            } catch (TimeoutException timeout) {
                if (!baseline) throw new AssertionError("Cache entry deadlocked during future reentry", timeout);
                check(nested.get(), "Baseline timed out before reentry");
            }
        } finally {
            executor.shutdownNow();
        }
    }

    private static void parallelReaders() throws Exception {
        int count = 16;
        ExecutorService executor = workers(count);
        try {
            for (int iteration = 0; iteration < 16; iteration++) {
                CountDownLatch started = new CountDownLatch(count);
                CompletableFuture<Object> task = new CompletableFuture<>();
                CacheEntry<Object> entry = CacheEntry.supply(task);
                Object expected = new Object();
                List<Future<Object>> readers = new ArrayList<>();
                for (int i = 0; i < count; i++) {
                    readers.add(executor.submit(() -> {
                        started.countDown();
                        return entry.get();
                    }));
                }
                check(started.await(2, TimeUnit.SECONDS), "Readers did not start");
                check(!entry.isDone(), "Incomplete task reported done");
                task.complete(expected);
                for (Future<Object> reader : readers) {
                    check(reader.get(2, TimeUnit.SECONDS) == expected, "Waiters did not share the future result");
                }
                check(entry.call() == expected && entry.isDone(), "Completed value was not published");
            }
        } finally {
            executor.shutdownNow();
        }
    }

    private static void failuresAndClose() {
        IllegalStateException cause = new IllegalStateException("sentinel future failure");
        try {
            CacheEntry.supply(CompletableFuture.failedFuture(cause)).get();
            throw new AssertionError("Failed future became a success");
        } catch (RuntimeException failure) {
            Throwable root = failure;
            while (root.getCause() != null) root = root.getCause();
            check(root == cause, "Future failure cause was lost");
        }
        AtomicInteger closes = new AtomicInteger();
        SafeCloseable closeable = closes::incrementAndGet;
        CacheEntry<SafeCloseable> entry = CacheEntry.supply(CompletableFuture.completedFuture(closeable));
        check(entry.get() == closeable, "Resource result was changed");
        check(entry.getTimestamp() > 0, "Timestamp refresh was lost");
        entry.close();
        check(closes.get() == 1, "Cached resource was not closed");
        try {
            CacheEntry.supply(CompletableFuture.completedFuture(null)).get();
            throw new AssertionError("Unexpected null future result was accepted");
        } catch (NullPointerException expected) {
            check(true, "Non-null invariant");
        }
    }

    public static void main(String[] args) throws Exception {
        boolean baseline = args.length > 0 && args[0].equals("baseline");
        reentry(baseline);
        if (!baseline) {
            parallelReaders();
            failuresAndClose();
        }
        System.out.println("CACHE_QA_JSON={\"status\":\"" + (baseline ? "expected-reentry-deadlock" : "pass")
            + "\",\"assertions\":" + assertions + ",\"parallel_readers\":" + (baseline ? 0 : 256) + "}");
    }
}
