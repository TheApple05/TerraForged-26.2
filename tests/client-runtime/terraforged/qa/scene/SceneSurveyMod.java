package terraforged.qa.scene;

import java.io.PrintWriter;
import java.io.StringWriter;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.security.MessageDigest;
import java.time.Instant;
import java.util.ArrayList;
import java.util.HexFormat;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.Set;
import java.util.concurrent.CompletableFuture;

import com.google.gson.Gson;
import com.google.gson.GsonBuilder;

import net.fabricmc.api.ClientModInitializer;
import net.fabricmc.fabric.api.client.event.lifecycle.v1.ClientTickEvents;
import net.fabricmc.loader.api.FabricLoader;
import net.minecraft.SharedConstants;
import net.minecraft.client.Minecraft;
import net.minecraft.client.Screenshot;
import net.minecraft.client.CameraType;
import net.minecraft.client.KeyMapping;
import net.minecraft.client.gui.components.Button;
import net.minecraft.client.gui.screens.BackupConfirmScreen;
import net.minecraft.client.gui.screens.TitleScreen;
import net.minecraft.client.input.MouseButtonInfo;
import net.minecraft.core.BlockPos;
import net.minecraft.core.registries.BuiltInRegistries;
import net.minecraft.network.chat.Component;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.server.level.ServerPlayer;
import net.minecraft.world.level.GameType;
import net.minecraft.world.level.levelgen.Heightmap;
import raccoonman.reterraforged.RTFCommon;
import raccoonman.reterraforged.world.worldgen.GeneratorContext;
import raccoonman.reterraforged.world.worldgen.RTFRandomState;
import raccoonman.reterraforged.world.worldgen.cell.Cell;

/** Optional bounded visual survey. Searches raw heightmap cells, never tiles/chunks. */
public final class SceneSurveyMod implements ClientModInitializer {
    private static final Gson JSON = new GsonBuilder().setPrettyPrinting().create();
    private static final Gson COMPACT = new Gson();
    private static final List<String> KINDS = List.of("mountain", "river", "coast");
    private final Map<String, Object> report = new LinkedHashMap<>();
    private final List<Map<String, Object>> stages = new ArrayList<>();
    private final List<Map<String, Object>> screenshots = new ArrayList<>();
    private final List<Map<String, Object>> scenes = new ArrayList<>();
    private Path output;
    private String phase = "title";
    private int ticks, sceneIndex;
    private boolean busy, finished;
    private BackupConfirmScreen handledBackup;
    private final long started = System.nanoTime();
    private CompletableFuture<SearchResult> search;
    private SearchResult found;
    private CompletableFuture<Map<String, Object>> prepared;

    private record Candidate(String kind, int x, int z, Map<String, Object> rawCell) {}
    private record SearchResult(List<Candidate> candidates, Map<String, Object> evidence) {}

    @Override
    public void onInitializeClient() {
        output = Path.of(System.getProperty("terraforged.clientqa.output", "qa-output")).toAbsolutePath().normalize();
        report.put("schemaVersion", 1);
        report.put("mode", "bounded-scene-survey");
        report.put("status", "failed");
        report.put("startedAt", Instant.now().toString());
        report.put("stages", stages);
        report.put("screenshots", screenshots);
        report.put("scenes", scenes);
        report.put("notTested", List.of("All seeds/presets/terrain types", "Exact classic visual parity", "NeoForge scene survey", "Third-party mods", "User visual acceptance"));
        ClientTickEvents.END_CLIENT_TICK.register(this::tick);
    }

