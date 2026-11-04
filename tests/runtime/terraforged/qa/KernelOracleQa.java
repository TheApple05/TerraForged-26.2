package terraforged.qa;

import java.io.BufferedOutputStream;
import java.io.DataOutputStream;
import java.lang.reflect.Field;
import java.lang.reflect.Modifier;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.security.MessageDigest;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.Comparator;
import java.util.HexFormat;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.TreeMap;
import java.util.concurrent.TimeUnit;
import java.util.zip.GZIPOutputStream;

import com.google.gson.GsonBuilder;
import com.mojang.serialization.JsonOps;
import net.minecraft.resources.RegistryOps;
import net.minecraft.server.MinecraftServer;
import raccoonman.reterraforged.data.worldgen.preset.settings.Preset;
import raccoonman.reterraforged.registries.RTFRegistries;
import raccoonman.reterraforged.world.worldgen.GeneratorContext;
import raccoonman.reterraforged.world.worldgen.RTFRandomState;
import raccoonman.reterraforged.world.worldgen.cell.Cell;
import raccoonman.reterraforged.world.worldgen.cell.terrain.Terrain;
import raccoonman.reterraforged.world.worldgen.noise.module.Noise;

/** Independent Java-only kernel corpus. Never reads native output or alters production fields. */
public final class KernelOracleQa {
    private static final long[] SEEDS = {0L, -1L, Integer.MIN_VALUE, Integer.MAX_VALUE, 8675309L, 8675309L + (1L << 32)};
    private static final int[] COORDINATES = {-129, -128, -17, -16, -1, 0, 15, 16, 127, 128, 129};
    private static final Field[] FIELDS = Arrays.stream(Cell.class.getFields())
        .filter(field -> !Modifier.isStatic(field.getModifiers()))
        .sorted(Comparator.comparing(Field::getName)).toArray(Field[]::new);

    private KernelOracleQa() {}

