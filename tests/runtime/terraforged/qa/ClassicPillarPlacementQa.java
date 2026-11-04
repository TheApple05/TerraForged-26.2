package terraforged.qa;

import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.LinkedHashMap;
import java.util.Map;
import com.google.gson.GsonBuilder;
import com.google.gson.JsonParser;
import net.minecraft.core.BlockPos;
import net.minecraft.core.QuartPos;
import net.minecraft.tags.BlockTags;
import net.minecraft.world.level.ChunkPos;
import net.minecraft.world.level.levelgen.Heightmap;
import net.minecraft.world.level.levelgen.feature.FeaturePlaceContext;
import raccoonman.reterraforged.world.worldgen.RTFRandomState;
import raccoonman.reterraforged.world.worldgen.feature.ClassicPillarFeature;
import raccoonman.reterraforged.world.worldgen.feature.ClassicPillarMath;
import raccoonman.reterraforged.world.worldgen.noise.module.Noises;

/** Optional read-only telemetry at the exact production feature invocation. */
public final class ClassicPillarPlacementQa {
    private static final Map<String, Object> records = new LinkedHashMap<>();

    public static void before(FeaturePlaceContext<ClassicPillarFeature.Config> placement) {
        try {
            if (!wanted(placement)) return;
            boolean stone = placement.config().stoneForest();
            String id = stone ? "reterraforged:stone_forest" : "reterraforged:bryce";
            var level = placement.level();
            var context = ((RTFRandomState) (Object) level.getLevel().getChunkSource().randomState()).generatorContext();
            var chunkPos = ChunkPos.containing(placement.origin());
            var chunk = level.getChunk(chunkPos.x(), chunkPos.z());
            var tile = context.cache.provideAtChunk(chunkPos.x(), chunkPos.z()).getChunkReader(chunkPos.x(), chunkPos.z());
            int seed = context.seed.root();
            var a = ClassicPillarCandidateQa.noise(stone ? "STONE_A" : "BRYCE");
            var b = stone ? ClassicPillarCandidateQa.noise("STONE_B") : null;
            var detail = stone ? null : Noises.alpha(Noises.perlin(seed + 12341343, 4, 1), 0.05F);
            int positive = 0, positiveNatural = 0, positiveMatching = 0, positiveEmpty = 0;
            Map<String, Integer> footings = new LinkedHashMap<>();
            var examples = new ArrayList<Map<String, Object>>();
            var pos = new BlockPos.MutableBlockPos();
            for (int z = 0; z < 16; z++) for (int x = 0; x < 16; x++) {
                int worldX = chunkPos.getBlockX(x), worldZ = chunkPos.getBlockZ(z);
                int y = chunk.getHeight(Heightmap.Types.OCEAN_FLOOR, x, z);
                var cell = tile.getCell(x, z);
                float mask = ClassicPillarMath.mask(cell.biomeRegionEdge, cell.riverMask, cell.gradient);
                float shape = stone ? Math.max(ClassicPillarMath.terrace(a.compute(worldX, worldZ, seed), 1), ClassicPillarMath.terrace(b.compute(worldX, worldZ, seed), 1))
                    : ClassicPillarMath.terrace(a.compute(worldX, worldZ, seed), 0.999F) * detail.compute(worldX, worldZ, seed);
                int height = ClassicPillarMath.addedHeight(shape, mask, stone);
                if (height <= 0) continue;
                positive++;
                var base = chunk.getBlockState(pos.set(worldX, y, worldZ));
                footings.merge(base.toString(), 1, Integer::sum);
                boolean natural = base.is(BlockTags.GRASS_BLOCKS) || base.is(BlockTags.BASE_STONE_OVERWORLD) || base.is(BlockTags.DIRT) || base.is(BlockTags.SAND) || base.is(BlockTags.TERRACOTTA);
                boolean matches = level.getNoiseBiome(QuartPos.fromBlock(worldX), QuartPos.fromBlock(y), QuartPos.fromBlock(worldZ)).unwrapKey().orElseThrow().identifier().toString().equals(id);
                if (natural) positiveNatural++;
                if (natural && matches) positiveMatching++;
                boolean empty = true;
                for (int blockY = y + 1; blockY <= Math.min(level.getMaxY(), y + height); blockY++) if (!chunk.getBlockState(pos.set(worldX, blockY, worldZ)).isAir()) { empty = false; break; }
                if (natural && matches && empty) positiveEmpty++;
                if (examples.size() < 12) examples.add(Map.of("x", worldX, "z", worldZ, "currentOceanFloor", y, "base", base.toString(),
                    "predictedHeightBeforeBoundary", height, "naturalFooting", natural, "matchingBiomeAtCurrentFloor", matches, "clearAboveCurrentFloor", empty));
            }
            var report = new LinkedHashMap<String, Object>();
            report.put("biome", id); report.put("chunkX", chunkPos.x()); report.put("chunkZ", chunkPos.z());
            report.put("positiveShapeMaskColumnsBeforeBoundary", positive); report.put("positiveWithNaturalCurrentFooting", positiveNatural);
            report.put("positiveWithNaturalMatchingBiome", positiveMatching); report.put("positiveWithNaturalMatchingEmptySpace", positiveEmpty);
            report.put("currentFootingsUnderPositiveMask", footings); report.put("examples", examples);
            report.put("scope", "Read-only HEAD telemetry immediately before the real production pillar feature; no placement or registry changes. Predicted counts omit boundary taper and world-height/sea-level eligibility.");
            synchronized (records) { records.put(key(placement), report); persist(); }
            System.out.println("TERRAFORGED_PILLAR_PLACEMENT_HEAD_JSON=" + new com.google.gson.Gson().toJson(report));
        } catch (Exception failure) { throw new IllegalStateException("Pillar telemetry failed", failure); }
    }

