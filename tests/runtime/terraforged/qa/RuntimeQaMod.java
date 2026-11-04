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
        // Export-only work reads frozen registries and writes isolated files.
        // Hashing many files can exceed a tick on slow filesystems; return the
        // lifecycle callback before this read-only QA work starts. Generation,
        // structures and benchmarks still run on the real server thread.
        if ("export".equals(System.getProperty("terraforged.qa.mode"))
                && !Thread.currentThread().getName().equals("TerraForged-export-QA")) {
            Thread.ofPlatform().name("TerraForged-export-QA").start(() -> run(server));
            return;
        }
        String mode = System.getProperty("terraforged.qa.mode", "missing");
        Path output = Path.of(System.getProperty("terraforged.qa.output", "qa-output")).toAbsolutePath().normalize();
        Map<String, Object> report = new LinkedHashMap<>();
        List<Map<String, Object>> nodes = new ArrayList<>();
        report.put("schemaVersion", 1);
        report.put("mode", mode);
        report.put("qaThread", Thread.currentThread().getName());
        report.put("startedAt", Instant.now().toString());
        report.put("serverStarted", true);
        report.put("saveCompleted", false);
        report.put("status", "failed");
        report.put("nodes", nodes);
        report.put("notTested", List.of("Visual parity", "All presets and configuration extremes", "All biomes or structures", "Multiplayer clients", "Unlisted third-party mod combinations", mode.startsWith("benchmark") ? "Performance outside this controlled benchmark" : "Controlled performance benchmark"));
        try {
            Files.createDirectories(output);
            report.put("runtime", provenance());
            node(nodes, "production-artifact", "passed", "Packaged origin and runtime recorded");
            if (raccoonman.reterraforged.world.worldgen.terrablender.TBCompat.isEnabled()) {
                Object registered = net.minecraft.core.registries.BuiltInRegistries.MATERIAL_RULE
                    .getValue(Identifier.parse("terrablender:merged"));
                Object expected = Class.forName("terrablender.worldgen.surface.NamespacedSurfaceRuleSource")
                    .getField("CODEC").get(null);
                require(registered instanceof com.mojang.serialization.MapCodec<?>,
                    "TerraBlender merged surface serializer is not the required MapCodec: " + registered);
                require(registered == expected, "TerraBlender merged surface serializer has the wrong MapCodec identity");
                report.put("terrablenderMaterialRuleCodec", Map.of("registry", "minecraft:worldgen/material_rule",
                    "id", "terrablender:merged", "actualValueClass", registered.getClass().getName(),
                    "requiredMapCodecType", true, "exactNamespacedSurfaceRuleCodecIdentity", true));
                node(nodes, "terrablender-material-rule-codec", "passed",
                    "Actual material_rule/terrablender:merged entry is the exact NamespacedSurfaceRuleSource MapCodec");
            }
            if (raccoonman.reterraforged.platform.ModLoaderUtil.isLoaded("biomesoplenty")) {
                List<String> registered = server.registryAccess().lookupOrThrow(net.minecraft.core.registries.Registries.BIOME)
                    .keySet().stream().filter(id -> id.getNamespace().equals("biomesoplenty"))
                    .map(Object::toString).sorted().toList();
                require(!registered.isEmpty(), "Loaded Biomes O' Plenty registered no biomes");
                List<String> possible = server.overworld().getChunkSource().getGenerator().getBiomeSource().possibleBiomes().stream()
                    .map(holder -> holder.unwrapKey().orElseThrow().identifier())
                    .filter(id -> id.getNamespace().equals("biomesoplenty"))
                    .map(Object::toString).sorted().toList();
                require(!possible.isEmpty(), "Loaded Biomes O' Plenty is absent from the actual overworld possible-biome set");
                int expectedRegistered = Integer.getInteger("terraforged.qa.expectedBopRegisteredCount", 68);
                int expectedPossible = Integer.getInteger("terraforged.qa.expectedBopPossibleCount", 59);
                require(registered.size() == expectedRegistered && possible.size() == expectedPossible,
                    "Pinned BOP inventory changed or was dropped: registered=" + registered.size() + "/" + expectedRegistered
                        + ", overworld possible=" + possible.size() + "/" + expectedPossible);
                report.put("biomeContentMod", Map.of("modId", "biomesoplenty", "registeredBiomeCount", registered.size(),
                    "registeredBiomeIds", registered, "overworldPossibleBiomeCount", possible.size(),
                    "overworldPossibleBiomeIds", possible, "everyBiomeReachableOrGenerated", false));
                node(nodes, "biomesoplenty-registered-biomes", "passed", registered.size()
                    + " actual registered biomes; " + possible.size() + " in overworld possible-biome set (not proof all are reachable)");
            }
            switch (mode) {
                case "export" -> exportPreset(server, output, report, nodes);
                case "generate", "reload" -> sampleWorld(server, output, mode, report, nodes);
                case "structures" -> {
                    var structures = StructureQa.verify(server, server.overworld());
                    report.put("structureChecks", structures);
                    Files.writeString(output.resolve("structure-baseline.json"), JSON.toJson(structures) + "\n", StandardCharsets.UTF_8);
                    sampleWorld(server, output, "generate", report, nodes);
                    node(nodes, "actual-structure-generation", "passed", "Native explicit placements and bounded locate generated actual starts and blocks");
                }
                case "benchmark", "benchmark-reload" -> benchmarkWorld(server, output, report, nodes);
                default -> throw new IllegalArgumentException("Explicit terraforged.qa.mode=export|generate|reload|benchmark is required");
            }
            report.put("classicBiomeChecks", ClassicBiomeQa.verify(server, server.overworld()));
            report.put("cellSamplerCacheChecks", CellSamplerCacheQa.verify(server.overworld()));
            report.put("kernelOracle", KernelOracleQa.export(server, output));
            if (Boolean.getBoolean("terraforged.qa.exportNoiseComposition")) {
                require(!mode.startsWith("benchmark"), "Noise graph capture is forbidden during benchmarks");
                Path reference = output.resolve("native-noise-composition.tsv");
                Class<?> probe = Class.forName("raccoonman.reterraforged.world.worldgen.noise.module.NativeNoiseCompositionProbe");
                probe.getMethod("main", String[].class).invoke(null, (Object) new String[] { reference.toString() });
                report.put("noiseComposition", Map.of("status", "captured-genuine-java-graphs", "file", reference.toString(),
                    "sha256", sha256(reference), "bytes", Files.size(reference)));
            }
            report.put("classicPillarCandidateSearch", ClassicPillarCandidateQa.verify(server.overworld()));
            report.put("nativeColumnReplay", NativeColumnReplayQa.verify(server.overworld()));
            report.put("classicBiomeTargetQualification", ClassicBiomeTargetSearchQa.verify(server.overworld()));
            report.put("classicBiomeStoredChunkChecks", ClassicBiomeChunkQa.verify(server, server.overworld()));
            TemplateBiomeDecoratorProbe.main(new String[0]);
            node(nodes, "biome-decorator-overrides", "passed", "Three production-helper assertions completed with real target holders");
            report.put("status", "passed");
        } catch (Throwable failure) {
            report.put("error", describe(failure));
            node(nodes, "callback", "failed", failure.toString());
        } finally {
            report.put("finishedAt", Instant.now().toString());
            boolean hold = Boolean.getBoolean("terraforged.qa.holdAfterGeneration") && "passed".equals(report.get("status"));
            report.put("cleanStopRequested", !hold);
            report.put("holdAfterGeneration", hold);
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
            if (!hold) server.halt(false);
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
        if (Boolean.getBoolean("terraforged.qa.exportAllBuiltins")) {
            exportBuiltinPresets(server, output, report, nodes);
            return;
        }
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
            String presetName = System.getProperty("terraforged.qa.preset", "legacy-default");
            var presetForExport = switch (presetName) {
                case "legacy-default" -> Presets.makeLegacyDefault();
                case "legacy-beautiful" -> Presets.makeLegacyBeautiful();
                case "legacy-huge-biomes" -> Presets.makeLegacyHugeBiomes();
                case "legacy-lite" -> Presets.makeLegacyLite();
                case "legacy-vanillaish" -> Presets.makeLegacyVanillaish();
                case "rtf-default" -> Presets.makeRTFDefault();
                default -> throw new IllegalArgumentException("Unknown QA built-in preset " + presetName);
            };
            String classicVariants = System.getProperty("terraforged.qa.classicBiomeVariants");
            if (classicVariants != null) {
                require(classicVariants.equals("true") || classicVariants.equals("false"), "Invalid explicit classic biome QA setting");
                presetForExport.miscellaneous().classicBiomeVariants = Boolean.parseBoolean(classicVariants);
            }
            String customFeatures = System.getProperty("terraforged.qa.customBiomeFeatures");
            if (customFeatures != null) {
                require(customFeatures.equals("true") || customFeatures.equals("false"), "Invalid explicit custom feature QA setting");
                presetForExport.miscellaneous().customBiomeFeatures = Boolean.parseBoolean(customFeatures);
            }
            report.put("selectedPresetSettings", Map.of("preset", presetName, "classicBiomeVariants", presetForExport.miscellaneous().classicBiomeVariants,
                "customBiomeFeatures", presetForExport.miscellaneous().customBiomeFeatures));
            Datapacks.makePreset(presetForExport, server.registryAccess(), cache, pack, "Runtime QA legacy default").run();
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

    private static void exportBuiltinPresets(MinecraftServer server, Path output, Map<String, Object> report, List<Map<String, Object>> nodes) throws Throwable {
        record ExportCase(String name, String preset, String classic, String features) {}
        var cases = List.of(new ExportCase("legacy-default", "legacy-default", null, null),
            new ExportCase("legacy-beautiful", "legacy-beautiful", null, null), new ExportCase("legacy-huge-biomes", "legacy-huge-biomes", null, null),
            new ExportCase("legacy-lite", "legacy-lite", null, null), new ExportCase("legacy-vanillaish", "legacy-vanillaish", null, null),
            new ExportCase("rtf-default", "rtf-default", null, null), new ExportCase("default-classic-off", "legacy-default", "false", "true"),
            new ExportCase("default-features-off", "legacy-default", "true", "false"), new ExportCase("default-both-off", "legacy-default", "false", "false"));
        Map<String, String> previous = new LinkedHashMap<>();
        var propertyNames = List.of("terraforged.qa.exportAllBuiltins", "terraforged.qa.preset", "terraforged.qa.classicBiomeVariants", "terraforged.qa.customBiomeFeatures");
        propertyNames.forEach(name -> previous.put(name, System.getProperty(name)));
        List<Map<String, Object>> exports = new ArrayList<>();
        report.put("builtinPresetExports", exports);
        try {
            System.setProperty("terraforged.qa.exportAllBuiltins", "false");
            for (var export : cases) {
                System.setProperty("terraforged.qa.preset", export.preset());
                if (export.classic() == null) System.clearProperty("terraforged.qa.classicBiomeVariants"); else System.setProperty("terraforged.qa.classicBiomeVariants", export.classic());
                if (export.features() == null) System.clearProperty("terraforged.qa.customBiomeFeatures"); else System.setProperty("terraforged.qa.customBiomeFeatures", export.features());
                Map<String, Object> child = new LinkedHashMap<>();
                child.put("case", export.name());
                exports.add(child);
                exportPreset(server, output.resolve(export.name()), child, nodes);
                child.put("status", "passed");
            }
        } finally {
            previous.forEach((name, value) -> { if (value == null) System.clearProperty(name); else System.setProperty(name, value); });
        }
        long files = exports.stream().mapToLong(child -> ((Number) ((Map<?, ?>) child.get("export")).get("fileCount")).longValue()).sum();
        report.put("export", Map.of("fileCount", files, "builtinPresetCount", 6, "explicitOptionCases", 3));
        node(nodes, "all-built-in-preset-exports", "passed", "All five UI presets, RTF default and three independent selection/decoration option cases exported");
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
        if (raccoonman.reterraforged.platform.ModLoaderUtil.isLoaded("biomesoplenty")) {
            var observed = new TreeMap<String, Integer>();
            for (var sample : samples) observed.merge((String) sample.get("biome"), 1, Integer::sum);
            int bopColumns = observed.entrySet().stream().filter(entry -> entry.getKey().startsWith("biomesoplenty:"))
                .mapToInt(Map.Entry::getValue).sum();
            require(bopColumns > 0, "Pinned content workload observed no actual BOP biomes; registry presence is insufficient");
            report.put("biomeContentObservations", Map.of("observedBiomeColumns", observed, "actualBopColumns", bopColumns,
                "totalColumns", samples.size(), "actualContentGenerationObserved", true));
        }
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
            Path structureBaseline = output.resolve("structure-baseline.json");
            if (Files.isRegularFile(structureBaseline)) {
                report.put("structureReloadChecks", StructureQa.verifyReload(level,
                    JsonParser.parseString(Files.readString(structureBaseline)).getAsJsonObject()));
                node(nodes, "structure-block-reload-digests", "passed", "Explicitly placed structures retain their full bounding-box block hashes");
            }
        } else {
            require(!Files.exists(baseline), "Generate requires a fresh baseline path; use reload for an existing world");
            Files.writeString(baseline, JSON.toJson(evidence) + "\n", StandardCharsets.UTF_8);
        }
        server.saveEverything(false, true, true);
        report.put("saveCompleted", true);
        node(nodes, "world-save-flush", "passed", "Actual server saveEverything completed with flush and force");
    }

    /** Fixed sequential FULL-chunk workload. Wall time includes native neighbor dependencies. */
    private static void benchmarkWorld(MinecraftServer server, Path output, Map<String, Object> report,
            List<Map<String, Object>> nodes) throws Exception {
        ServerLevel level = server.overworld();
        boolean vanilla = Boolean.getBoolean("terraforged.qa.vanillaControl");
        var state = (RTFRandomState) (Object) level.getChunkSource().randomState();
        var presets = level.registryAccess().lookupOrThrow(RTFRegistries.PRESET);
        require(presets.get(Preset.KEY).isPresent() != vanilla, "Benchmark generator/preset mismatch");
        require((state.generatorContext() == null) == vanilla, "Benchmark generator/context mismatch");
        report.put("generator", vanilla ? "vanilla-control" : "terraforged-custom");
        report.put("benchmarkEnvironment", Map.of("java", System.getProperty("java.version"),
            "vmName", System.getProperty("java.vm.name"), "vmArguments", java.lang.management.ManagementFactory.getRuntimeMXBean().getInputArguments(),
            "availableProcessors", Runtime.getRuntime().availableProcessors(), "maxHeapBytes", Runtime.getRuntime().maxMemory(),
            "terrainPoolCoreSize", raccoonman.reterraforged.concurrent.ThreadPools.WORLD_GEN.getCorePoolSize(),
            "seed", level.getSeed(), "nativeWorldHeight", List.of(level.getMinY(), level.getMaxY()),
            "terrainSettings", raccoonman.reterraforged.config.PerformanceConfig.read(
                raccoonman.reterraforged.config.PerformanceConfig.DEFAULT_FILE_PATH).result().orElseThrow().toString()));
        report.put("performanceConfigProbe", performanceConfigProbe(output));
        var phases = new ArrayList<Map<String, Object>>();
        // Far from startup spawn, disjoint regions. First request may generate a whole terrain tile.
        var first = benchmarkCoordinates(1024, 1024);
        var second = benchmarkCoordinates(2048, -2048);
        boolean reload = System.getProperty("terraforged.qa.mode").equals("benchmark-reload");
        phases.add(benchmarkPhase(level, reload ? "disk-reload" : "first-new-region", first));
        phases.add(benchmarkPhase(level, "resident-reread", first));
        if (!reload) phases.add(benchmarkPhase(level, "second-new-region", second));
        report.put("benchmarkPhases", phases);
        report.put("benchmarkScope", List.of("Each phase requests 64 FULL chunks in fixed x-major order",
            "New regions include any native neighbor generation and first terrain-tile cost",
            "Resident reread is a cache access measurement, not newly generated chunk throughput",
            "No forced GC, no JVM warmup outside recorded startup, no FPS claim"));
        Path baseline = output.resolve("benchmark-column-baseline.json");
        var observations = new TreeMap<String, Object>();
        for (var coordinate : first) {
            var chunk = level.getChunk(coordinate[0], coordinate[1]);
            int x = coordinate[0] * 16 + 7, z = coordinate[1] * 16 + 7;
            int y = chunk.getHeight(Heightmap.Types.WORLD_SURFACE, 7, 7);
            observations.put(x + "," + z, Map.of("height", y,
                "biome", level.getBiome(new BlockPos(x, y, z)).unwrapKey().orElseThrow().identifier().toString(),
                "columnSha256", columnDigest(level, x, z)));
        }
        if (reload) {
            require(Files.isRegularFile(baseline), "Benchmark reload requires its own previous world baseline");
            require(JsonParser.parseString(Files.readString(baseline)).equals(JSON.toJsonTree(observations)), "Benchmark saved columns changed on reload");
            node(nodes, "benchmark-reload-columns", "passed", "All 64 full-height column hashes, heights and biome holders match the same world's saved baseline");
        } else {
            require(!Files.exists(baseline), "Benchmark generation requires a fresh baseline");
            Files.writeString(baseline, JSON.toJson(observations) + "\n", StandardCharsets.UTF_8);
        }
        report.put("benchmarkColumnBaseline", Map.of("path", baseline.toString(), "sha256", sha256(baseline), "columnCount", observations.size()));
        // Structures are deliberately outside the timed chunk phases.
        if (raccoonman.reterraforged.platform.ModLoaderUtil.isLoaded("repurposed_structures")) {
            var structures = level.registryAccess().lookupOrThrow(net.minecraft.core.registries.Registries.STRUCTURE);
            var ids = structures.keySet().stream().filter(id -> id.getNamespace().equals("repurposed_structures"))
                .map(Object::toString).sorted().toList();
            require(!ids.isEmpty(), "Loaded Repurposed Structures registered no structures");
            report.put("structureMod", Map.of("modId", "repurposed_structures", "registeredStructureIds", ids,
                "registeredStructureCount", ids.size(), "naturalPlacementCoverage", false));
        }
        server.saveEverything(false, true, true);
        report.put("saveCompleted", true);
        node(nodes, "benchmark-full-chunks", "passed", reload
            ? "64 persisted FULL chunks and 64 resident rereads; saved full-height columns verified"
            : "128 FULL chunk requests in two new regions and 64 resident rereads; 64 full-height columns saved");
    }

    private static Map<String, Object> performanceConfigProbe(Path output) throws Exception {
        var defaults = raccoonman.reterraforged.config.PerformanceConfig.makeDefault();
        Path test = Files.createTempDirectory(output, "performance-config-probe-").resolve("settings.conf");
        var created = raccoonman.reterraforged.config.PerformanceConfig.read(test).result().orElseThrow();
        require(created.equals(defaults) && Files.isRegularFile(test), "Absent config did not create effective defaults");
        Files.writeString(test, "tileSize=2\nbatchCount=3\nthreadCount=2\n", StandardCharsets.UTF_8);
        var parsed = raccoonman.reterraforged.config.PerformanceConfig.read(test).result().orElseThrow();
        require(parsed.tileSize() == 2 && parsed.batchCount() == 3 && parsed.threadCount() == 2, "Config values ignored");
        require(raccoonman.reterraforged.config.PerformanceConfig.parse("# empty defaults\n").result().orElseThrow().equals(defaults), "Partial config fallback changed");
        int rejected = 0;
        for (String invalid : List.of("tileSize=0", "tileSize=-1", "tileSize=6", "tileSize=not-a-number",
                "batchCount=0", "batchCount=21", "threadCount=0", "threadCount=" + (raccoonman.reterraforged.config.PerformanceConfig.MAX_THREAD_COUNT + 1),
                "unknown=1")) {
            require(raccoonman.reterraforged.config.PerformanceConfig.parse(invalid).error().isPresent(), "Invalid config accepted: " + invalid);
            rejected++;
        }
        Files.writeString(test, "tileSize=0\n", StandardCharsets.UTF_8);
        require(raccoonman.reterraforged.config.PerformanceConfig.read(test).error().isPresent(), "Invalid file accepted");
        require(Files.readString(test).equals("tileSize=0\n"), "Invalid file overwritten");
        require(raccoonman.reterraforged.config.PerformanceConfig.read(test.getParent()).error().isPresent(), "Directory accepted as config");
        var pool = raccoonman.reterraforged.concurrent.ThreadPools.WORLD_GEN;
        int restore = pool.getCorePoolSize();
        var workers = java.util.concurrent.ConcurrentHashMap.<String>newKeySet();
        var entered = new java.util.concurrent.CountDownLatch(3);
        var release = new java.util.concurrent.CountDownLatch(1);
        try {
            raccoonman.reterraforged.concurrent.ThreadPools.configure(1);
            require(pool.getCorePoolSize() == 1 && pool.getMaximumPoolSize() == 1, "Pool decrease ignored");
            raccoonman.reterraforged.concurrent.ThreadPools.configure(3);
            require(pool.getCorePoolSize() == 3 && pool.getMaximumPoolSize() == 3, "Pool increase ignored");
            var futures = new ArrayList<java.util.concurrent.Future<?>>();
            for (int i = 0; i < 3; i++) futures.add(pool.submit(() -> {
                workers.add(Thread.currentThread().getName()); entered.countDown();
                try { require(release.await(5, java.util.concurrent.TimeUnit.SECONDS), "Worker release timed out"); }
                catch (InterruptedException failure) { Thread.currentThread().interrupt(); throw new RuntimeException(failure); }
            }));
            require(entered.await(5, java.util.concurrent.TimeUnit.SECONDS), "Configured pool did not execute three simultaneous workers");
            require(workers.size() == 3, "Configured worker count was not observed");
            release.countDown();
            for (var future : futures) future.get(5, java.util.concurrent.TimeUnit.SECONDS);
        } finally {
            release.countDown();
            raccoonman.reterraforged.concurrent.ThreadPools.configure(restore);
        }
        for (int invalid : new int[] {0, raccoonman.reterraforged.config.PerformanceConfig.MAX_THREAD_COUNT + 1}) {
            boolean failed = false;
            try { raccoonman.reterraforged.concurrent.ThreadPools.configure(invalid); } catch (IllegalArgumentException expected) { failed = true; }
            require(failed, "Invalid executor size accepted");
        }
        return Map.of("status", "passed", "invalidInputsRejected", rejected, "createdDefaults", true,
            "validFileSettingsApplied", true, "invalidFilePreserved", true, "actualConcurrentWorkers", workers.stream().sorted().toList(),
            "poolRestored", pool.getCorePoolSize() == restore);
    }

    private static List<int[]> benchmarkCoordinates(int x, int z) {
        var result = new ArrayList<int[]>();
        for (int dx = 0; dx < 8; dx++) for (int dz = 0; dz < 8; dz++) result.add(new int[] {x + dx, z + dz});
        return result;
    }

    private static Map<String, Object> benchmarkPhase(ServerLevel level, String name, List<int[]> coordinates) {
        var heapPools = java.lang.management.ManagementFactory.getMemoryPoolMXBeans().stream()
            .filter(pool -> pool.getType() == java.lang.management.MemoryType.HEAP).toList();
        heapPools.forEach(java.lang.management.MemoryPoolMXBean::resetPeakUsage);
        long gcCount = gcCount(), gcMillis = gcMillis();
        int loadedBefore = level.getChunkSource().getLoadedChunksCount();
        long start = System.nanoTime();
        var durations = new ArrayList<Long>();
        var biomes = new TreeMap<String, Integer>();
        for (var coordinate : coordinates) {
            long requested = System.nanoTime();
            var chunk = level.getChunk(coordinate[0], coordinate[1]);
            require(chunk.getPersistedStatus().isOrAfter(net.minecraft.world.level.chunk.status.ChunkStatus.FULL), "Benchmark chunk not FULL");
            durations.add(System.nanoTime() - requested);
            int y = chunk.getHeight(Heightmap.Types.WORLD_SURFACE, 7, 7);
            String biome = level.getBiome(new BlockPos(coordinate[0] * 16 + 7, y, coordinate[1] * 16 + 7))
                .unwrapKey().orElseThrow().identifier().toString();
            biomes.merge(biome, 1, Integer::sum);
        }
        long elapsed = System.nanoTime() - start;
        var sorted = durations.stream().sorted().toList();
        Map<String, Object> phase = new LinkedHashMap<>();
        phase.put("name", name); phase.put("requestedFullChunks", coordinates.size());
        phase.put("originChunk", List.of(coordinates.getFirst()[0], coordinates.getFirst()[1]));
        phase.put("elapsedNanos", elapsed); phase.put("requestsPerSecond", coordinates.size() * 1_000_000_000.0 / elapsed);
        double median = (sorted.get((sorted.size() - 1) / 2) + sorted.get(sorted.size() / 2)) / 2.0;
        phase.put("perRequestNanos", durations); phase.put("medianRequestNanos", median);
        phase.put("p95RequestNanos", sorted.get((int) Math.ceil(sorted.size() * 0.95) - 1));
        phase.put("loadedChunksBefore", loadedBefore); phase.put("loadedChunksAfter", level.getChunkSource().getLoadedChunksCount());
        phase.put("heapUsedBytesAfter", java.lang.management.ManagementFactory.getMemoryMXBean().getHeapMemoryUsage().getUsed());
        phase.put("heapCommittedBytesAfter", java.lang.management.ManagementFactory.getMemoryMXBean().getHeapMemoryUsage().getCommitted());
        phase.put("heapPoolPeaks", heapPools.stream().map(pool -> Map.of("pool", pool.getName(), "peakUsedBytes", pool.getPeakUsage().getUsed())).toList());
        phase.put("gcCollections", gcCount() - gcCount); phase.put("gcMillis", gcMillis() - gcMillis);
        phase.put("observedBiomes", biomes);
        return phase;
    }

    private static long gcCount() {
        return java.lang.management.ManagementFactory.getGarbageCollectorMXBeans().stream().mapToLong(bean -> Math.max(0, bean.getCollectionCount())).sum();
    }

    private static long gcMillis() {
        return java.lang.management.ManagementFactory.getGarbageCollectorMXBeans().stream().mapToLong(bean -> Math.max(0, bean.getCollectionTime())).sum();
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