    public static Map<String, Object> export(MinecraftServer server, Path qaOutput) throws Exception {
        if (!Boolean.getBoolean("terraforged.qa.exportKernelOracle")) return Map.of("requested", false);
        if (System.getProperty("terraforged.qa.mode", "").startsWith("benchmark"))
            throw new IllegalStateException("Kernel capture is forbidden during benchmarks");
        var state = (RTFRandomState) (Object) server.overworld().getChunkSource().randomState();
        var active = state.generatorContext();
        if (active == null) throw new IllegalStateException("A genuine active custom preset is required");
        Path output = qaOutput.resolve("kernel-oracle-v1");
        Files.createDirectory(output);
        var ops = RegistryOps.create(JsonOps.INSTANCE, server.registryAccess());
        String preset = Preset.DIRECT_CODEC.encodeStart(ops, active.preset).getOrThrow().toString();
        Path presetFile = output.resolve("preset.json");
        Files.writeString(presetFile, preset + "\n", StandardCharsets.UTF_8);
        var registry = server.registryAccess().lookupOrThrow(RTFRegistries.NOISE);
        var noises = new TreeMap<String, String>();
        for (var entry : registry.entrySet()) {
            String name = entry.getKey().identifier().toString();
            noises.put(name, Noise.DIRECT_CODEC.encodeStart(ops, entry.getValue()).getOrThrow().toString());
        }
        var json = new GsonBuilder().setPrettyPrinting().create();
        Path noiseFile = output.resolve("noise-registry.json");
        Files.writeString(noiseFile, json.toJson(noises) + "\n", StandardCharsets.UTF_8);
        List<Map<String, Object>> captures = new ArrayList<>();
        for (int index = 0; index < SEEDS.length; index++) {
            long worldSeed = SEEDS[index];
            String[] repeatDigests = new String[2];
            for (int repeat = 0; repeat < 2; repeat++) {
                Path file = output.resolve("seed-" + index + "-repeat-" + repeat + ".bin.gz");
                long records = 0;
                try (var context = GeneratorContext.makeCached(active.preset.copy(), registry, (int) worldSeed, 3, 6, false);
                     var stream = new DataOutputStream(new BufferedOutputStream(new GZIPOutputStream(Files.newOutputStream(file))))) {
                    stream.writeUTF("TerraForged kernel oracle v1");
                    stream.writeLong(worldSeed);
                    stream.writeInt((int) worldSeed);
                    stream.writeInt(FIELDS.length);
                    for (Field field : FIELDS) { stream.writeUTF(field.getName()); stream.writeUTF(field.getType().getName()); }
                    var heightmap = context.generator.getHeightmap();
                    for (int x : COORDINATES) for (int z : COORDINATES) {
                        Cell cell = new Cell();
                        heightmap.applyTerrain(cell, x, z);
                        writeCell(stream, "terrain", x, z, cell); records++;
                        heightmap.applyRivers(cell, x, z, heightmap.continent().getRivermap(cell));
                        writeCell(stream, "rivers", x, z, cell); records++;
                        Cell withoutClimate = new Cell(); withoutClimate.copyFrom(cell);
                        heightmap.applyClimate(cell, x, z, true);
                        writeCell(stream, "climate-true", x, z, cell); records++;
                        heightmap.applyClimate(withoutClimate, x, z, false);
                        writeCell(stream, "climate-false", x, z, withoutClimate); records++;
                    }
                    // Actual production TileGenerator, including its real configured halo
                    // and complete erosion/smoothing/required-filter sequence.
                    for (int[] position : new int[][] {{-1,-1},{0,0}}) {
                        try (var tile = context.generator.generate(position[0], position[1]).get(90, TimeUnit.SECONDS)) {
                            var size = tile.getBlockSize();
                            stream.writeUTF("tile-layout");
                            stream.writeInt(position[0]); stream.writeInt(position[1]);
                            stream.writeInt(size.size()); stream.writeInt(size.border()); stream.writeInt(size.total());
                            for (int z = 0; z < size.total(); z++) for (int x = 0; x < size.total(); x++) {
                                int worldX = tile.getBlockX() - size.border() + x;
                                int worldZ = tile.getBlockZ() - size.border() + z;
                                Cell cell = tile.getCellRaw(x, z);
                                if (cell.isAbsent()) throw new IllegalStateException("Actual tile omitted a backing cell");
                                writeCell(stream, "filtered-tile", worldX, worldZ, cell); records++;
                            }
                        }
                    }
                    stream.writeUTF("complete"); stream.writeLong(records);
                }
                repeatDigests[repeat] = sha(file);
                var capture = new LinkedHashMap<String, Object>();
                capture.put("worldSeed", worldSeed); capture.put("narrowedKernelSeed", (int) worldSeed);
                capture.put("repeat", repeat); capture.put("file", file.getFileName().toString());
                capture.put("sha256", repeatDigests[repeat]); capture.put("bytes", Files.size(file));
                capture.put("cellRecords", records); capture.put("tileSizeFactor", 3); capture.put("batchCount", 6);
                captures.add(capture);
            }
            if (!repeatDigests[0].equals(repeatDigests[1]))
                throw new IllegalStateException("Fresh Java kernel contexts differ for seed " + worldSeed);
        }
        Map<String, Object> result = new LinkedHashMap<>();
        result.put("schemaVersion", 1); result.put("requested", true); result.put("status", "passed-java-reference-capture-only");
        result.put("productionJarSha256", System.getProperty("terraforged.qa.expectedModSha256"));
        result.put("presetSha256", sha(presetFile)); result.put("noiseRegistrySha256", sha(noiseFile));
        result.put("noiseRegistryEntries", noises.size()); result.put("fieldNames", Arrays.stream(FIELDS).map(Field::getName).toList());
        result.put("captures", captures); result.put("freshContextsAgree", true);
        result.put("scope", "Actual terrain/rivers/climate points and complete production-filtered tile backings/halos; no native output used");
        result.put("notCovered", List.of("Native kernel implementation", "Every landscape/seed/preset", "Intermediate individual filter snapshots", "Game chunks or native client"));
        Files.writeString(output.resolve("manifest.json"), json.toJson(result) + "\n", StandardCharsets.UTF_8);
        return result;
    }

    private static void writeCell(DataOutputStream stream, String stage, int x, int z, Cell cell) throws Exception {
        stream.writeUTF(stage); stream.writeInt(x); stream.writeInt(z);
        for (Field field : FIELDS) {
            if (field.getType() == float.class) {
                float value = field.getFloat(cell);
                if (!Float.isFinite(value)) throw new IllegalStateException("Nonfinite field " + field.getName());
                stream.writeInt(Float.floatToRawIntBits(value));
            } else if (field.getType() == int.class) stream.writeInt(field.getInt(cell));
            else if (field.getType() == boolean.class) stream.writeBoolean(field.getBoolean(cell));
            else if (field.get(cell) instanceof Terrain terrain) stream.writeUTF(terrain.getName());
            else if (field.get(cell) instanceof Enum<?> value) stream.writeUTF(value.name());
            else throw new IllegalStateException("Unsupported Cell field " + field.getName());
        }
    }

    private static String sha(Path path) throws Exception {
        MessageDigest digest = MessageDigest.getInstance("SHA-256");
        try (var input = Files.newInputStream(path)) {
            byte[] buffer = new byte[65536]; int count;
            while ((count = input.read(buffer)) >= 0) digest.update(buffer, 0, count);
        }
        return HexFormat.of().formatHex(digest.digest());
    }
}
