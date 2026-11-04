package terraforged.qa;

import java.io.PrintWriter;
import java.io.StringWriter;
import java.net.URI;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.StandardCopyOption;
import java.security.MessageDigest;
import java.time.Instant;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.Collections;
import java.util.Comparator;
import java.util.HexFormat;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.TreeMap;
import java.util.UUID;

import com.google.gson.Gson;
import com.google.gson.GsonBuilder;
import com.google.gson.JsonObject;
import com.google.gson.JsonParser;

import net.fabricmc.api.ModInitializer;
import net.fabricmc.fabric.api.event.lifecycle.v1.ServerLifecycleEvents;
import net.fabricmc.loader.api.FabricLoader;
import net.minecraft.SharedConstants;
import net.minecraft.core.BlockPos;
import net.minecraft.core.Registry;
import net.minecraft.data.PackOutput;
import net.minecraft.resources.Identifier;
import net.minecraft.server.MinecraftServer;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.world.level.chunk.ChunkAccess;
import net.minecraft.world.level.levelgen.Heightmap;
import raccoonman.reterraforged.RTFCommon;
import raccoonman.reterraforged.data.worldgen.Datapacks;
import raccoonman.reterraforged.data.worldgen.preset.settings.Preset;
import raccoonman.reterraforged.data.worldgen.preset.settings.Presets;
import raccoonman.reterraforged.registries.RTFRegistries;
import raccoonman.reterraforged.world.worldgen.GeneratorContext;
import raccoonman.reterraforged.world.worldgen.RTFRandomState;
import raccoonman.reterraforged.world.worldgen.cell.Cell;
import raccoonman.reterraforged.world.worldgen.feature.template.TemplateBiomeDecoratorProbe;

/** Test-only companion. Never packaged inside the production mod. */
public final class RuntimeQaMod implements ModInitializer {
    private static final Gson JSON = new GsonBuilder().setPrettyPrinting().create();
    private static final Gson COMPACT = new Gson();
    private static final int[][] CHUNKS = {{0, 0}, {-1, -1}, {7, 0}, {8, -1}, {-8, 7}, {-9, 8}};
    private static final int[] LOCALS = {0, 7, 15};

    @Override
    public void onInitialize() {
        ServerLifecycleEvents.SERVER_STARTED.register(RuntimeQaMod::run);
    }

    private static void run(MinecraftServer server) {
        String mode = System.getProperty("terraforged.qa.mode", "missing");
        Path output = Path.of(System.getProperty("terraforged.qa.output", "qa-output")).toAbsolutePath().normalize();
        Map<String, Object> report = new LinkedHashMap<>();
        List<Map<String, Object>> nodes = new ArrayList<>();
        report.put("schemaVersion", 1);
        report.put("mode", mode);
        report.put("startedAt", Instant.now().toString());
        report.put("serverStarted", true);
        report.put("saveCompleted", false);
        report.put("status", "failed");
        report.put("nodes", nodes);
        report.put("notTested", List.of("Visual parity", "All presets and configuration extremes", "All biomes or structures", "Multiplayer clients", "Third-party mod combinations", "Performance benchmark"));
        try {
            Files.createDirectories(output);
            report.put("runtime", provenance());
            node(nodes, "production-artifact", "passed", "Packaged origin and runtime recorded");
            switch (mode) {
                case "export" -> exportPreset(server, output, report, nodes);
                case "generate", "reload" -> sampleWorld(server, output, mode, report, nodes);
                default -> throw new IllegalArgumentException("Explicit terraforged.qa.mode=export|generate|reload is required");
            }
            TemplateBiomeDecoratorProbe.main(new String[0]);
            node(nodes, "biome-decorator-overrides", "passed", "Three production-helper assertions completed with real target holders");
            report.put("status", "passed");
        } catch (Throwable failure) {
            report.put("error", describe(failure));
            node(nodes, "callback", "failed", failure.toString());
        } finally {
            report.put("finishedAt", Instant.now().toString());
            report.put("cleanStopRequested", true);
            try {
                Files.createDirectories(output);
                Path pending = output.resolve("result.json.tmp");
                Files.writeString(pending, JSON.toJson(report) + "\n", StandardCharsets.UTF_8);
                Files.move(pending, output.resolve("result.json"), StandardCopyOption.REPLACE_EXISTING);
            } catch (Throwable writeFailure) {
                report.put("status", "failed");
                report.put("resultWriteError", describe(writeFailure));
            }
            System.out.println("TERRAFORGED_QA_RESULT_JSON=" + COMPACT.toJson(report));
            // The server harness must verify the process exit and shutdown log as well as this result.
            server.halt(false);
        }
    }