    private void tick(Minecraft client) {
        if (finished || busy) return;
        try {
            require(System.nanoTime() - started < 420_000_000_000L, "Scene survey exceeded its 420-second bound at " + phase);
            if (client.gui.overlay() != null) return;
            switch (phase) {
                case "title" -> {
                    if (!(client.gui.screen() instanceof TitleScreen)) { ticks = 0; return; }
                    if (++ticks < 30) return;
                    report.put("runtime", provenance());
                    phase = "world-loading"; ticks = 0;
                    busy = true;
                    try {
                        String world = System.getProperty("terraforged.clientqa.world", "");
                        require(!world.isBlank(), "Scene survey requires a fresh copied custom world via --qa-world");
                        client.createWorldOpenFlows().openWorld(world, () -> finish(client, false, new IllegalStateException("Copied world opening cancelled")));
                    } finally { busy = false; }
                }
                case "world-loading" -> {
                    if (client.gui.screen() instanceof BackupConfirmScreen screen) {
                        if (screen == handledBackup) return;
                        handledBackup = screen;
                        var proceed = screen.children().stream().filter(child -> child instanceof Button button && button.getMessage().equals(Component.translatable("selectWorld.backupJoinSkipButton"))).findFirst().orElseThrow();
                        busy = true;
                        try { ((Button) proceed).onPress(new MouseButtonInfo(0, 0)); }
                        finally { busy = false; }
                        return;
                    }
                    if (client.level == null || client.player == null || client.getSingleplayerServer() == null) return;
                    if (++ticks < 40) return;
                    var server = client.getSingleplayerServer();
                    int anchorX = client.player.getBlockX();
                    int anchorZ = client.player.getBlockZ();
                    search = server.submit(() -> {
                        require(server.overworld().getSeed() == 123456789L, "Scene survey requires the validated dev8 UI-created world's seed");
                        return search(context(server.overworld()), server.overworld().getSeed(), anchorX, anchorZ);
                    });
                    client.options.pauseOnLostFocus = false;
                    client.gui.setScreen(null);
                    phase = "search"; ticks = 0;
                    stage("search-started", "Bounded direct heightmap search; no chunk or tile requests in search loop");
                }
                case "search" -> {
                    if (!search.isDone()) return;
                    found = search.join();
                    report.put("search", found.evidence());
                    report.put("customGeneratorActive", true);
                    stage("search-complete", found.candidates().size() + " candidate categories selected within the recorded bound");
                    phase = "next-scene"; ticks = 0;
                }
                case "next-scene" -> {
                    if (sceneIndex >= found.candidates().size()) {
                        report.put("sceneCoverageComplete", scenes.size() == 3 && scenes.stream().allMatch(scene -> Boolean.TRUE.equals(scene.get("classificationVerified"))));
                        phase = "save";
                        return;
                    }
                    var server = client.getSingleplayerServer();
                    require(server != null && client.player != null, "Actual integrated world vanished before scene preparation");
                    Candidate candidate = found.candidates().get(sceneIndex);
                    var uuid = client.player.getUUID();
                    prepared = server.submit(() -> prepare(server.overworld(), server.getPlayerList().getPlayer(uuid), candidate));
                    phase = "settle"; ticks = 0;
                }
                case "settle" -> {
                    if (!prepared.isDone()) return;
                    Map<String, Object> scene = prepared.join();
                    require(client.player != null, "Actual player disappeared during scene settling");
                    @SuppressWarnings("unchecked")
                    List<Number> requested = (List<Number>) scene.get("camera");
                    client.mouseHandler.releaseMouse();
                    KeyMapping.releaseAll();
                    client.options.setCameraType(CameraType.FIRST_PERSON);
                    client.setCameraEntity(client.player);
                    client.player.setPos(requested.get(0).doubleValue(), requested.get(1).doubleValue(), requested.get(2).doubleValue());
                    client.player.setYRot(requested.get(3).floatValue());
                    client.player.setXRot(requested.get(4).floatValue());
                    client.player.yRotO = requested.get(3).floatValue();
                    client.player.xRotO = requested.get(4).floatValue();
                    if (++ticks < 400) return;
                    require(client.level != null && client.player != null, "Actual world disappeared before screenshot");
                    var observedCamera = client.gameRenderer.mainCamera();
                    require(Math.abs(observedCamera.xRot() - requested.get(4).floatValue()) < 0.1F && Math.abs(observedCamera.yRot() - requested.get(3).floatValue()) < 0.1F, "Actual rendered camera rotation does not match the requested scene pose");
                    require(Math.abs(observedCamera.position().x - requested.get(0).doubleValue()) < 0.1 && Math.abs(observedCamera.position().z - requested.get(2).doubleValue()) < 0.1, "Actual rendered camera position does not match the requested scene pose");
                    scene.put("actualPlayerPose", List.of(client.player.getX(), client.player.getY(), client.player.getZ(), client.player.getYRot(), client.player.getXRot()));
                    scene.put("actualRenderedCameraPose", List.of(observedCamera.position().x, observedCamera.position().y, observedCamera.position().z, observedCamera.yRot(), observedCamera.xRot()));
                    scene.put("poseHeldByQa", "Actual first-person spectator pose held while mouse is released; entity and rendered-camera rotation/position verified before capture");
                    String kind = found.candidates().get(sceneIndex).kind();
                    String filename = String.format("%02d-%s.png", 8 + KINDS.indexOf(kind), kind);
                    scene.put("capturedAt", Instant.now().toString());
                    scene.put("renderDistance", client.options.renderDistance().get());
                    scene.put("settleTicks", ticks);
                    scene.put("screenshot", output.resolve("screenshots").resolve(filename).toString());
                    scenes.add(scene);
                    stage("scene-" + kind, "Actual chunks rendered; classificationVerified=" + scene.get("classificationVerified"));
                    sceneIndex++;
                    capture(client, filename);
                }
                case "save" -> {
                    busy = true;
                    try { client.disconnectWithSavingScreen(); }
                    finally { busy = false; }
                    stage("copied-world-saved", "Actual integrated world save/disconnect completed");
                    finish(client, true, null);
                }
                default -> throw new IllegalStateException("Unknown survey phase: " + phase);
            }
        } catch (Throwable failure) { finish(client, false, failure); }
    }