    public static void after(FeaturePlaceContext<ClassicPillarFeature.Config> placement, boolean changed) {
        try {
            if (!wanted(placement)) return;
            synchronized (records) {
                @SuppressWarnings("unchecked") var report = (Map<String, Object>) records.get(key(placement));
                if (report == null) throw new IllegalStateException("Missing pillar HEAD telemetry");
                report.put("actualProductionFeatureReturnedChanged", changed);
                persist();
            }
            System.out.println("TERRAFORGED_PILLAR_PLACEMENT_RETURN_JSON=" + new com.google.gson.Gson().toJson(Map.of("key", key(placement), "changed", changed)));
        } catch (Exception failure) { throw new IllegalStateException("Pillar telemetry failed", failure); }
    }

    private static String key(FeaturePlaceContext<ClassicPillarFeature.Config> placement) {
        var p = ChunkPos.containing(placement.origin());
        return (placement.config().stoneForest() ? "stone_forest:" : "bryce:") + p.x() + "," + p.z();
    }

    private static boolean wanted(FeaturePlaceContext<ClassicPillarFeature.Config> placement) throws Exception {
        if (!Boolean.getBoolean("terraforged.qa.tracePillarPlacement")) return false;
        String path = System.getProperty("terraforged.qa.classicBiomeChunkTargets");
        if (path == null) return false;
        var targets = JsonParser.parseString(Files.readString(Path.of(path))).getAsJsonObject();
        var target = targets.getAsJsonObject(placement.config().stoneForest() ? "reterraforged:stone_forest" : "reterraforged:bryce");
        if (target == null) return false;
        var p = ChunkPos.containing(placement.origin());
        return (target.get("x").getAsInt() >> 4) == p.x() && (target.get("z").getAsInt() >> 4) == p.z();
    }

    private static void persist() throws Exception {
        Path output = Path.of(System.getProperty("terraforged.qa.output", "qa-output"));
        Files.createDirectories(output);
        Files.writeString(output.resolve("classic-pillar-placement-probe.json"), new GsonBuilder().setPrettyPrinting().create().toJson(records) + "\n");
    }
}
