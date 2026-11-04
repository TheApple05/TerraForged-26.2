package terraforged.qa;

import java.lang.ref.WeakReference;
import java.lang.reflect.Modifier;
import java.net.URL;
import java.net.URLClassLoader;
import java.nio.file.Files;
import java.nio.file.Path;
import java.security.MessageDigest;
import java.util.ArrayList;
import java.util.HexFormat;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.TreeMap;

import net.minecraft.server.level.ServerLevel;
import net.minecraft.world.level.levelgen.DensityFunction;
import raccoonman.reterraforged.concurrent.cache.CacheManager;
import raccoonman.reterraforged.registries.RTFRegistries;
import raccoonman.reterraforged.world.worldgen.GeneratorContext;
import raccoonman.reterraforged.world.worldgen.RTFRandomState;
import raccoonman.reterraforged.world.worldgen.cell.Cell;
import raccoonman.reterraforged.world.worldgen.cell.heightmap.WorldLookup;
import raccoonman.reterraforged.world.worldgen.densityfunction.CellSampler;

/** Real old packaged cache class and genuine target generator contexts; no game stubs. */
public final class CellSamplerCacheQa {
    private static final String OLD_CACHE = "raccoonman.reterraforged.world.worldgen.densityfunction.CellSampler$Cache2d";
    private static int assertions;