    private static Map<String, Object> provenance() throws Exception {
        FabricLoader loader = FabricLoader.getInstance();
        require(!loader.isDevelopmentEnvironment(), "QA requires a production Fabric launch, not development mode");
        var mod = loader.getModContainer("reterraforged").orElseThrow();
        URI source = RTFCommon.class.getProtectionDomain().getCodeSource().getLocation().toURI();
        Path origin = Path.of(source).toAbsolutePath().normalize();
        require(Files.isRegularFile(origin) && origin.getFileName().toString().endsWith(".jar"), "Production class must originate from a packaged JAR: " + origin);
        String actualSha = sha256(origin);
        String expectedSha = System.getProperty("terraforged.qa.expectedModSha256", "");
        if (!expectedSha.isBlank()) require(actualSha.equalsIgnoreCase(expectedSha), "Production JAR SHA-256 differs from the requested artifact");
        Map<String, Object> runtime = new LinkedHashMap<>();
        runtime.put("minecraft", SharedConstants.getCurrentVersion().id());
        runtime.put("java", System.getProperty("java.version"));
        runtime.put("javaClassVersion", System.getProperty("java.class.version"));
        runtime.put("developmentEnvironment", loader.isDevelopmentEnvironment());
        runtime.put("loader", loader.getModContainer("fabricloader").orElseThrow().getMetadata().getVersion().getFriendlyString());
        runtime.put("modVersion", mod.getMetadata().getVersion().getFriendlyString());
        runtime.put("modClassOrigin", origin.toString());
        runtime.put("modSha256", actualSha);
        runtime.put("expectedModSha256", expectedSha.isBlank() ? null : expectedSha);
        runtime.put("loaderModOrigins", mod.getOrigin().getPaths().stream().map(Path::toString).toList());
        return runtime;
    }

