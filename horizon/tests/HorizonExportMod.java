package terraforged.horizon;

import com.google.gson.Gson;
import com.google.gson.GsonBuilder;
import net.fabricmc.api.ModInitializer;
import net.fabricmc.fabric.api.event.lifecycle.v1.ServerLifecycleEvents;
import net.minecraft.core.BlockPos;
import net.minecraft.core.registries.Registries;
import net.minecraft.server.MinecraftServer;
import net.minecraft.world.level.block.state.BlockState;
import net.minecraft.world.level.block.state.properties.Property;
import raccoonman.reterraforged.RTFCommon;
import raccoonman.reterraforged.world.worldgen.RTFRandomState;

import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.security.MessageDigest;
import java.util.ArrayList;
import java.util.HexFormat;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.TreeMap;
import java.util.TreeSet;

/** Developer-only snapshot preparation. No Java runs in the native Horizon player. */
public final class HorizonExportMod implements ModInitializer {
    private static final Gson JSON = new GsonBuilder().setPrettyPrinting().create();
    @Override public void onInitialize() { ServerLifecycleEvents.SERVER_STARTED.register(HorizonExportMod::run); }

    private static void run(MinecraftServer server) {
        Path qa = Path.of(System.getProperty("terraforged.qa.output")).toAbsolutePath();
        Map<String, Object> report = new LinkedHashMap<>();
        report.put("status", "failed"); report.put("mode", "generate");
        report.put("serverStarted", true); report.put("saveCompleted", false);
        try {
            var level = server.overworld();
            var context = ((RTFRandomState) (Object) level.getChunkSource().randomState()).generatorContext();
            require(context != null && context.cache != null, "Requires actual TerraForged generation; vanilla fallback refused");
            int radius = Integer.getInteger("terraforged.horizon.radius", 1);
            int centerX = Integer.getInteger("terraforged.horizon.chunkX", 0);
            int centerZ = Integer.getInteger("terraforged.horizon.chunkZ", 0);
            require(radius >= 0 && radius <= 16, "Explicit bounded radius 0..16 required");
            boolean seekClassic = Boolean.getBoolean("terraforged.horizon.seekClassic");
            if (seekClassic) {
                boolean found = false;
                var source = level.getChunkSource().getGenerator().getBiomeSource();
                var sampler = level.getChunkSource().randomState().sampler();
                search: for (int dz = -8; dz <= 8; dz++) for (int dx = -8; dx <= 8; dx++) {
                    int cx = centerX + dx * 64, cz = centerZ + dz * 64;
                    var key = source.getNoiseBiome(cx * 4 + 2, 20, cz * 4 + 2, sampler).unwrapKey().orElseThrow();
                    if (key.identifier().getNamespace().equals("reterraforged")) {
                        centerX = cx; centerZ = cz; found = true; break search;
                    }
                }
                require(found, "No actual classic biome found in the bounded 289-position survey");
            }
            Path root = qa.resolve("horizon-prepared");
            require(!Files.exists(root), "Snapshot destination already exists; choose a fresh QA instance");
            Files.createDirectories(root.resolve("chunks"));
            // Features write into neighboring chunks. Complete a one-chunk FULL halo first.
            for (int z = centerZ - radius - 1; z <= centerZ + radius + 1; z++)
                for (int x = centerX - radius - 1; x <= centerX + radius + 1; x++) level.getChunk(x, z);
            Map<String, String> hashes = new TreeMap<>();
            var observedClassicBiomes = new TreeSet<String>();
            long blocks = 0, quartBiomes = 0;
            int minY = level.getMinY(), height = level.getMaxY() - minY + 1;
            var biomeRegistry = level.registryAccess().lookupOrThrow(Registries.BIOME);
            for (int cz = centerZ - radius; cz <= centerZ + radius; cz++) {
                for (int cx = centerX - radius; cx <= centerX + radius; cx++) {
                    var chunk = level.getChunk(cx, cz);
                    // The v1 bridge does not transfer typed NBT/structure state; refuse a lossy snapshot.
                    require(chunk.getBlockEntities().isEmpty(), "Chunk contains block entities; v1 snapshot cannot preserve them: " + cx + "," + cz);
                    List<Map<String, Object>> palette = new ArrayList<>();
                    Map<BlockState, Integer> ids = new LinkedHashMap<>();
                    List<int[]> runs = new ArrayList<>();
                    BlockPos.MutableBlockPos pos = new BlockPos.MutableBlockPos();
                    for (int y = minY; y < minY + height; y++) for (int z = 0; z < 16; z++) for (int x = 0; x < 16; x++) {
                        BlockState state = chunk.getBlockState(pos.set(cx * 16 + x, y, cz * 16 + z));
                        Integer id = ids.get(state);
                        if (id == null) {
                            id = palette.size(); ids.put(state, id);
                            Map<String, String> properties = new TreeMap<>();
                            state.getValues().forEach(value -> properties.put(value.property().getName(), property(value.property(), value.value())));
                            palette.add(Map.of("name", state.getBlock().builtInRegistryHolder().key().identifier().toString(), "properties", properties));
                        }
                        append(runs, id); blocks++;
                    }
                    List<String> biomePalette = new ArrayList<>();
                    List<int[]> biomeRuns = new ArrayList<>();
                    for (int qy = minY >> 2; qy < (minY + height) >> 2; qy++) for (int z = 0; z < 4; z++) for (int x = 0; x < 4; x++) {
                        String name = biomeRegistry.getKey(chunk.getNoiseBiome(cx * 4 + x, qy, cz * 4 + z).value()).toString();
                        if (name.startsWith("reterraforged:")) observedClassicBiomes.add(name);
                        int id = biomePalette.indexOf(name);
                        if (id < 0) { id = biomePalette.size(); biomePalette.add(name); }
                        append(biomeRuns, id); quartBiomes++;
                    }
                    Map<String, Object> entry = new LinkedHashMap<>();
                    entry.put("schema", 1); entry.put("position", new int[] {cx, cz}); entry.put("minY", minY); entry.put("height", height);
                    entry.put("blockPalette", palette); entry.put("blocks", runs); entry.put("biomePalette", biomePalette); entry.put("biomes", biomeRuns);
                    Path file = root.resolve("chunks").resolve(cx + "." + cz + ".json");
                    Files.writeString(file, JSON.toJson(entry) + "\n", StandardCharsets.UTF_8);
                    hashes.put(cx + "," + cz, hash(file));
                }
            }
            Path artifact = Path.of(RTFCommon.class.getProtectionDomain().getCodeSource().getLocation().toURI());
            require(Files.isRegularFile(artifact), "Snapshot preparation requires the packaged mod artifact");
            Map<String, Object> manifest = new LinkedHashMap<>();
            manifest.put("schema", 1); manifest.put("sourceMinecraft", "26.2"); manifest.put("targetMinecraft", "26.3");
            manifest.put("seed", level.getSeed()); manifest.put("minY", minY); manifest.put("height", height);
            manifest.put("seaLevel", level.getSeaLevel()); manifest.put("spawn", new int[] {centerX * 16 + 8, centerZ * 16 + 8});
            manifest.put("sourceModSha256", hash(artifact)); manifest.put("chunks", hashes);
            require(!seekClassic || !observedClassicBiomes.isEmpty(), "Survey candidate did not store an actual classic biome after FULL generation");
            Files.writeString(root.resolve("terraforged-prepared.json"), JSON.toJson(manifest) + "\n", StandardCharsets.UTF_8);
            report.put("snapshot", Map.of("path", root.toString(), "chunkCount", hashes.size(), "blockCount", blocks, "quartBiomeCount", quartBiomes,
                "manifestSha256", hash(root.resolve("terraforged-prepared.json")), "sourceModSha256", hash(artifact), "entitiesOrStructureMetadataTransferred", false));
            report.put("observedClassicBiomeIds", observedClassicBiomes);
            report.put("nodes", List.of(Map.of("name", "custom-generator-active", "status", "passed")));
            server.saveEverything(false, true, true);
            report.put("saveCompleted", true); report.put("status", "passed");
        } catch (Throwable failure) {
            report.put("error", failure.toString()); failure.printStackTrace();
        } finally {
            try { Files.createDirectories(qa); Files.writeString(qa.resolve("result.json"), JSON.toJson(report) + "\n", StandardCharsets.UTF_8); }
            catch (Exception failure) { failure.printStackTrace(); }
            System.out.println("TERRAFORGED_QA_RESULT_JSON=" + new Gson().toJson(report));
            server.halt(false);
        }
    }
    private static void append(List<int[]> runs, int value) {
        if (!runs.isEmpty() && runs.getLast()[0] == value) runs.getLast()[1]++;
        else runs.add(new int[] {value, 1});
    }
    @SuppressWarnings({"rawtypes", "unchecked"}) private static String property(Property property, Comparable value) { return property.getName(value); }
    private static String hash(Path path) throws Exception { return HexFormat.of().formatHex(MessageDigest.getInstance("SHA-256").digest(Files.readAllBytes(path))); }
    private static void require(boolean value, String message) { if (!value) throw new IllegalStateException(message); }
}