    private static GeneratorContext context(ServerLevel level) {
        var state = (RTFRandomState) (Object) level.getChunkSource().randomState();
        require(state.preset() != null && state.generatorContext() != null && state.generatorContext().cache != null, "Actual world is missing TerraForged preset/context/cache; refusing vanilla fallback");
        return state.generatorContext();
    }

    private static SearchResult search(GeneratorContext context, long seed, int anchorX, int anchorZ) {
        long begin = System.nanoTime();
        Map<String, Candidate> selected = new LinkedHashMap<>();
        Map<String, Float> scores = new LinkedHashMap<>();
        Map<String, Integer> counts = new LinkedHashMap<>();
        Candidate nearRiver = null, nearCoast = null;
        float riverDistance = Float.POSITIVE_INFINITY, coastDistance = Float.POSITIVE_INFINITY;
        int samples = 0;
        for (int z = 0; z < 16; z++) for (int x = 0; x < 16; x++) {
            if (System.nanoTime() - begin > 20_000_000_000L) break;
            int worldX = anchorX + (x - 8) * 512;
            int worldZ = anchorZ + (z - 8) * 512;
            Cell cell = sample(context, worldX, worldZ); samples++;
            choose(selected, scores, counts, cell, worldX, worldZ, context.levels.water);
            if (!cell.terrain.isCoast() && !cell.terrain.isDeepOcean() && !cell.terrain.isShallowOcean() && cell.riverMask < riverDistance) {
                riverDistance = cell.riverMask; nearRiver = new Candidate("river", worldX, worldZ, fields(cell));
            }
            float seaDistance = Math.abs(cell.height - context.levels.water);
            if (cell.terrain.isOverground() && seaDistance < coastDistance) {
                coastDistance = seaDistance; nearCoast = new Candidate("coast", worldX, worldZ, fields(cell));
            }
        }
        for (Candidate center : new Candidate[] {nearRiver, nearCoast}) {
            if (center == null || selected.containsKey(center.kind())) continue;
            for (int z = 0; z < 16; z++) for (int x = 0; x < 16; x++) {
                if (System.nanoTime() - begin > 20_000_000_000L) break;
                int worldX = center.x() + (x - 8) * 16;
                int worldZ = center.z() + (z - 8) * 16;
                Cell cell = sample(context, worldX, worldZ); samples++;
                choose(selected, scores, counts, cell, worldX, worldZ, context.levels.water);
            }
        }
        List<Candidate> candidates = new ArrayList<>();
        List<String> missing = new ArrayList<>();
        for (String kind : KINDS) {
            if (selected.containsKey(kind)) candidates.add(selected.get(kind));
            else missing.add(kind);
        }
        Map<String, Object> evidence = new LinkedHashMap<>();
        evidence.put("seed", seed); evidence.put("anchor", List.of(anchorX, anchorZ));
        evidence.put("coarseGrid", "16x16 at 512-block spacing around anchor");
        evidence.put("refinement", "Only missing river/coast categories, at most 16x16 at 16-block spacing around nearest coarse cell");
        evidence.put("heightmapSamples", samples); evidence.put("maximumHeightmapSamples", 768);
        evidence.put("elapsedMillis", (System.nanoTime() - begin) / 1_000_000L);
        evidence.put("timeBoundMillis", 20_000); evidence.put("tileRequestsDuringSearch", 0); evidence.put("chunkRequestsDuringSearch", 0);
        evidence.put("matchingSampleCounts", counts); evidence.put("missingScenesUnverified", missing);
        evidence.put("searchApi", "Actual GeneratorContext.generator.getHeightmap().apply(cell,x,z,true), unfiltered candidate classification");
        return new SearchResult(candidates, evidence);
    }