    private static void exportPreset(MinecraftServer server, Path output, Map<String, Object> report, List<Map<String, Object>> nodes) throws Throwable {
        Path pack = output.resolve("presetpack");
        if (Files.exists(pack)) {
            try (var files = Files.walk(pack)) {
                require(files.noneMatch(Files::isRegularFile), "Preset output must be empty; choose a fresh QA output directory");
            }
        }
        Files.createDirectories(pack);
        Path cache = output.resolve("cache-export-" + UUID.randomUUID());
        long start = System.nanoTime();
        try {
            Datapacks.makePreset(Presets.makeLegacyDefault(), server.registryAccess(), cache, pack, "Runtime QA legacy default").run();
        } catch (Throwable failure) {
            Map<String, Object> partial = new LinkedHashMap<>();
            partial.put("presetPack", pack.toString());
            partial.put("cache", cache.toString());
            try (var files = Files.walk(pack)) {
                List<Path> written = files.filter(Files::isRegularFile).toList();
                partial.put("fileCount", written.size());
                partial.put("jsonFileCount", written.stream().filter(file -> file.toString().endsWith(".json")).count());
                if (Files.isRegularFile(pack.resolve("pack.mcmeta"))) {
                    partial.put("packMetadata", JsonParser.parseString(Files.readString(pack.resolve("pack.mcmeta"))));
                }
            } catch (Throwable inventoryFailure) {
                partial.put("inventoryError", describe(inventoryFailure));
            }
            report.put("export", partial);
            throw failure;
        }
        node(nodes, "legacy-preset-export", "passed", "Actual Datapacks.makePreset(...).run() completed");

        Path metadataPath = pack.resolve("pack.mcmeta");
        require(Files.isRegularFile(metadataPath), "Preset export did not write pack.mcmeta");
        JsonObject metadata = JsonParser.parseString(Files.readString(metadataPath)).getAsJsonObject();
        require(metadata.has("pack"), "Pack metadata has no pack object");
        Identifier registryId = RTFRegistries.PRESET.identifier();
        String directory = registryId.getNamespace().equals("minecraft") ? registryId.getPath() : registryId.getNamespace() + "/" + registryId.getPath();
        Path presetPath = new PackOutput(pack).createPathProvider(PackOutput.Target.DATA_PACK, directory).json(Preset.KEY.identifier());
        require(Files.isRegularFile(presetPath), "Export omitted the actual custom preset registry entry: " + presetPath);
        JsonObject preset = JsonParser.parseString(Files.readString(presetPath)).getAsJsonObject();
        require(preset.has("terrain") && preset.has("rivers") && preset.has("world"), "Custom preset entry has no terrain/river/world settings");

        Map<String, String> hashes = new TreeMap<>();
        try (var files = Files.walk(pack)) {
            for (Path file : files.filter(Files::isRegularFile).toList()) {
                hashes.put(pack.relativize(file).toString().replace('\\', '/'), sha256(file));
            }
        }
        long jsonCount = hashes.keySet().stream().filter(path -> path.endsWith(".json")).count();
        require(jsonCount > 0, "Preset export produced no JSON resources");
        report.put("export", Map.of("presetPack", pack.toString(), "cache", cache.toString(), "presetEntry", pack.relativize(presetPath).toString(), "fileCount", hashes.size(), "jsonFileCount", jsonCount, "packMetadata", metadata, "fileSha256", hashes, "elapsedMillis", (System.nanoTime() - start) / 1_000_000));
        node(nodes, "export-inventory", "passed", hashes.size() + " files inventoried, including the actual custom preset entry");
    }

