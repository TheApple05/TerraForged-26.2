package raccoonman.reterraforged.concurrent.cache;

import java.util.Collections;
import java.util.LinkedList;
import java.util.List;
import java.util.concurrent.TimeUnit;
import java.util.function.IntFunction;
import java.util.function.Supplier;
import java.util.ArrayList;

import raccoonman.reterraforged.concurrent.cache.map.LongMap;
import raccoonman.reterraforged.concurrent.cache.map.StampedBoundLongMap;

@Deprecated
public class CacheManager {
	private static final List<Cache<?>> CACHES = Collections.synchronizedList(new LinkedList<>());
    private static final ThreadLocal<List<Cache<?>>> CAPTURE = new ThreadLocal<>();
	
    public static <V extends ExpiringEntry> Cache<V> createCache(long expireTime, long pollInterval, TimeUnit unit) {
    	return createCache(256, expireTime, pollInterval, unit);
    }
	
	public static <V extends ExpiringEntry> Cache<V> createCache(int capacity, long expireTime, long pollInterval, TimeUnit unit) {
		return createCache(capacity, expireTime, pollInterval, unit, StampedBoundLongMap::new);
	}
	
	public static <V extends ExpiringEntry> Cache<V> createCache(int capacity, long expireTime, long pollInterval, TimeUnit unit, IntFunction<LongMap<V>> mapFunc) {
		Cache<V> cache = new Cache<>(capacity, expireTime, pollInterval, unit, mapFunc);
		return cache;
	}

    static void register(Cache<?> cache) {
        CACHES.add(cache);
        List<Cache<?>> owner = CAPTURE.get();
        if (owner != null) owner.add(cache);
    }

    static void unregister(Cache<?> cache) { CACHES.remove(cache); }

    /** Capture caches constructed synchronously by one generator, including river caches. */
    public static <T> Captured<T> capture(Supplier<T> factory) {
        List<Cache<?>> previous = CAPTURE.get();
        List<Cache<?>> captured = new ArrayList<>();
        CAPTURE.set(captured);
        try {
            return new Captured<>(factory.get(), List.copyOf(captured));
        } catch (RuntimeException | Error failure) {
            captured.forEach(Cache::close);
            throw failure;
        } finally {
            if (previous == null) CAPTURE.remove();
            else { CAPTURE.set(previous); previous.addAll(captured); }
        }
    }

    public record Captured<T>(T value, List<Cache<?>> caches) {}

    public static int activeCacheCount() { return CACHES.size(); }
	
	public static void clear() throws Exception {
        List<Cache<?>> snapshot;
        synchronized (CACHES) {
            snapshot = List.copyOf(CACHES);
            CACHES.clear();
        }
        // Never invoke callbacks or wait for map locks while holding the owner list lock.
        snapshot.forEach(Cache::close);
	}
}
