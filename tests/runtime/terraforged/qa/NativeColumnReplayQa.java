package terraforged.qa;

import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.security.MessageDigest;
import java.util.ArrayList;
import java.util.HexFormat;
import java.util.LinkedHashMap;
import java.util.Map;
import com.google.gson.GsonBuilder;
import com.google.gson.JsonParser;
import net.minecraft.server.level.ServerLevel;

/** Exact native inputs captured from real RuinedPortal structure placement. */
public final class NativeColumnReplayQa {
    public static Map<String, Object> verify(ServerLevel level) throws Exception {
        String input = System.getProperty("terraforged.qa.replayBaseColumns");
        if (input == null) return Map.of("requested", false);
        if (System.getProperty("terraforged.qa.mode", "").startsWith("benchmark")) throw new IllegalStateException("Column replay is functional QA, not a throughput benchmark");
        Path path = Path.of(input).toAbsolutePath().normalize();
        byte[] bytes = Files.readAllBytes(path);
        var captured = JsonParser.parseString(new String(bytes, StandardCharsets.UTF_8)).getAsJsonObject();
        if (captured.get("seed").getAsLong() != level.getSeed()) throw new IllegalStateException("Captured column seed mismatch");
        var generator = level.getChunkSource().getGenerator();
        var state = level.getChunkSource().randomState();
        var measurements = new ArrayList<Map<String, Object>>();
        long total = System.nanoTime();
        for (var element : captured.getAsJsonArray("measurements")) {
            var point = element.getAsJsonObject();
            int x = point.get("x").getAsInt(), z = point.get("z").getAsInt();
            long started = System.nanoTime();
            var column = generator.getBaseColumn(x, z, level, state);
            long elapsed = System.nanoTime() - started;
            var digest = MessageDigest.getInstance("SHA-256");
            int count = 0;
            for (int y = level.getMinY(); y <= level.getMaxY(); y++) {
                digest.update((column.getBlock(y).toString() + "\n").getBytes(StandardCharsets.UTF_8));
                count++;
            }
            measurements.add(Map.of("x", x, "z", z, "blockStatesHashed", count,
                "columnSha256", HexFormat.of().formatHex(digest.digest()), "nativeCallElapsedNanos", elapsed));
        }
        var result = new LinkedHashMap<String, Object>();
        result.put("requested", true); result.put("seed", level.getSeed()); result.put("input", path.toString());
        result.put("inputSha256", HexFormat.of().formatHex(MessageDigest.getInstance("SHA-256").digest(bytes)));
        result.put("nativeApi", "ChunkGenerator.getBaseColumn(x,z,actualServerLevel,actualRandomState)");
        result.put("minY", level.getMinY()); result.put("maxYInclusive", level.getMaxY()); result.put("columns", measurements);
        result.put("elapsedMillis", (System.nanoTime() - total) / 1_000_000.0);
        result.put("scope", "Native exact-input functional column hashes after six baseline FULL chunks. Timings are diagnostics under uncontrolled functional load, not throughput benchmark evidence.");
        Path output = Path.of(System.getProperty("terraforged.qa.output", "qa-output"));
        Files.createDirectories(output);
        Files.writeString(output.resolve("native-column-replay.json"), new GsonBuilder().setPrettyPrinting().create().toJson(result) + "\n");
        return result;
    }
}