    private static void sampleWorld(MinecraftServer server, Path output, String mode, Map<String, Object> report, List<Map<String, Object>> nodes) throws Exception {
        ServerLevel level = server.overworld();
        Registry<Preset> presets = level.registryAccess().lookupOrThrow(RTFRegistries.PRESET);
        boolean vanillaControl = Boolean.getBoolean("terraforged.qa.vanillaControl");
        require(presets.get(Preset.KEY).isPresent() != vanillaControl,
            vanillaControl ? "Vanilla control requires absent custom preset" : "Custom Preset.KEY is absent; refusing vanilla fallback");
        report.put("generator", vanillaControl ? "vanilla-control" : "terraforged-custom");
        Object randomState = level.getChunkSource().randomState();
        require(randomState instanceof RTFRandomState, "Actual random state does not have the required TerraForged interface");
        GeneratorContext context = ((RTFRandomState) randomState).generatorContext();
        if (vanillaControl) {
            require(context == null && ((RTFRandomState) randomState).preset() == null, "Vanilla control unexpectedly has a TerraForged context/preset");
            node(nodes, "explicit-vanilla-control", "passed", "Explicit control requires absent custom preset/context; this is not custom terrain evidence");
        } else {
            require(context != null && context.cache != null, "TerraForged generator context/cache is null; refusing vanilla fallback");
            require(((RTFRandomState) randomState).preset() != null, "Random state has no TerraForged preset");
            node(nodes, "custom-generator-active", "passed", "Custom preset entry, RTFRandomState, GeneratorContext and TileCache are present");
        }
        if (raccoonman.reterraforged.world.worldgen.terrablender.TBCompat.isEnabled()) {
            var sampler = level.getChunkSource().randomState().sampler();
            require((Object) sampler instanceof raccoonman.reterraforged.world.worldgen.terrablender.TBClimateSampler,
                "Loaded TerraBlender sampler is missing the compatibility interface");
            var extended = (raccoonman.reterraforged.world.worldgen.terrablender.TBClimateSampler) (Object) sampler;
            require(extended.getUniqueness() != null, "TerraBlender uniqueness density was not installed");
            Object point = sampler.sample(0, 0, 0);
            require(point instanceof raccoonman.reterraforged.world.worldgen.terrablender.TBTargetPoint,
                "Sampled climate target is missing the compatibility interface");
            double uniqueness = ((raccoonman.reterraforged.world.worldgen.terrablender.TBTargetPoint) point).getUniqueness();
            require(Double.isFinite(uniqueness), "TerraBlender sampled uniqueness was not populated");
            node(nodes, "terrablender-sampler-uniqueness", "passed",
                "Actual global sampler and climate target carry finite uniqueness: " + uniqueness);
        }

        List<Map<String, Object>> samples = new ArrayList<>();
        List<Map<String, Object>> chunkEvidence = new ArrayList<>();
        long start = System.nanoTime();
        String workload = System.getProperty("terraforged.qa.workload", "default");
        boolean boundaryWorkload = workload.equals("boundary64");
        require(boundaryWorkload || workload.equals("default"), "Unknown QA workload: " + workload);
        List<int[]> sampleChunks = boundaryWorkload ? boundaryChunks() : Arrays.asList(CHUNKS);
        List<int[]> requestedChunks = new ArrayList<>(sampleChunks);
        if (boundaryWorkload) {
            // Complete a FULL one-chunk halo before observing any sampled column. Features from
            // neighboring chunks can write leaves/logs across boundaries after an earlier sample.
            Map<String, int[]> halo = new TreeMap<>();
            for (int[] coordinate : sampleChunks) for (int dx = -1; dx <= 1; dx++) for (int dz = -1; dz <= 1; dz++) {
                int x = coordinate[0] + dx, z = coordinate[1] + dz;
                halo.put(x + "," + z, new int[] {x, z});
            }
            requestedChunks = new ArrayList<>(halo.values());
            requestedChunks.sort(Comparator.<int[]>comparingInt(value -> value[0]).thenComparingInt(value -> value[1]));
            if (Boolean.getBoolean("terraforged.qa.reverseRequests")) Collections.reverse(requestedChunks);
            for (int[] coordinate : requestedChunks) level.getChunk(coordinate[0], coordinate[1]);
            node(nodes, "full-halo-before-sampling", "passed", requestedChunks.size() + " FULL chunks completed before sampling " + sampleChunks.size() + " chunks");
        }
        report.put("workload", Map.of("name", workload, "sampledChunkCount", sampleChunks.size(),
            "fullRequestedChunkCount", requestedChunks.size(), "reverseRequests", Boolean.getBoolean("terraforged.qa.reverseRequests"),
            "fullHaloChunks", boundaryWorkload ? 1 : 0));
        for (int[] coordinate : sampleChunks) {
            int cx = coordinate[0], cz = coordinate[1];
            ChunkAccess chunk = level.getChunk(cx, cz);
            require(chunk.getPos().x() == cx && chunk.getPos().z() == cz, "Loaded wrong chunk");
            // Map.of iteration order is salted per JVM; sort all map keys included in the cross-run digest.
            Map<String, Object> chunkRecord = new TreeMap<>(Map.of("x", cx, "z", cz, "status", chunk.getPersistedStatus().toString()));
            if (!vanillaControl) {
                chunkRecord.put("tileX", context.cache.chunkToTile(cx));
                chunkRecord.put("tileZ", context.cache.chunkToTile(cz));
            }
            chunkEvidence.add(chunkRecord);
            for (int lx : LOCALS) for (int lz : LOCALS) {
                int x = cx * 16 + lx, z = cz * 16 + lz;
                int surface = chunk.getHeight(Heightmap.Types.WORLD_SURFACE, lx, lz);
                require(surface >= level.getMinY() && surface <= level.getMaxY(), "Surface height outside dimension bounds at " + x + "," + z + ": " + surface);
                BlockPos pos = new BlockPos(x, surface, z);
                var biome = level.getBiome(pos).unwrapKey().orElseThrow(() -> new IllegalStateException("Unregistered biome at " + pos));
                Cell cell = new Cell();
                if (!vanillaControl) require(context.lookup.applyCell(cell, x, z, true, true), "Accurate terrain cell was not available at " + x + "," + z);
                Map<String, Object> sample = new LinkedHashMap<>();
                sample.put("x", x); sample.put("z", z); sample.put("surfaceY", surface);
                if (boundaryWorkload) {
                    sample.put("terrainFloorY", chunk.getHeight(Heightmap.Types.OCEAN_FLOOR_WG, lx, lz));
                    sample.put("columnBlockStatesSha256", columnDigest(level, x, z));
                }
                sample.put("biome", biome.identifier().toString());
                sample.put("surfaceBlock", level.getBlockState(pos).getBlock().builtInRegistryHolder().key().identifier().toString());
                if (!vanillaControl) {
                    Map<String, Integer> bits = new TreeMap<>();
                    finite(bits, "height", cell.height); finite(bits, "gradient", cell.gradient);
                    finite(bits, "temperature", cell.temperature); finite(bits, "moisture", cell.moisture);
                    finite(bits, "erosion", cell.erosion); finite(bits, "weirdness", cell.weirdness);
                    finite(bits, "riverMask", cell.riverMask); finite(bits, "heightErosion", cell.heightErosion);
                    finite(bits, "sediment", cell.sediment);
                    require(cell.terrain != null && cell.biome != null, "Terrain/climate type is absent at " + x + "," + z);
                    sample.put("terrain", cell.terrain.getName());
                    sample.put("cellBiome", cell.biome.toString());
                    sample.put("cellFloatBits", bits);
                }
                samples.add(sample);
            }
            node(nodes, "chunk-" + cx + "," + cz, "passed", "Full chunk and nine column/terrain samples recorded");
        }

        Map<String, Object> deterministic = new LinkedHashMap<>();
        deterministic.put("seed", level.getSeed());
        deterministic.put("minY", level.getMinY()); deterministic.put("maxY", level.getMaxY());
        deterministic.put("chunks", chunkEvidence); deterministic.put("samples", samples);
        String digest = sha256(COMPACT.toJson(deterministic).getBytes(StandardCharsets.UTF_8));
        Map<String, Object> evidence = new LinkedHashMap<>(deterministic);
        evidence.put("digestSha256", digest);
        evidence.put("elapsedMillis", (System.nanoTime() - start) / 1_000_000);
        if (boundaryWorkload) evidence.put("columnHashYBoundsInclusive", List.of(level.getMinY(), level.getMaxY()));
        report.put("generation", evidence);
        if (Boolean.getBoolean("terraforged.qa.dumpColumns")) {
            Path columnsPath = output.resolve("columns-" + workload + ".json");
            Map<String, List<Map<String, Object>>> columns = new TreeMap<>();
            for (Map<String, Object> sample : samples) {
                int x = (Integer) sample.get("x"), z = (Integer) sample.get("z");
                List<Map<String, Object>> runs = new ArrayList<>();
                String previous = null;
                int fromY = level.getMinY();
                for (int y = level.getMinY(); y <= level.getMaxY(); y++) {
                    String state = canonicalState(level, new BlockPos(x, y, z));
                    if (previous != null && !previous.equals(state)) {
                        runs.add(Map.of("fromY", fromY, "toY", y - 1, "state", previous));
                        fromY = y;
                    }
                    previous = state;
                }
                runs.add(Map.of("fromY", fromY, "toY", level.getMaxY(), "state", previous));
                columns.put(x + "," + z, runs);
            }
            Files.writeString(columnsPath, JSON.toJson(columns) + "\n", StandardCharsets.UTF_8);
            report.put("diagnosticColumnDump", Map.of("path", columnsPath.toString(), "sha256", sha256(columnsPath),
                "minYInclusive", level.getMinY(), "maxYInclusive", level.getMaxY(), "columnCount", columns.size()));
        }

        Path baseline = output.resolve(boundaryWorkload ? "generation-boundary64-inclusive-baseline.json" : "generation-baseline.json");
        if (mode.equals("reload")) {
            require(Files.isRegularFile(baseline), "Reload requires an existing generation-baseline.json from this same world");
            JsonObject prior = JsonParser.parseString(Files.readString(baseline)).getAsJsonObject();
            require(prior.get("digestSha256").getAsString().equals(digest), "Persisted/recomputed observations differ from generation baseline");
            node(nodes, "disk-reload-digest", "passed", digest);
        } else {
            require(!Files.exists(baseline), "Generate requires a fresh baseline path; use reload for an existing world");
            Files.writeString(baseline, JSON.toJson(evidence) + "\n", StandardCharsets.UTF_8);
        }
        server.saveEverything(false, true, true);
        report.put("saveCompleted", true);
        node(nodes, "world-save-flush", "passed", "Actual server saveEverything completed with flush and force");
    }

