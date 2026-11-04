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

import net.minecraft.core.BlockPos;
import net.minecraft.core.Holder;
import net.minecraft.core.HolderSet;
import net.minecraft.core.SectionPos;
import net.minecraft.core.registries.Registries;
import net.minecraft.resources.Identifier;
import net.minecraft.server.MinecraftServer;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.util.RandomSource;
import net.minecraft.world.level.ChunkPos;
import net.minecraft.world.level.levelgen.structure.BoundingBox;
import net.minecraft.world.level.levelgen.structure.Structure;
import net.minecraft.world.level.levelgen.structure.StructureStart;

/** Actual native structure generation/placement, kept outside production and timed benchmark phases. */
public final class StructureQa {
    public static Map<String, Object> verifyReload(ServerLevel level, com.google.gson.JsonObject baseline) throws Exception {
        var checked = new ArrayList<Map<String, Object>>();
        for (var item : baseline.getAsJsonArray("explicitPlacements")) {
            var placement = item.getAsJsonObject();
            var bounds = placement.getAsJsonArray("boundingBox");
            var box = new BoundingBox(bounds.get(0).getAsInt(), bounds.get(1).getAsInt(), bounds.get(2).getAsInt(),
                bounds.get(3).getAsInt(), bounds.get(4).getAsInt(), bounds.get(5).getAsInt());
            String actual = digest(level, box);
            require(placement.get("afterSha256").getAsString().equals(actual), "Placed structure blocks changed on reload: " + placement.get("id"));
            checked.add(Map.of("id", placement.get("id").getAsString(), "sha256", actual));
        }
        require(!checked.isEmpty(), "No placed structures in reload baseline");
        return Map.of("status", "passed", "savedStructureBlockDigests", checked);
    }

