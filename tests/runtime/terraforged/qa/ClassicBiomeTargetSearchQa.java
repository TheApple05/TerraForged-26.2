package terraforged.qa;

import java.nio.file.Files;
import java.nio.file.Path;
import java.util.LinkedHashMap;
import java.util.Map;
import com.google.gson.GsonBuilder;
import com.google.gson.JsonParser;
import net.minecraft.core.QuartPos;
import net.minecraft.server.level.ServerLevel;
import raccoonman.reterraforged.world.worldgen.RTFRandomState;
import raccoonman.reterraforged.world.worldgen.cell.Cell;

/** Qualify natural targets with completed filtered terrain, without changing any biome. */
public final class ClassicBiomeTargetSearchQa {
    public static Map<String, Object> verify(ServerLevel level) throws Exception {
        String input = System.getProperty("terraforged.qa.qualifyClassicTargets");
        if (input == null) return Map.of("requested", false);
        Path originsPath = Path.of(input).toAbsolutePath().normalize();
        var origins = JsonParser.parseString(Files.readString(originsPath)).getAsJsonObject();
        var state = level.getChunkSource().randomState();
        var context = ((RTFRandomState) (Object) state).generatorContext();
        if (context == null) throw new IllegalStateException("Target qualification requires the actual custom context");
        var source = level.getChunkSource().getGenerator().getBiomeSource();
        int maximumRadius = Integer.getInteger("terraforged.qa.qualifyClassicRadius", 128);
        if (maximumRadius < 0 || maximumRadius > 512) throw new IllegalArgumentException("Bound search to 0..512 blocks");
        Map<String, Object> targets = new LinkedHashMap<>();
        Map<String, Object> searches = new LinkedHashMap<>();
        Cell cell = new Cell();
        long started = System.nanoTime();
        for (var entry : origins.entrySet()) {
            var origin = entry.getValue().getAsJsonObject();
            int originX = origin.get("x").getAsInt(), originZ = origin.get("z").getAsInt();
            int y = origin.get("y").getAsInt(), checked = 0;
            Map<String, Object> found = null;
            search: for (int radius = 0; radius <= maximumRadius; radius += 4) {
                for (int dz = -radius; dz <= radius; dz += 4) for (int dx = -radius; dx <= radius; dx += 4) {
                    if (Math.abs(dx) != radius && Math.abs(dz) != radius) continue;
                    int x = originX + dx, z = originZ + dz;
                    // Complete the same filtered tile used during real biome generation.
                    context.lookup.applyCell(cell.reset(), x, z, true, true);
                    String actual = source.getNoiseBiome(QuartPos.fromBlock(x), QuartPos.fromBlock(y),
                        QuartPos.fromBlock(z), state.sampler()).unwrapKey().orElseThrow().identifier().toString();
                    checked++;
                    if (!actual.equals(entry.getKey())) continue;
                    found = Map.of("x", x, "y", y, "z", z, "terrain", cell.terrain.getName(),
                        "filteredHeight", cell.height, "filteredTerrainQualified", true);
                    break search;
                }
            }
            if (found == null) throw new IllegalStateException("No filtered natural target within " + maximumRadius + " blocks of " + entry.getKey());
            targets.put(entry.getKey(), found);
            searches.put(entry.getKey(), Map.of("originX", originX, "originZ", originZ, "sourceCalls", checked));
        }
        Path output = Path.of(System.getProperty("terraforged.qa.output", "qa-output"));
        Files.createDirectories(output);
        Path result = output.resolve("classic-filtered-targets.json");
        Files.writeString(result, new GsonBuilder().setPrettyPrinting().create().toJson(targets) + "\n");
        return Map.of("requested", true, "targets", targets, "searches", searches, "path", result.toAbsolutePath().toString(),
            "maximumRadiusBlocks", maximumRadius, "elapsedMillis", (System.nanoTime() - started) / 1_000_000.0,
            "scope", "Actual source after filtered tile completion; no forced biome or block mutation. FULL stored/save/reload checks remain separate.");
    }
}