    private static List<int[]> boundaryChunks() {
        List<int[]> result = new ArrayList<>();
        // Four disjoint 4x4 regions cross positive/negative tile corners and the origin.
        for (int[] origin : new int[][] {{-10, -10}, {-2, -2}, {6, 6}, {-66, 62}}) {
            for (int dx = 0; dx < 4; dx++) for (int dz = 0; dz < 4; dz++) result.add(new int[] {origin[0] + dx, origin[1] + dz});
        }
        result.sort(Comparator.<int[]>comparingInt(value -> value[0]).thenComparingInt(value -> value[1]));
        return result;
    }

    private static String columnDigest(ServerLevel level, int x, int z) throws Exception {
        MessageDigest digest = MessageDigest.getInstance("SHA-256");
        BlockPos.MutableBlockPos pos = new BlockPos.MutableBlockPos(x, level.getMinY(), z);
        for (int y = level.getMinY(); y <= level.getMaxY(); y++) {
            pos.setY(y);
            String row = y + ":" + canonicalState(level, pos) + "\n";
            digest.update(row.getBytes(StandardCharsets.UTF_8));
        }
        return HexFormat.of().formatHex(digest.digest());
    }

    private static String canonicalState(ServerLevel level, BlockPos pos) {
        var state = level.getBlockState(pos);
        Map<String, String> properties = new TreeMap<>();
        state.getValues().forEach(value -> properties.put(value.property().getName(), value.value().toString()));
        return state.getBlock().builtInRegistryHolder().key().identifier() + ":" + COMPACT.toJson(properties);
    }

