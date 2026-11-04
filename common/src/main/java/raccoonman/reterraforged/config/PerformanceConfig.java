package raccoonman.reterraforged.config;

import java.nio.file.Path;
import java.nio.file.Files;
import java.nio.charset.StandardCharsets;
import java.io.IOException;
import java.io.StringReader;
import java.util.Properties;

import com.mojang.serialization.DataResult;

import raccoonman.reterraforged.concurrent.ThreadPools;
import raccoonman.reterraforged.platform.ConfigUtil;

public record PerformanceConfig(int tileSize, int batchCount, int threadCount) {
	public static final Path DEFAULT_FILE_PATH = ConfigUtil.rtf("performance_internal.conf");
	
    // Larger tiles grow quadratically and can allocate gigabytes per tile.
    public static final int MAX_TILE_SIZE = 5;
    public static final int MAX_BATCH_COUNT = 20;
    public static final int MAX_THREAD_COUNT = Math.min(256, Math.max(2, Runtime.getRuntime().availableProcessors() * 2));

    /** Read a process-local properties file. Invalid input is reported, never overwritten. */
    public static synchronized DataResult<PerformanceConfig> read(Path path) {
        try {
            if (!Files.exists(path)) {
                Path parent = path.toAbsolutePath().getParent();
                if (parent != null) Files.createDirectories(parent);
                PerformanceConfig defaults = makeDefault();
                Files.writeString(path, "# TerraForged terrain workers; restart the game/server after changing.\n"
                    + "# tileSize=1.." + MAX_TILE_SIZE + ", batchCount=1.." + MAX_BATCH_COUNT
                    + ", threadCount=1.." + MAX_THREAD_COUNT + "\n"
                    + "tileSize=" + defaults.tileSize() + "\nbatchCount=" + defaults.batchCount()
                    + "\nthreadCount=" + defaults.threadCount() + "\n", StandardCharsets.UTF_8,
                    java.nio.file.StandardOpenOption.CREATE_NEW);
                return DataResult.success(defaults);
            }
            return parse(Files.readString(path, StandardCharsets.UTF_8));
        } catch (IOException | IllegalArgumentException failure) {
            return DataResult.error(() -> "Cannot read TerraForged performance configuration " + path + ": " + failure.getMessage());
        }
    }

    public static DataResult<PerformanceConfig> parse(String text) {
        try {
            Properties values = new Properties();
            values.load(new StringReader(text));
            for (String key : values.stringPropertyNames()) {
                if (!key.equals("tileSize") && !key.equals("batchCount") && !key.equals("threadCount")) {
                    throw new IllegalArgumentException("Unknown setting: " + key);
                }
            }
            PerformanceConfig defaults = makeDefault();
            return DataResult.success(new PerformanceConfig(
                integer(values, "tileSize", defaults.tileSize(), MAX_TILE_SIZE),
                integer(values, "batchCount", defaults.batchCount(), MAX_BATCH_COUNT),
                integer(values, "threadCount", defaults.threadCount(), MAX_THREAD_COUNT)));
        } catch (IOException | IllegalArgumentException failure) {
            return DataResult.error(() -> "Invalid TerraForged performance configuration: " + failure.getMessage());
        }
    }

    private static int integer(Properties values, String key, int fallback, int maximum) {
        String text = values.getProperty(key);
        int value = text == null ? fallback : Integer.parseInt(text.trim());
        if (value < 1 || value > maximum) throw new IllegalArgumentException(key + " must be between 1 and " + maximum);
        return value;
    }
    
    public static PerformanceConfig makeDefault() {
        int tileSize = 3;
        int batchCount = 6;
        int threadCount = Math.min(ThreadPools.availableProcessors(), MAX_THREAD_COUNT);
        return new PerformanceConfig(tileSize, batchCount, threadCount);
    }
}
