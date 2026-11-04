package raccoonman.reterraforged.concurrent;

import java.util.concurrent.ThreadPoolExecutor;
import java.util.concurrent.LinkedBlockingQueue;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.atomic.AtomicInteger;

public class ThreadPools {
	private static final AtomicInteger NEXT_WORKER = new AtomicInteger();
	public static final ThreadPoolExecutor WORLD_GEN = new ThreadPoolExecutor(availableProcessors(), availableProcessors(),
        0L, TimeUnit.MILLISECONDS, new LinkedBlockingQueue<>(), task ->
            new Thread(task, "TerraForged-terrain-" + NEXT_WORKER.incrementAndGet()));

    /** Resize the shared terrain pool without replacing an executor that owns pending work. */
    public static synchronized void configure(int threads) {
        if (threads < 1 || threads > raccoonman.reterraforged.config.PerformanceConfig.MAX_THREAD_COUNT) {
            throw new IllegalArgumentException("Invalid terrain worker count: " + threads);
        }
        if (threads > WORLD_GEN.getMaximumPoolSize()) WORLD_GEN.setMaximumPoolSize(threads);
        WORLD_GEN.setCorePoolSize(threads);
        WORLD_GEN.setMaximumPoolSize(threads);
    }
	
	public static int availableProcessors() {
		return Math.max(2, Runtime.getRuntime().availableProcessors());
	}
}