    private static void finite(Map<String, Integer> bits, String name, float value) {
        require(Float.isFinite(value), "Nonfinite terrain field: " + name);
        bits.put(name, Float.floatToRawIntBits(value));
    }

    private static void require(boolean condition, String message) {
        if (!condition) throw new IllegalStateException(message);
    }

    private static void node(List<Map<String, Object>> nodes, String name, String status, String detail) {
        Map<String, Object> node = Map.of("name", name, "status", status, "detail", detail, "at", Instant.now().toString());
        nodes.add(node);
        System.out.println("TERRAFORGED_QA_NODE_JSON=" + COMPACT.toJson(node));
    }

    private static Map<String, String> describe(Throwable failure) {
        StringWriter trace = new StringWriter();
        failure.printStackTrace(new PrintWriter(trace));
        return Map.of("type", failure.getClass().getName(), "message", failure.toString(), "stackTrace", trace.toString());
    }

    private static String sha256(Path path) throws Exception {
        try (var input = Files.newInputStream(path)) {
            MessageDigest digest = MessageDigest.getInstance("SHA-256");
            byte[] buffer = new byte[64 * 1024];
            for (int count; (count = input.read(buffer)) != -1;) digest.update(buffer, 0, count);
            return HexFormat.of().formatHex(digest.digest());
        }
    }

    private static String sha256(byte[] bytes) throws Exception {
        return HexFormat.of().formatHex(MessageDigest.getInstance("SHA-256").digest(bytes));
    }
}
