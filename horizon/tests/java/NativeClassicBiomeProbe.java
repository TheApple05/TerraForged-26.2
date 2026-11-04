import com.google.gson.*;
import java.io.*;
import java.nio.charset.StandardCharsets;
import java.nio.file.*;
import java.util.*;
import net.minecraft.core.registries.Registries;
import net.minecraft.resources.*;
import net.minecraft.world.level.biome.Biome;
import raccoonman.reterraforged.world.worldgen.biome.ClassicBiomeSource;
import raccoonman.reterraforged.world.worldgen.cell.Cell;
import raccoonman.reterraforged.world.worldgen.cell.terrain.*;

public final class NativeClassicBiomeProbe {
    static float value(JsonElement e) { return Float.intBitsToFloat((int)Long.parseUnsignedLong(e.getAsString(), 16)); }
    public static void main(String[] args) throws Exception {
        net.minecraft.SharedConstants.tryDetectVersion();
        net.minecraft.server.Bootstrap.bootStrap();
        Path jar = Path.of(args[1]).toRealPath();
        for (Class<?> type : List.of(ClassicBiomeSource.class, Cell.class, TerrainType.class)) {
            Path origin = Path.of(type.getProtectionDomain().getCodeSource().getLocation().toURI()).toRealPath();
            if (!origin.equals(jar)) throw new IllegalStateException("Wrong packaged origin for " + type);
            System.err.println("PACKAGED_CLASS_ORIGIN=" + type.getName() + "=" + origin);
        }
        JsonObject input = JsonParser.parseString(Files.readString(Path.of(args[0]))).getAsJsonObject();
        TerrainType.getOrCreate("native_clone_lake", TerrainType.LAKE);
        TerrainType.getOrCreate("native_clone_wetland", TerrainType.WETLAND);
        TerrainType.getOrCreate("native_clone_beach", TerrainType.BEACH);
        long records = 0;
        Set<String> variants = new TreeSet<>();
        // A file keeps Minecraft's bootstrap logging out of the numerical stream.
        try (BufferedWriter out = Files.newBufferedWriter(Path.of(args[2]), StandardCharsets.UTF_8, StandardOpenOption.CREATE_NEW)) {
            int index = 0;
            for (JsonElement raw : input.getAsJsonArray("hashBits")) {
                int hash = Float.floatToRawIntBits(ClassicBiomeSource.regionChoice(value(raw)));
                out.write("hash/" + index++ + "\t" + String.format(Locale.ROOT, "%08x", hash) + "\n");
                records++;
            }
            int p = 0;
            for (JsonElement name : input.getAsJsonArray("parents")) {
                ResourceKey<Biome> parent = ResourceKey.create(Registries.BIOME, Identifier.parse(name.getAsString()));
                int t = 0;
                for (JsonElement terrain : input.getAsJsonArray("terrains")) {
                    Cell cell = new Cell(); cell.terrain = Objects.requireNonNull(TerrainType.get(terrain.getAsString()));
                    int r = 0;
                    for (JsonElement region : input.getAsJsonArray("regionBits")) {
                        cell.biomeRegionId = value(region); int k = 0;
                        for (JsonElement temperature : input.getAsJsonArray("temperatureBits")) {
                            cell.regionTemperature = value(temperature);
                            String result = ClassicBiomeSource.selectKey(parent, cell).identifier().toString();
                            if (result.startsWith("reterraforged:")) variants.add(result);
                            out.write("select/" + p + "/" + t + "/" + r + "/" + k++ + "\t" + result + "\n"); records++;
                        }
                        r++;
                    }
                    t++;
                }
                p++;
            }
        }
        if (variants.size() != 17) throw new IllegalStateException("Missing variant paths: " + variants);
        System.err.println("CLASSIC_BIOME_CAPTURE_OK records=" + records + " variants=" + variants.size() + " pid=" + ProcessHandle.current().pid());
    }
}
