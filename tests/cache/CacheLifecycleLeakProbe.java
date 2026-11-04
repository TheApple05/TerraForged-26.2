import java.util.concurrent.TimeUnit;
import raccoonman.reterraforged.concurrent.cache.Cache;
import raccoonman.reterraforged.concurrent.cache.CacheManager;
import raccoonman.reterraforged.concurrent.cache.ExpiringEntry;
import raccoonman.reterraforged.concurrent.cache.map.LongMap;

/** Uses only the original API so the exact upstream leak can be reproduced. */
public final class CacheLifecycleLeakProbe {
    public static void main(String[] args) throws Exception {
        try {
            var owners = CacheManager.class.getDeclaredField("CACHES"); owners.setAccessible(true);
            var storage = Cache.class.getDeclaredField("map"); storage.setAccessible(true);
            int before = ((java.util.List<?>) owners.get(null)).size();
            Cache<ExpiringEntry> cache = CacheManager.createCache(8, 60, 20, TimeUnit.SECONDS);
            cache.computeIfAbsent(1, key -> new ExpiringEntry() {
                public long getTimestamp() { return System.currentTimeMillis(); }
                public void close() {}
            });
            cache.close();
            int after = ((java.util.List<?>) owners.get(null)).size();
            int retainedEntries = ((LongMap<?>) storage.get(cache)).size();
            System.out.println("ownersBefore=" + before + ",ownersAfter=" + after + ",entriesAfterClose=" + retainedEntries);
            if (after != before || retainedEntries != 0) throw new AssertionError("Closed cache remains strongly owned and retains map entries");
        } finally { Cache.SCHEDULER.shutdownNow(); }
    }
}