    private static Cell sample(GeneratorContext context, int x, int z) {
        Cell cell = new Cell();
        context.generator.getHeightmap().apply(cell, x, z, true);
        require(Float.isFinite(cell.height) && Float.isFinite(cell.riverMask) && Float.isFinite(cell.continentEdge), "Nonfinite terrain search cell");
        return cell;
    }

    private static void choose(Map<String, Candidate> selected, Map<String, Float> scores, Map<String, Integer> counts, Cell cell, int x, int z, float waterLevel) {
        for (String kind : KINDS) if (matches(kind, cell)) {
            counts.merge(kind, 1, Integer::sum);
            float score = switch (kind) { case "mountain" -> cell.height; case "river" -> 1 - cell.riverMask; default -> -Math.abs(cell.height - waterLevel); };
            if (!scores.containsKey(kind) || score > scores.get(kind)) {
                scores.put(kind, score); selected.put(kind, new Candidate(kind, x, z, fields(cell)));
            }
        }
    }

    private static boolean matches(String kind, Cell cell) {
        return switch (kind) { case "mountain" -> cell.terrain.isMountain(); case "river" -> cell.terrain.isRiver(); default -> cell.terrain.isCoast(); };
    }

    private static Map<String, Object> prepare(ServerLevel level, ServerPlayer player, Candidate candidate) {
        long begin = System.nanoTime();
        GeneratorContext context = context(level);
        require(player != null, "Synthetic QA player missing from integrated server");
        level.getChunk(candidate.x() >> 4, candidate.z() >> 4);
        int targetY = level.getHeight(Heightmap.Types.MOTION_BLOCKING_NO_LEAVES, candidate.x(), candidate.z());
        int cameraX = candidate.x() - 96, cameraZ = candidate.z() + 96;
        level.getChunk(cameraX >> 4, cameraZ >> 4);
        int cameraSurface = level.getHeight(Heightmap.Types.MOTION_BLOCKING_NO_LEAVES, cameraX, cameraZ);
        require(targetY > level.getMinY() + 1 && cameraSurface > level.getMinY() + 1, "Scene camera/target column is empty or unloaded");
        Cell actual = new Cell();
        context.lookup.applyCell(actual, candidate.x(), candidate.z(), true, true);
        double cameraY = Math.max(targetY + 40.0, cameraSurface + 32.0);
        float yaw = (float) Math.toDegrees(Math.atan2(-(candidate.x() - cameraX), candidate.z() - cameraZ));
        float pitch = (float) Math.toDegrees(Math.atan2(cameraY - targetY, Math.hypot(candidate.x() - cameraX, candidate.z() - cameraZ)));
        player.setGameMode(GameType.SPECTATOR);
        player.teleportTo(level, cameraX + 0.5, cameraY, cameraZ + 0.5, Set.of(), yaw, pitch, true);
        Map<String, Object> result = new LinkedHashMap<>();
        result.put("kind", candidate.kind()); result.put("seed", level.getSeed()); result.put("target", List.of(candidate.x(), targetY, candidate.z()));
        result.put("camera", List.of(cameraX + 0.5, cameraY, cameraZ + 0.5, yaw, pitch));
        result.put("cameraSurfaceHeight", cameraSurface); result.put("rawSearchCell", candidate.rawCell()); result.put("actualCachedTileCell", fields(actual));
        result.put("classificationVerified", matches(candidate.kind(), actual)); result.put("customGeneratorActive", true);
        result.put("actualBiome", level.getBiome(new BlockPos(candidate.x(), targetY, candidate.z())).unwrapKey().orElseThrow().identifier().toString());
        result.put("surfaceBlock", BuiltInRegistries.BLOCK.getKey(level.getBlockState(new BlockPos(candidate.x(), targetY - 1, candidate.z())).getBlock()).toString());
        result.put("preparationMillis", (System.nanoTime() - begin) / 1_000_000L);
        result.put("explicitFullChunkRequests", List.of(List.of(candidate.x() >> 4, candidate.z() >> 4), List.of(cameraX >> 4, cameraZ >> 4)));
        result.put("preparedAt", Instant.now().toString());
        return result;
    }

