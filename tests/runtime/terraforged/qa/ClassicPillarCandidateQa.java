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
import raccoonman.reterraforged.world.worldgen.feature.ClassicPillarFeature;
import raccoonman.reterraforged.world.worldgen.feature.ClassicPillarMath;
import raccoonman.reterraforged.world.worldgen.noise.module.Noise;
import raccoonman.reterraforged.world.worldgen.noise.module.Noises;

/** Read-only candidate planning; a forecast is never accepted as placed-block evidence. */
public final class ClassicPillarCandidateQa {
    public static Map<String, Object> verify(ServerLevel level) throws Exception {
        String originsFile = System.getProperty("terraforged.qa.classicPillarCandidateOrigins");
        if (originsFile == null) return Map.of("requested", false);
        var origins = JsonParser.parseString(Files.readString(Path.of(originsFile))).getAsJsonObject();
        var state = level.getChunkSource().randomState();
        var context = ((RTFRandomState) (Object) state).generatorContext();
        if (context == null) throw new IllegalStateException("Candidate search needs a real custom generator");
        var source = level.getChunkSource().getGenerator().getBiomeSource();
        int seed = context.seed.root();
        Noise bryce = noise("BRYCE"), stoneA = noise("STONE_A"), stoneB = noise("STONE_B");
        Noise detail = Noises.alpha(Noises.perlin(seed + 12341343, 4, 1), 0.05F);
        Map<String, Object> found = new LinkedHashMap<>();
        Map<String, Object> evidence = new LinkedHashMap<>();
        int radius = Integer.getInteger("terraforged.qa.classicPillarCandidateRadius", 1024);
        Cell cell = new Cell();
        long started = System.nanoTime();
        for (String id : new String[] {"reterraforged:bryce", "reterraforged:stone_forest"}) {
            var origin = origins.getAsJsonObject(id);
            int originX = origin.get("x").getAsInt(), originZ = origin.get("z").getAsInt();
            boolean stone = id.endsWith(":stone_forest");
            int checked = 0, naturalMatches = 0, best = 0;
            Map<String, Object> chosen = null;
            for (int dz = -radius; dz <= radius; dz += 4) for (int dx = -radius; dx <= radius; dx += 4) {
                int x = originX + dx, z = originZ + dz;
                float shape = stone ? Math.max(ClassicPillarMath.terrace(stoneA.compute(x, z, seed), 1), ClassicPillarMath.terrace(stoneB.compute(x, z, seed), 1))
                    : ClassicPillarMath.terrace(bryce.compute(x, z, seed), 0.999F) * detail.compute(x, z, seed);
                checked++;
                if (shape * (stone ? 50 : 30) <= best || shape < 0.3F) continue;
                context.lookup.getHeightmap().apply(cell.reset(), x, z, true);
                int y = Math.max(context.preset.world().properties.seaLevel, context.levels.scale(cell.height));
                if (y <= context.preset.world().properties.seaLevel) continue;
                if (!source.getNoiseBiome(QuartPos.fromBlock(x), QuartPos.fromBlock(y), QuartPos.fromBlock(z), state.sampler())
                        .unwrapKey().orElseThrow().identifier().toString().equals(id)) continue;
                naturalMatches++;
                float mask = ClassicPillarMath.mask(cell.biomeRegionEdge, cell.riverMask, cell.gradient);
                for (int distance = 4; distance <= 16; distance += 4) {
                    boolean matches = true;
                    for (int[] offset : new int[][] {{distance, 0}, {-distance, 0}, {0, distance}, {0, -distance}}) {
                        if (!source.getNoiseBiome(QuartPos.fromBlock(x + offset[0]), QuartPos.fromBlock(y), QuartPos.fromBlock(z + offset[1]), state.sampler())
                                .unwrapKey().orElseThrow().identifier().toString().equals(id)) { matches = false; break; }
                    }
                    if (!matches) { mask *= distance / 16.0F; break; }
                }
                int height = ClassicPillarMath.addedHeight(shape, mask, stone);
                if (height > best) {
                    best = height;
                    chosen = new LinkedHashMap<>();
                    chosen.put("x", x); chosen.put("y", y); chosen.put("z", z);
                    chosen.put("predictedAddedHeight", height); chosen.put("shape", shape); chosen.put("mask", mask);
                    chosen.put("regionEdge", cell.biomeRegionEdge); chosen.put("riverMask", cell.riverMask); chosen.put("gradient", cell.gradient);
                    chosen.put("terrain", cell.terrain.getName());
                }
            }
            if (chosen == null) throw new IllegalStateException("No natural positive pillar candidate near " + id);
            found.put(id, chosen);
            evidence.put(id, Map.of("originX", originX, "originZ", originZ, "shapePointsExamined", checked, "actualBiomeMatchesExamined", naturalMatches));
        }
        Path output = Path.of(System.getProperty("terraforged.qa.output", "qa-output"));
        Files.createDirectories(output);
        var gson = new GsonBuilder().setPrettyPrinting().create();
        Files.writeString(output.resolve("classic-pillar-candidates.json"), gson.toJson(found) + "\n");
        var result = new LinkedHashMap<String, Object>();
        result.put("requested", true); result.put("seed", level.getSeed()); result.put("radius", radius);
        result.put("candidates", found); result.put("search", evidence); result.put("elapsedMillis", (System.nanoTime() - started) / 1_000_000.0);
        result.put("scope", "Read-only real source/sampler, original production ridge objects and direct pre-filter heightmap mask forecast. No chunks generated, biome overrides, or block placement. Actual post-SURFACE added-block proof is required separately.");
        Files.writeString(output.resolve("classic-pillar-candidate-search.json"), gson.toJson(result) + "\n");
        return result;
    }

    static Noise noise(String name) throws Exception {
        var field = ClassicPillarFeature.class.getDeclaredField(name);
        if (!field.trySetAccessible()) throw new IllegalStateException("Production ridge is not accessible to the QA module");
        return (Noise) field.get(null);
    }
}
