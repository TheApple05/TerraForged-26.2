package terraforged.qa;

import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.security.MessageDigest;
import java.util.ArrayList;
import java.util.HexFormat;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import com.google.gson.GsonBuilder;
import com.google.gson.JsonObject;
import com.google.gson.JsonParser;
import net.minecraft.core.BlockPos;
import net.minecraft.core.QuartPos;
import net.minecraft.server.MinecraftServer;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.world.level.chunk.LevelChunk;
import net.minecraft.world.level.levelgen.Heightmap;

/** FULL stored chunk and reload checks at previously surveyed, unforced locations. */
public final class ClassicBiomeChunkQa {
    private ClassicBiomeChunkQa() {
    }

    public static Map<String, Object> verify(MinecraftServer server, ServerLevel level) throws Exception {
        String file = System.getProperty("terraforged.qa.classicBiomeChunkTargets");
        if (file == null) return Map.of("requested", false);
        Path targetsPath = Path.of(file).toAbsolutePath().normalize();
        JsonObject targets = JsonParser.parseString(Files.readString(targetsPath)).getAsJsonObject();
        var source = level.getChunkSource().getGenerator().getBiomeSource();
        var sampler = level.getChunkSource().randomState().sampler();
        Map<String, LevelChunk> chunks = new LinkedHashMap<>();
        Map<String, int[]> beforeSurface = new LinkedHashMap<>();
        Map<String, String> baselineStatuses = new LinkedHashMap<>();
        for (var entry : targets.entrySet()) {
            if (!entry.getKey().equals("reterraforged:bryce") && !entry.getKey().equals("reterraforged:stone_forest")) continue;
            var location = entry.getValue().getAsJsonObject();
            int chunkX = location.get("x").getAsInt() >> 4, chunkZ = location.get("z").getAsInt() >> 4;
            var surface = level.getChunkSource().getChunk(chunkX, chunkZ, net.minecraft.world.level.chunk.status.ChunkStatus.SURFACE, true);
            baselineStatuses.put(entry.getKey(), surface.getPersistedStatus().toString());
            if (surface.getPersistedStatus().isBefore(net.minecraft.world.level.chunk.status.ChunkStatus.FEATURES)) {
                int[] heights = new int[256];
                for (int z = 0; z < 16; z++) for (int x = 0; x < 16; x++) heights[z * 16 + x] = surface.getHeight(Heightmap.Types.OCEAN_FLOOR_WG, x, z);
                beforeSurface.put(entry.getKey(), heights);
            }
        }
        List<Map<String, Object>> observations = new ArrayList<>();
        long start = System.nanoTime();
        // Finish every requested FULL chunk before taking hashes; neighboring
        // decoration must not run after an earlier digest was recorded.
        for (var entry : targets.entrySet()) {
            var location = entry.getValue().getAsJsonObject();
            int x = location.get("x").getAsInt(), z = location.get("z").getAsInt();
            chunks.put(entry.getKey(), level.getChunk(x >> 4, z >> 4));
        }
        Map<String, String> digests = new LinkedHashMap<>();
        BlockPos.MutableBlockPos position = new BlockPos.MutableBlockPos();
        for (var entry : targets.entrySet()) {
            var location = entry.getValue().getAsJsonObject();
            int x = location.get("x").getAsInt(), y = location.get("y").getAsInt(), z = location.get("z").getAsInt();
            LevelChunk chunk = chunks.get(entry.getKey());
            int quartX = QuartPos.fromBlock(x), quartY = QuartPos.fromBlock(y), quartZ = QuartPos.fromBlock(z);
            String stored = chunk.getNoiseBiome(quartX, quartY, quartZ).unwrapKey().orElseThrow().identifier().toString();
            String queried = source.getNoiseBiome(quartX, quartY, quartZ, sampler).unwrapKey().orElseThrow().identifier().toString();
            if (!stored.equals(entry.getKey()) || !queried.equals(stored)) {
                throw new IllegalStateException("Survey/stored/source disagreement at " + entry.getKey() + " " + x + "," + y + "," + z + ": stored=" + stored + ", source=" + queried);
            }
            MessageDigest digest = MessageDigest.getInstance("SHA-256");
            for (int blockY = level.getMinY(); blockY <= level.getMaxY(); blockY++) {
                String state = chunk.getBlockState(position.set(x, blockY, z)).toString();
                digest.update((state + "\n").getBytes(StandardCharsets.UTF_8));
            }
            digests.put(entry.getKey(), HexFormat.of().formatHex(digest.digest()));
            int surfaceY = chunk.getHeight(Heightmap.Types.WORLD_SURFACE, x & 15, z & 15);
            observations.add(Map.of("biome", stored, "queriedBiome", queried, "x", x, "y", y, "z", z,
                "chunkX", x >> 4, "chunkZ", z >> 4, "surfaceY", surfaceY, "fullHeightColumnSha256", digests.get(entry.getKey())));
        }
        Path output = Path.of(System.getProperty("terraforged.qa.output", "qa-output"));
        Path baseline = output.resolve("classic-biome-chunk-baseline.json");
        boolean reload = System.getProperty("terraforged.qa.mode", "").equals("reload");
        if (reload) {
            JsonObject previous = JsonParser.parseString(Files.readString(baseline)).getAsJsonObject();
            if (previous.size() != digests.size()) throw new IllegalStateException("Classic chunk reload target count changed");
            for (var entry : digests.entrySet()) {
                if (!previous.has(entry.getKey()) || !previous.get(entry.getKey()).getAsString().equals(entry.getValue())) throw new IllegalStateException("Saved classic chunk column changed for " + entry.getKey());
            }
        } else {
            Files.createDirectories(output);
            Files.writeString(baseline, new GsonBuilder().setPrettyPrinting().create().toJson(digests) + "\n", StandardCharsets.UTF_8);
        }
        List<Map<String, Object>> pillars = new ArrayList<>();
        for (var entry : baselineStatuses.entrySet()) {
            Map<String, Object> proof = new LinkedHashMap<>();
            proof.put("biome", entry.getKey());
            proof.put("requestedBaselineStage", "minecraft:surface");
            proof.put("observedBaselineStage", entry.getValue());
            proof.put("surfaceBaselineAvailable", beforeSurface.containsKey(entry.getKey()));
            if (beforeSurface.containsKey(entry.getKey())) {
                proof.putAll(pillarEvidence(level, chunks.get(entry.getKey()), entry.getKey(), beforeSurface.get(entry.getKey())));
            }
            pillars.add(proof);
        }
        server.saveEverything(false, true, true);
        Map<String, Object> result = new LinkedHashMap<>();
        result.put("requested", true);
        result.put("targetsPath", targetsPath.toString());
        result.put("targetsSha256", HexFormat.of().formatHex(MessageDigest.getInstance("SHA-256").digest(Files.readAllBytes(targetsPath))));
        result.put("fullChunkTargets", targets.size());
        result.put("observations", observations);
        result.put("pillarSurfaceEvidence", pillars);
        result.put("columnMinY", level.getMinY());
        result.put("columnMaxYInclusive", level.getMaxY());
        result.put("elapsedMillis", (System.nanoTime() - start) / 1_000_000.0);
        result.put("baselinePath", baseline.toAbsolutePath().toString());
        result.put("reloadDigestsMatched", reload);
        result.put("nativeSaveEverythingFlushCompleted", true);
        Files.writeString(output.resolve("classic-biome-stored-chunk-checks.json"), new GsonBuilder().setPrettyPrinting().create().toJson(result) + "\n", StandardCharsets.UTF_8);
        if (Boolean.getBoolean("terraforged.qa.requirePillarEvidence")) {
            for (var proof : pillars) if (Boolean.TRUE.equals(proof.get("surfaceBaselineAvailable")) && ((Number) proof.get("positiveAddedPillarColumns")).intValue() == 0)
                throw new IllegalStateException("No positive pillars in surveyed target chunk for " + proof.get("biome") + "; complete stored-chunk diagnostics retained");
        }
        return result;
    }
    private static Map<String, Object> pillarEvidence(ServerLevel level, LevelChunk chunk, String expectedBiome, int[] before) throws Exception {
        boolean stoneForest = expectedBiome.endsWith(":stone_forest");
        java.util.Set<net.minecraft.world.level.block.Block> palette = stoneForest
            ? java.util.Set.of(net.minecraft.world.level.block.Blocks.STONE, net.minecraft.world.level.block.Blocks.DIRT, net.minecraft.world.level.block.Blocks.GRASS_BLOCK)
            : java.util.Set.of(net.minecraft.world.level.block.Blocks.TERRACOTTA, net.minecraft.world.level.block.Blocks.DYED_TERRACOTTA.orange(),
                net.minecraft.world.level.block.Blocks.DYED_TERRACOTTA.white(), net.minecraft.world.level.block.Blocks.DYED_TERRACOTTA.brown(),
                net.minecraft.world.level.block.Blocks.DYED_TERRACOTTA.yellow(), net.minecraft.world.level.block.Blocks.DYED_TERRACOTTA.lightGray());
        var blocks = net.minecraft.core.registries.BuiltInRegistries.BLOCK;
        int maximum = stoneForest ? 50 : 30;
        int positive = 0, capped = 0, maximumObserved = 0;
        int rawAddedColumns = 0, wrongCapColumns = 0;
        Map<String, Integer> materials = new LinkedHashMap<>();
        List<Map<String, Object>> examples = new ArrayList<>();
        List<Map<String, Object>> diagnostics = new ArrayList<>();
        var context = ((raccoonman.reterraforged.world.worldgen.RTFRandomState) (Object) level.getChunkSource().randomState()).generatorContext();
        var tile = context.cache.provideAtChunk(chunk.getPos().x(), chunk.getPos().z()).getChunkReader(chunk.getPos().x(), chunk.getPos().z());
        var ridgeA = ClassicPillarCandidateQa.noise(stoneForest ? "STONE_A" : "BRYCE");
        var ridgeB = stoneForest ? ClassicPillarCandidateQa.noise("STONE_B") : null;
        int seed = context.seed.root();
        var detail = stoneForest ? null : raccoonman.reterraforged.world.worldgen.noise.module.Noises.alpha(raccoonman.reterraforged.world.worldgen.noise.module.Noises.perlin(seed + 12341343, 4, 1), 0.05F);
        int predictedPositive = 0, predictedMaximum = 0;
        BlockPos.MutableBlockPos position = new BlockPos.MutableBlockPos();
        for (int z = 0; z < 16; z++) for (int x = 0; x < 16; x++) {
            int baseY = before[z * 16 + x];
            int worldX = chunk.getPos().getMinBlockX() + x, worldZ = chunk.getPos().getMinBlockZ() + z;
            String biome = chunk.getNoiseBiome(QuartPos.fromBlock(worldX), QuartPos.fromBlock(baseY), QuartPos.fromBlock(worldZ)).unwrapKey().orElseThrow().identifier().toString();
            if (!biome.equals(expectedBiome)) continue;
            var cell = tile.getCell(x, z);
            float mask = raccoonman.reterraforged.world.worldgen.feature.ClassicPillarMath.mask(cell.biomeRegionEdge, cell.riverMask, cell.gradient);
            float shape = stoneForest
                ? Math.max(raccoonman.reterraforged.world.worldgen.feature.ClassicPillarMath.terrace(ridgeA.compute(worldX, worldZ, seed), 1), raccoonman.reterraforged.world.worldgen.feature.ClassicPillarMath.terrace(ridgeB.compute(worldX, worldZ, seed), 1))
                : raccoonman.reterraforged.world.worldgen.feature.ClassicPillarMath.terrace(ridgeA.compute(worldX, worldZ, seed), 0.999F) * detail.compute(worldX, worldZ, seed);
            int predicted = raccoonman.reterraforged.world.worldgen.feature.ClassicPillarMath.addedHeight(shape, mask, stoneForest);
            if (predicted > 0) predictedPositive++;
            predictedMaximum = Math.max(predictedMaximum, predicted);
            int height = 0;
            while (baseY + height + 1 <= level.getMaxY() && palette.contains(chunk.getBlockState(position.set(worldX, baseY + height + 1, worldZ)).getBlock())) {
                height++;
            }
            if (diagnostics.size() < 12 && (x == 12 || x == 0) && z % 4 == 0) {
                Map<String,Object> diagnostic = new LinkedHashMap<>();
                diagnostic.put("x", worldX); diagnostic.put("z", worldZ); diagnostic.put("surfaceBeforeFeatures", baseY); diagnostic.put("paletteAddedHeight", height);
                diagnostic.put("oceanFloorAfter", chunk.getHeight(Heightmap.Types.OCEAN_FLOOR, x, z));
                diagnostic.put("stateAtBase", chunk.getBlockState(position.set(worldX, baseY, worldZ)).toString());
                diagnostic.put("stateAboveBase", chunk.getBlockState(position.set(worldX, baseY + 1, worldZ)).toString());
                diagnostic.put("actualFilteredTileMask", mask); diagnostic.put("actualFilteredTileRegionEdge", cell.biomeRegionEdge);
                diagnostic.put("actualFilteredTileRiverMask", cell.riverMask); diagnostic.put("actualFilteredTileGradient", cell.gradient);
                diagnostic.put("shape", shape); diagnostic.put("predictedAddedHeightBeforeBoundary", predicted);
                diagnostics.add(diagnostic);
            }
            if (height == 0) continue;
            rawAddedColumns++;
            var top = chunk.getBlockState(position.set(worldX, baseY + height, worldZ));
            boolean cappedCorrectly = !stoneForest || (top.is(net.minecraft.world.level.block.Blocks.GRASS_BLOCK)
                && (height == 1 || chunk.getBlockState(position.set(worldX, baseY + height - 1, worldZ)).is(net.minecraft.world.level.block.Blocks.DIRT)));
            if (!cappedCorrectly) { wrongCapColumns++; continue; }
            if (height > maximum) throw new IllegalStateException("Classic pillar exceeds its height cap: " + expectedBiome + " " + height);
            positive++;
            if (cappedCorrectly) capped++;
            maximumObserved = Math.max(maximumObserved, height);
            for (int y = baseY + 1; y <= baseY + height; y++) materials.merge(blocks.getKey(chunk.getBlockState(position.set(worldX, y, worldZ)).getBlock()).toString(), 1, Integer::sum);
            if (examples.size() < 12) examples.add(Map.of("x", worldX, "z", worldZ, "surfaceBeforeFeatures", baseY, "addedHeight", height, "top", blocks.getKey(top.getBlock()).toString()));
        }
        Map<String, Object> evidence = new LinkedHashMap<>();
        evidence.put("positiveAddedPillarColumns", positive); evidence.put("correctlyCappedColumns", capped); evidence.put("maximumAddedHeight", maximumObserved);
        evidence.put("heightCap", maximum); evidence.put("actualAddedMaterialCounts", materials); evidence.put("examples", examples);
        evidence.put("rawPaletteAddedColumns", rawAddedColumns); evidence.put("wrongCapColumns", wrongCapColumns); evidence.put("diagnosticColumns", diagnostics);
        evidence.put("actualFilteredTilePredictedPositiveBeforeBoundary", predictedPositive); evidence.put("actualFilteredTilePredictedMaximumBeforeBoundary", predictedMaximum);
        evidence.put("scope", "Actual palette-only columns added after SURFACE, with current stored target biome; not a visual or original-world identity verdict");
        return evidence;
    }

}
