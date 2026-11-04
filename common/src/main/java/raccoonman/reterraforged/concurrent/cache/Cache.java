package raccoonman.reterraforged.concurrent.cache;

import java.util.concurrent.ScheduledThreadPoolExecutor;
import java.util.concurrent.ScheduledFuture;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.atomic.AtomicBoolean;
import java.util.function.IntFunction;
import java.util.function.LongFunction;

import raccoonman.reterraforged.concurrent.cache.map.LongMap;

public class Cache<V extends ExpiringEntry> implements AutoCloseable {
	public static final ScheduledThreadPoolExecutor SCHEDULER = createScheduler();

    private static ScheduledThreadPoolExecutor createScheduler() {
        ScheduledThreadPoolExecutor executor = new ScheduledThreadPoolExecutor(1, task ->
            new Thread(task, "TerraForged-cache-maintenance"));
        executor.setRemoveOnCancelPolicy(true);
        return executor;
    }
	
    private LongMap<V> map;
    private long lifetimeMS;
    private volatile long timeout;
    private ScheduledFuture<?> poll;
    private final AtomicBoolean closed = new AtomicBoolean();
    
    public Cache(int capacity, long expireTime, long pollInterval, TimeUnit unit, IntFunction<LongMap<V>> mapFunc) {
        this.timeout = 0L;
        this.map = mapFunc.apply(capacity);
        this.lifetimeMS = unit.toMillis(expireTime);
        
        long intervalMillis = unit.toMillis(pollInterval);
        this.poll = SCHEDULER.scheduleAtFixedRate(this::poll, intervalMillis, intervalMillis, TimeUnit.MILLISECONDS);
        CacheManager.register(this);
    }
    
    public void remove(long key) {
        this.map.remove(key, ExpiringEntry::close);
    }
    
    public V get(long key) {
        if (this.closed.get()) return null;
        return this.map.get(key);
    }
    
    public V computeIfAbsent(long key, LongFunction<V> func) {
        if (this.closed.get()) throw new IllegalStateException("Cache is closed");
        return this.map.computeIfAbsent(key, value -> {
            if (this.closed.get()) throw new IllegalStateException("Cache is closed");
            return func.apply(value);
        });
    }
    
    public void poll() {
        if (this.closed.get()) return;
        this.timeout = System.currentTimeMillis() - this.lifetimeMS;
        this.map.removeIf((entry) -> entry.getTimestamp() < this.timeout);
    }

	@Override
	public void close() {
        if (this.closed.compareAndSet(false, true)) {
            this.poll.cancel(false);
            // Drop ownership without closing values still used by in-flight readers.
            // In particular, never return live Tile arrays to a pool here.
            try {
                this.map.clear();
            } finally {
                CacheManager.unregister(this);
            }
        }
	}

    public int size() { return this.map.size(); }
    public boolean isClosed() { return this.closed.get(); }
}