    public static Map<String, Object> verify(ServerLevel level) throws Exception {
        String reference = System.getProperty("terraforged.qa.previousJar");
        if (reference == null) return Map.of("requested", false);
        require(!System.getProperty("terraforged.qa.mode", "").startsWith("benchmark"), "GC/lifecycle probe must not run during a benchmark");
        assertions = 0;
        var state = (RTFRandomState) (Object) level.getChunkSource().randomState();
        var actual = state.generatorContext();
        require(actual != null, "Cache probe requires an active genuine custom world");
        Path jar = Path.of(reference).toAbsolutePath().normalize();
        require(Files.isRegularFile(jar), "Old packaged reference is absent");
        String jarSha = HexFormat.of().formatHex(MessageDigest.getInstance("SHA-256").digest(Files.readAllBytes(jar)));
        require(jarSha.equalsIgnoreCase(System.getProperty("terraforged.qa.previousJarSha256", "")), "Old reference JAR hash differs");
        int ownersBefore = CacheManager.activeCacheCount();
        var noises = level.registryAccess().lookupOrThrow(RTFRegistries.NOISE);
        int seed = (int) level.getSeed();
        try (GeneratorContext first = GeneratorContext.makeCached(actual.preset, noises, seed, 1, 1, false);
             GeneratorContext second = GeneratorContext.makeCached(actual.preset, noises, seed + 104729, 1, 1, false);
             URLClassLoader oldLoader = new URLClassLoader(new URL[] {jar.toUri().toURL()}, WorldLookup.class.getClassLoader()) {
                 @Override protected Class<?> loadClass(String name, boolean resolve) throws ClassNotFoundException {
                     if (!name.equals(OLD_CACHE)) return super.loadClass(name, resolve);
                     synchronized (getClassLoadingLock(name)) {
                         Class<?> loaded = findLoadedClass(name);
                         if (loaded == null) loaded = findClass(name);
                         if (resolve) resolveClass(loaded);
                         return loaded;
                     }
                 }
             }) {
            var lookupA = new CountingLookup(first);
            var lookupB = new CountingLookup(second);
            int x = Integer.getInteger("terraforged.qa.cacheProbeX", 2048);
            int z = Integer.getInteger("terraforged.qa.cacheProbeZ", -2048);
            int[][] corners = {{x, z}, {x + 4, z}, {x, z + 4}, {x + 4, z + 4}};
            var expected = new ArrayList<Map<String, Object>>();
            for (var point : corners) expected.add(sample(lookupA, point[0], point[1], false));
            Class<?> previousClass = oldLoader.loadClass(OLD_CACHE);
            Object previous = previousClass.getConstructor().newInstance();
            var oldCompute = previousClass.getMethod("getAndUpdate", WorldLookup.class, int.class, int.class, boolean.class);
            require(previousClass.getClassLoader() == oldLoader, "Reference cache did not originate from isolated old JAR");
            lookupA.calls = 0;
            for (int y = -64; y < 320; y += 4) for (int corner = 0; corner < 4; corner++) {
                var point = corners[corner];
                Cell result = (Cell) oldCompute.invoke(previous, lookupA, point[0], point[1], false);
                require(snapshot(result).equals(expected.get(corner)), "Old actual cell differs from uncached target context");
            }
            int oldCalls = lookupA.calls;
            CellSampler.Cache2d cached = new CellSampler.Cache2d(16);
            lookupA.calls = 0;
            for (int y = -64; y < 320; y += 4) for (int corner = 0; corner < 4; corner++) {
                var point = corners[corner];
                require(snapshot(cached.getAndUpdate(lookupA, point[0], point[1], false)).equals(expected.get(corner)), "Cached cell raw fields changed");
            }
            int newCalls = lookupA.calls;
            require(oldCalls == 384 && newCalls == 4, "Four-corner vertical call-count regression: " + oldCalls + " -> " + newCalls);
            for (var lookup : List.of(lookupA, lookupB, lookupA)) {
                for (boolean climate : new boolean[] {false, true, false}) {
                    var direct = sample(lookup, x, z, climate);
                    require(snapshot(cached.getAndUpdate(lookup, x, z, climate)).equals(direct), "Lookup/climate cache key mismatch");
                }
            }
            require(!sample(lookupA, x, z, true).equals(sample(lookupB, x, z, true)), "Different genuine seeds produced identical complete test cells");
            require(!sample(lookupA, x, z, true).get("biomeRegionId").equals(sample(lookupB, x, z, true).get("biomeRegionId")), "ThreadLocal test requires distinct genuine region IDs");
            CellSampler a = new CellSampler(() -> lookupA, CellSampler.Field.BIOME_REGION);
            CellSampler b = new CellSampler(() -> lookupB, CellSampler.Field.BIOME_REGION);
            var point = new DensityFunction.SinglePointContext(x, 80, z);
            for (var sampler : List.of(a, b, a)) {
                var lookup = sampler == b ? lookupB : lookupA;
                require(Double.doubleToRawLongBits(sampler.compute(point)) == Double.doubleToRawLongBits((double) Float.intBitsToFloat(
                    (Integer) sample(lookup, x, z, true).get("biomeRegionId"))), "Shared static ThreadLocal reused another world's region");
            }
            // Bound the local table and verify evicted reader snapshots remain intact.
            Cell held = cached.getAndUpdate(lookupA, x, z, false);
            var heldSnapshot = snapshot(held);
            for (int index = 1; index <= 48; index++) cached.getAndUpdate(lookupA, x + index * 4, z + index * 4, false);
            require(snapshot(held).equals(heldSnapshot), "Eviction reset an in-flight reader's cell");
            var table = CellSampler.Cache2d.class.getDeclaredField("cells");
            require(table.trySetAccessible(), "Cannot inspect actual cache bounds");
            require(((Cell[]) table.get(cached)).length == 16, "Per-NoiseChunk cell cache is unbounded");
            var owner = CellSampler.Cache2d.class.getDeclaredField("owner");
            require(owner.getType() == WeakReference.class, "Cell cache has a strong lookup owner");
            System.out.println("CELL_CACHE_FOUR_CORNERS_OK oldCalls=" + oldCalls + " newCalls=" + newCalls + " completeRawFieldParity=true");
            if (Boolean.getBoolean("terraforged.qa.probeCacheReentry")) reentry(lookupA, lookupB, x, z);
            CellSampler.Cache2d weakCache = new CellSampler.Cache2d(16);
            var released = releasedContext(actual, noises, seed + 17, weakCache, x, z);
            for (int attempt = 0; attempt < 12 && (released.lookup().get() != null || released.context().get() != null); attempt++) {
                System.gc(); Thread.sleep(20);
            }
            require(released.lookup().get() == null && released.context().get() == null, "Closed genuine context retained through cache or scheduler");
            Map<String, Object> report = new LinkedHashMap<>();
            report.put("requested", true); report.put("status", "passed"); report.put("assertions", assertions);
            report.put("referenceJar", jar.toString()); report.put("referenceJarSha256", jarSha);
            report.put("referenceClassOrigin", previousClass.getProtectionDomain().getCodeSource().getLocation().toString());
            report.put("referenceClassIsolation", "Only old Cache2d class loaded from old JAR; all lookups and target game classes are genuine current classes");
            report.put("cornerCoordinates", java.util.Arrays.stream(corners).map(v -> List.of(v[0], v[1])).toList());
            report.put("verticalLevels", 96); report.put("oldActualHeightmapCalls", oldCalls); report.put("newActualHeightmapCalls", newCalls);
            report.put("allCellFields", expected.getFirst().keySet()); report.put("rawFieldParity", true);
            report.put("lookupSwitchOrder", List.of("A", "B", "A")); report.put("climateSwitchOrder", List.of(false, true, false));
            report.put("boundedSlots", 16); report.put("closedContextWeakReferencesCleared", true);
            report.put("scope", "Algorithmic call counts and complete cell value/lifecycle parity; not a controlled wall-time or FPS benchmark");
            return report;
        } finally {
            require(CacheManager.activeCacheCount() == ownersBefore, "Probe generator contexts leaked managed caches");
        }
    }