    private static Map<String, Object> fields(Cell cell) {
        return Map.of("terrain", cell.terrain.getName(), "category", cell.terrain.getCategory().name(), "height", cell.height, "riverMask", cell.riverMask, "continentEdge", cell.continentEdge, "biomeType", cell.biome.name(), "temperature", cell.temperature, "moisture", cell.moisture, "erosion", cell.erosion, "weirdness", cell.weirdness);
    }

    private Map<String, Object> provenance() throws Exception {
        var loader = FabricLoader.getInstance();
        require(!loader.isDevelopmentEnvironment(), "Scene survey requires packaged production launch");
        Path origin = Path.of(RTFCommon.class.getProtectionDomain().getCodeSource().getLocation().toURI()).toAbsolutePath().normalize();
        require(Files.isRegularFile(origin), "Production mod class origin is not a packaged JAR");
        String hash = sha256(origin);
        require(hash.equals(System.getProperty("terraforged.clientqa.expectedModSha256", "")), "Production artifact hash mismatch");
        return Map.of("minecraft", SharedConstants.getCurrentVersion().id(), "java", System.getProperty("java.version"), "loader", loader.getModContainer("fabricloader").orElseThrow().getMetadata().getVersion().getFriendlyString(), "modSha256", hash, "modClassOrigin", origin.toString(), "developmentEnvironment", false);
    }

    private void capture(Minecraft client, String name) throws Exception {
        Path file = output.resolve("screenshots").resolve(name);
        require(!Files.exists(file), "Refusing stale scene screenshot output");
        busy = true;
        int width = client.gameRenderer.mainRenderTarget().width, height = client.gameRenderer.mainRenderTarget().height;
        Screenshot.grab(output.toFile(), name, client.gameRenderer.mainRenderTarget(), 1, message -> client.execute(() -> {
            try {
                require(Files.isRegularFile(file) && Files.size(file) > 0, "Actual Minecraft screenshot failed: " + message.getString());
                screenshots.add(Map.of("path", file.toString(), "sha256", sha256(file), "bytes", Files.size(file), "width", width, "height", height, "captureApi", "net.minecraft.client.Screenshot.grab"));
                stage("screenshot-" + name, file.toString()); phase = "next-scene"; ticks = 0; busy = false;
            } catch (Throwable failure) { finish(client, false, failure); }
        }));
    }

    private void stage(String name, String detail) {
        stages.add(Map.of("name", name, "detail", detail, "at", Instant.now().toString()));
        try { Files.createDirectories(output); Files.writeString(output.resolve("progress.json"), JSON.toJson(report) + "\n", StandardCharsets.UTF_8); }
        catch (Exception failure) { throw new IllegalStateException("Cannot persist scene survey progress", failure); }
        System.out.println("TERRAFORGED_CLIENT_QA_STAGE_JSON=" + COMPACT.toJson(stages.getLast()));
    }

    private void finish(Minecraft client, boolean passed, Throwable failure) {
        if (finished) return;
        finished = true;
        report.put("status", passed ? "passed" : "failed"); report.put("phase", phase); report.put("finishedAt", Instant.now().toString()); report.put("cleanStopRequested", true);
        if (failure != null) { StringWriter trace = new StringWriter(); failure.printStackTrace(new PrintWriter(trace)); report.put("error", trace.toString()); }
        try { Files.createDirectories(output); Files.writeString(output.resolve("result.json"), JSON.toJson(report) + "\n", StandardCharsets.UTF_8); }
        catch (Exception writeFailure) { report.put("status", "failed"); report.put("writeError", writeFailure.toString()); }
        System.out.println("TERRAFORGED_CLIENT_QA_RESULT_JSON=" + COMPACT.toJson(report)); client.stop();
    }

    private static String sha256(Path file) throws Exception { return HexFormat.of().formatHex(MessageDigest.getInstance("SHA-256").digest(Files.readAllBytes(file))); }
    private static void require(boolean condition, String message) { if (!condition) throw new IllegalStateException(message); }
}