    public static Map<String, Object> verify(MinecraftServer server, ServerLevel level) throws Exception {
        var registry = level.registryAccess().lookupOrThrow(Registries.STRUCTURE);
        var generator = level.getChunkSource().getGenerator();
        var placements = new ArrayList<Map<String, Object>>();
        var attempts = new ArrayList<Map<String, Object>>();
        Path attemptsFile = Path.of(System.getProperty("terraforged.qa.output", "qa-output")).resolve("structure-site-attempts.json");
        Files.createDirectories(attemptsFile.toAbsolutePath().getParent());
        var ids = new ArrayList<>(List.of("minecraft:desert_pyramid", "minecraft:swamp_hut"));
        boolean repurposed = raccoonman.reterraforged.platform.ModLoaderUtil.isLoaded("repurposed_structures");
        if (repurposed) ids.addAll(List.of("repurposed_structures:igloo_grassy", "repurposed_structures:witch_hut_oak"));
        int index = 0;
        for (String id : ids) {
            Holder.Reference<Structure> holder = registry.get(Identifier.parse(id)).orElseThrow();
            long startTime = System.nanoTime();
            StructureStart start = null;
            int[][] offsets = {{0, 0}, {0, 128}, {128, 0}, {-128, 0}, {0, -128}, {256, 256},
                {-256, 256}, {256, -256}, {-256, -256}, {512, 0}, {0, 512}, {-512, 0}, {0, -512},
                {1024, 1024}, {-1024, 1024}, {1024, -1024}};
            for (int[] offset : offsets) {
                ChunkPos origin = new ChunkPos(128 + index * 32 + offset[0], -128 - index * 32 + offset[1]);
                Map<String, Object> attempt = new LinkedHashMap<>();
                attempt.put("id", id); attempt.put("chunk", List.of(origin.x(), origin.z())); attempt.put("status", "started");
                attempts.add(attempt);
                Files.writeString(attemptsFile, new com.google.gson.GsonBuilder().setPrettyPrinting().create().toJson(attempts) + "\n");
                long attemptedAt = System.nanoTime();
                StructureStart candidate = holder.value().generate(holder, level.dimension(), level.registryAccess(), generator,
                    generator.getBiomeSource(), level.getChunkSource().randomState(), level.getStructureManager(),
                    level.getSeed(), origin, 0, level, biome -> true);
                boolean valid = candidate.isValid() && !candidate.getPieces().isEmpty();
                attempt.put("status", valid ? "valid-native-start" : "native-site-rejected");
                attempt.put("elapsedMillis", (System.nanoTime() - attemptedAt) / 1_000_000.0);
                if (!valid) attempt.put("reason", "Native geometry/height/site rules rejected this explicit location despite bypassed biome eligibility");
                Files.writeString(attemptsFile, new com.google.gson.GsonBuilder().setPrettyPrinting().create().toJson(attempts) + "\n");
                if (valid) { start = candidate; break; }
            }
            require(start != null, "No candidate location produced valid native pieces: " + id + "; attempted sites retained at " + attemptsFile);
            require(start.isValid() && !start.getPieces().isEmpty(), "Native structure produced no valid pieces: " + id);
            BoundingBox box = start.getBoundingBox();
            long volume = (long) box.getXSpan() * box.getYSpan() * box.getZSpan();
            require(volume > 0 && volume < 4_000_000, "Unexpected structure volume: " + id + " " + volume);
            var chunks = ChunkPos.rangeClosed(new ChunkPos(SectionPos.blockToSectionCoord(box.minX()), SectionPos.blockToSectionCoord(box.minZ())),
                new ChunkPos(SectionPos.blockToSectionCoord(box.maxX()), SectionPos.blockToSectionCoord(box.maxZ()))).toList();
            for (var chunk : chunks) level.getChunk(chunk.x(), chunk.z());
            String before = digest(level, box);
            for (var chunk : chunks) start.placeInChunk(level, level.structureManager(), generator,
                RandomSource.create(level.getSeed() + index),
                new BoundingBox(chunk.getMinBlockX(), level.getMinY(), chunk.getMinBlockZ(), chunk.getMaxBlockX(), level.getMaxY() + 1, chunk.getMaxBlockZ()), chunk);
            String after = digest(level, box);
            require(!before.equals(after), "Native structure changed no blocks: " + id);
            placements.add(Map.of("id", id, "pieces", start.getPieces().size(), "boundingBox",
                List.of(box.minX(), box.minY(), box.minZ(), box.maxX(), box.maxY(), box.maxZ()),
                "fullChunksLoaded", chunks.size(), "beforeSha256", before, "afterSha256", after,
                "changedBlocksObserved", true, "elapsedMillis", (System.nanoTime() - startTime) / 1_000_000,
                "biomeEligibilityBypassedForExplicitPlacement", true));
            index++;
        }
        var locate = new ArrayList<Map<String, Object>>();
        for (String namespace : repurposed ? List.of("minecraft", "repurposed_structures") : List.of("minecraft")) {
            List<Holder<Structure>> wanted = registry.listElements().filter(holder -> {
                Identifier id = holder.key().identifier();
                return id.getNamespace().equals(namespace) && (namespace.equals("minecraft")
                    ? id.getPath().equals("mineshaft") : id.getPath().startsWith("mineshaft_")
                        && !List.of("mineshaft_basalt", "mineshaft_crimson", "mineshaft_end", "mineshaft_nether", "mineshaft_warped").contains(id.getPath()));
            }).map(holder -> (Holder<Structure>) holder).toList();
            require(!wanted.isEmpty(), "Locate target is absent: " + namespace);
            long time = System.nanoTime();
            var found = generator.findNearestMapStructure(level, HolderSet.direct(wanted), new BlockPos(0, 80, 0), 8, false);
            require(found != null, "Bounded native locate found no " + namespace + " mineshaft within eight placement regions");
            BlockPos pos = found.getFirst();
            var chunk = level.getChunk(SectionPos.blockToSectionCoord(pos.getX()), SectionPos.blockToSectionCoord(pos.getZ()));
            var structureStart = chunk.getStartForStructure(found.getSecond().value());
            require(structureStart != null && structureStart.isValid(), "Located structure did not survive actual FULL generation");
            locate.add(Map.of("id", found.getSecond().unwrapKey().orElseThrow().identifier().toString(),
                "position", List.of(pos.getX(), pos.getY(), pos.getZ()), "radiusPlacementRegions", 8,
                "actualFullChunk", chunk.getPos().toString(), "validNaturalStart", true,
                "pieces", structureStart.getPieces().size(), "elapsedMillis", (System.nanoTime() - time) / 1_000_000));
        }
        Map<String, Object> result = new LinkedHashMap<>();
        result.put("status", "passed"); result.put("explicitPlacements", placements); result.put("nativeLocateAndNaturalStarts", locate);
        result.put("explicitSiteAttempts", attempts); result.put("siteAttemptsFile", attemptsFile.toAbsolutePath().toString());
        result.put("scope", "Two vanilla plus two optional Repurposed Structures explicit placements; one native mineshaft locate/generation per namespace, not all structures");
        return result;
    }

    private static String digest(ServerLevel level, BoundingBox box) throws Exception {
        var hash = MessageDigest.getInstance("SHA-256");
        var pos = new BlockPos.MutableBlockPos();
        for (int y = Math.max(level.getMinY(), box.minY()); y <= Math.min(level.getMaxY(), box.maxY()); y++)
            for (int x = box.minX(); x <= box.maxX(); x++) for (int z = box.minZ(); z <= box.maxZ(); z++) {
                pos.set(x, y, z); hash.update((level.getBlockState(pos).toString() + "\n").getBytes(StandardCharsets.UTF_8));
            }
        return HexFormat.of().formatHex(hash.digest());
    }

    private static void require(boolean condition, String message) {
        if (!condition) throw new IllegalStateException(message);
    }
}