    private static Released releasedContext(GeneratorContext template,
            net.minecraft.core.HolderGetter<raccoonman.reterraforged.world.worldgen.noise.module.Noise> noises,
            int seed, CellSampler.Cache2d cache, int x, int z) {
        GeneratorContext context = GeneratorContext.makeCached(template.preset, noises, seed, 1, 1, false);
        WorldLookup lookup = context.lookup;
        cache.getAndUpdate(lookup, x, z, false);
        new CellSampler(() -> lookup, CellSampler.Field.BIOME_REGION).compute(new DensityFunction.SinglePointContext(x, 80, z));
        Released result = new Released(new WeakReference<>(lookup), new WeakReference<>(context));
        context.close();
        return result;
    }

    private record Released(WeakReference<WorldLookup> lookup, WeakReference<GeneratorContext> context) {}

    private static void reentry(CountingLookup a, CountingLookup b, int x, int z) throws Exception {
        var switched = new CellSampler.Cache2d(16);
        a.onNext = () -> switched.getAndUpdate(b, x + 4, z + 4, true);
        switched.getAndUpdate(a, x, z, false);
        require(snapshot(switched.getAndUpdate(b, x, z, true)).equals(sample(b, x, z, true)),
            "Reentrant A-to-B computation published A under B's owner/mode");
        var same = new CellSampler.Cache2d(16);
        a.onNext = () -> same.getAndUpdate(a, x + 4, z + 4, false);
        same.getAndUpdate(a, x, z, false);
        Cell zero = same.getAndUpdate(a, 0, 0, false);
        require(zero != null && snapshot(zero).equals(sample(a, 0, 0, false)), "Same-owner reentry left an uninitialized cache slot");
        var single = new CellSampler.Cache2d();
        single.getAndUpdate(a, x - 4, z - 4, false);
        Cell[] nested = new Cell[1];
        a.onNext = () -> nested[0] = single.getAndUpdate(b, x + 4, z + 4, true);
        single.getAndUpdate(a, x, z, false);
        require(nested[0] != null && snapshot(nested[0]).equals(sample(b, x + 4, z + 4, true)),
            "Single-cell reuse mutated the reentrant reader's returned cell");
    }

    private static Map<String, Object> sample(WorldLookup lookup, int x, int z, boolean climate) throws Exception {
        Cell cell = new Cell(); lookup.applyCell(cell, x, z, false, climate); return snapshot(cell);
    }

    private static Map<String, Object> snapshot(Cell cell) throws Exception {
        Map<String, Object> values = new TreeMap<>();
        for (var field : Cell.class.getFields()) {
            if (Modifier.isStatic(field.getModifiers())) continue;
            Object value = field.get(cell);
            if (field.getType() == float.class) value = Float.floatToRawIntBits((Float) value);
            else if (value instanceof raccoonman.reterraforged.world.worldgen.cell.terrain.Terrain terrain) value = terrain.getName();
            else if (value instanceof Enum<?> enumeration) value = enumeration.name();
            values.put(field.getName(), value);
        }
        return values;
    }

    private static final class CountingLookup extends WorldLookup {
        int calls;
        Runnable onNext;
        CountingLookup(GeneratorContext context) { super(context); }
        @Override public boolean applyCell(Cell cell, int x, int z, boolean load, boolean climate) {
            calls++;
            Runnable nested = this.onNext;
            this.onNext = null;
            if (nested != null) nested.run();
            return super.applyCell(cell, x, z, load, climate);
        }
    }

    private static void require(boolean condition, String message) {
        assertions++; if (!condition) throw new IllegalStateException(message);
    }
}
