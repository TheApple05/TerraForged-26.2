package raccoonman.reterraforged.client.gui.screen.presetconfig;

import java.io.PrintWriter;
import java.io.StringWriter;
import java.lang.reflect.Field;
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
import java.util.Optional;
import java.util.Set;
import java.util.concurrent.CompletableFuture;
import java.util.function.Supplier;
import java.util.function.Function;

import com.google.gson.Gson;
import com.google.gson.GsonBuilder;

import net.fabricmc.api.ClientModInitializer;
import net.fabricmc.fabric.api.client.event.lifecycle.v1.ClientTickEvents;
import net.fabricmc.loader.api.FabricLoader;
import net.fabricmc.fabric.impl.biome.modification.BiomeModificationImpl;
import net.minecraft.core.Holder;
import net.minecraft.world.level.biome.Biome;
import net.minecraft.world.level.biome.BiomeGenerationSettings;
import net.minecraft.world.level.biome.FeatureSorter;
import net.minecraft.world.level.chunk.ChunkGenerator;
import net.minecraft.SharedConstants;
import net.minecraft.client.Minecraft;
import net.minecraft.client.Screenshot;
import net.minecraft.client.gui.screens.TitleScreen;
import net.minecraft.client.gui.screens.BackupConfirmScreen;
import net.minecraft.client.gui.screens.ConfirmScreen;
import net.minecraft.client.gui.screens.Screen;
import net.minecraft.client.gui.screens.GenericMessageScreen;
import net.minecraft.client.gui.components.Button;
import net.minecraft.client.gui.components.EditBox;
import net.minecraft.client.gui.screens.worldselection.CreateWorldScreen;
import net.minecraft.client.gui.screens.worldselection.PresetEditor;
import net.minecraft.client.input.MouseButtonEvent;
import net.minecraft.client.input.MouseButtonInfo;
import raccoonman.reterraforged.RTFCommon;
import raccoonman.reterraforged.client.data.RTFTranslationKeys;
import raccoonman.reterraforged.client.gui.widget.Slider;
import raccoonman.reterraforged.client.gui.widget.WidgetList;
import raccoonman.reterraforged.world.worldgen.densityfunction.tile.Tile;
import raccoonman.reterraforged.world.worldgen.RTFRandomState;
import raccoonman.reterraforged.data.worldgen.preset.settings.Preset;
import raccoonman.reterraforged.registries.RTFRegistries;
import net.minecraft.world.level.GameType;
import net.minecraft.world.level.levelgen.Heightmap;
import net.minecraft.world.level.levelgen.presets.WorldPresets;
import net.minecraft.network.chat.Component;
import net.minecraft.network.chat.CommonComponents;

/** Separate client-only test mod: all captures come from Minecraft's real framebuffer. */
public final class ClientQaMod implements ClientModInitializer {
    private static final Gson JSON = new GsonBuilder().setPrettyPrinting().create();
    private static final Gson COMPACT = new Gson();
    private static final long SEED = 123456789L;
    private static final int SETTLE_TICKS = 30;
    private final Map<String, Object> report = new LinkedHashMap<>();
    private final List<Map<String, Object>> stages = new ArrayList<>();
    private final List<Map<String, Object>> screenshots = new ArrayList<>();
    private String phase = "title";
    private int ticks;
    private boolean busy, finished;
    private long started = System.nanoTime();
    private Path output;
    private CreateWorldScreen parent;
    private PresetConfigScreen config;
    private WorldSettingsPage editor;
    private CompletableFuture<Map<String, Object>> cameraSetup;
    private final String qaWorld = System.getProperty("terraforged.clientqa.world", "");
    private final List<Map<String, Object>> sliderChanges = new ArrayList<>();
    private String initialHeightDigest;
    private BackupConfirmScreen handledBackup;
    private final String vanillaWorld = System.getProperty("terraforged.clientqa.vanillaWorld", "");
    private boolean vanillaStage;
    private int originalGlobalModifierCount;
    private final boolean createFromUi = Boolean.getBoolean("terraforged.clientqa.createFromUi");
    private Screen handledCreationWarning;
    private String createdWorldFolder;

    @Override
    public void onInitializeClient() {
        output = Path.of(System.getProperty("terraforged.clientqa.output", "qa-output")).toAbsolutePath().normalize();
        report.put("schemaVersion", 1);
        report.put("status", "failed");
        report.put("startedAt", Instant.now().toString());
        report.put("stages", stages);
        report.put("screenshots", screenshots);
        report.put("preset", "Legacy default, selected through the actual preset-list callback");
        report.put("previewSeed", SEED);
        report.put("sliderChanges", sliderChanges);
        report.put("notTested", List.of("Every setting/page", "Other mod combinations", "NeoForge client", "Visual acceptance by user"));
        ClientTickEvents.END_CLIENT_TICK.register(this::tick);
    }

    private void tick(Minecraft client) {
        if (finished || busy) return;
        try {
            if (System.nanoTime() - started > 420_000_000_000L) throw new IllegalStateException("Client QA exceeded its 420-second bound at " + phase);
            if (client.gui.overlay() != null) return;
            switch (phase) {
                case "title" -> {
                    if (!(client.gui.screen() instanceof TitleScreen)) { ticks = 0; return; }
                    if (++ticks < SETTLE_TICKS) return;
                    Files.createDirectories(output);
                    report.put("runtime", provenance());
                    originalGlobalModifierCount = globalModifierCount();
                    report.put("globalModifierBaseline", originalGlobalModifierCount);
                    stage("title-ready", "Real title screen and production artifact verified");
                    capture(client, "01-title.png", Boolean.getBoolean("terraforged.clientqa.worldOnly") ? "open-copy" : "open-create");
                }
                case "open-copy" -> openQaWorld(client);
                case "open-create" -> {
                    phase = "create"; ticks = 0; busy = true;
                    try { CreateWorldScreen.openFresh(client, () -> client.gui.setScreen(new TitleScreen())); }
                    finally { busy = false; }
                }
                case "create" -> {
                    if (!(client.gui.screen() instanceof CreateWorldScreen actual)) return;
                    parent = actual;
                    if (++ticks < SETTLE_TICKS) return;
                    stage("create-world-ready", "Actual CreateWorldScreen loaded its real world-creation resources");
                    capture(client, "02-create-world.png", "open-list");
                }
                case "open-list" -> {
                    var factory = PresetEditor.EDITORS.get(Optional.of(WorldPresets.NORMAL));
                    require(factory != null, "Production preset editor was not registered for the normal world preset");
                    var registeredScreen = factory.createEditScreen(parent, parent.getUiState().getSettings());
                    require(registeredScreen instanceof PresetConfigScreen, "Registered production factory did not create the TerraForged editor");
                    config = (PresetConfigScreen) registeredScreen;
                    config.setSeed(SEED);
                    client.gui.setScreen(config);
                    phase = "list"; ticks = 0;
                }
                case "list" -> {
                    require(client.gui.screen() == config, "Actual preset-list screen was replaced unexpectedly");
                    if (++ticks < SETTLE_TICKS) return;
                    Object page = field(config, "currentPage");
                    require(page instanceof PresetListPage, "PresetConfigScreen did not initialize its actual preset list");
                    WidgetList<?> left = (WidgetList<?>) field(page, "left");
                    report.put("presetListRows", left.children().size());
                    require(left.children().size() >= 5, "Expected built-in preset rows are missing");
                    report.put("presetListTextFit", verifyListTextFit(client, page));
                    stage("preset-list-ready", left.children().size() + " actual list rows rendered");
                    capture(client, "03-preset-list.png", "open-editor");
                }
                case "open-editor" -> openEditor(client);
                case "editor" -> {
                    require(client.gui.screen() == config, "Preset editor screen was replaced unexpectedly");
                    if (++ticks < SETTLE_TICKS) return;
                    var evidence = previewEvidence();
                    report.put("previewBefore", evidence);
                    initialHeightDigest = evidence.get("heightSha256").toString();
                    Map<String, String> translations = new LinkedHashMap<>();
                    for (String key : List.of(RTFTranslationKeys.GUI_WORLD_SETTINGS_TITLE, RTFTranslationKeys.GUI_SLIDER_CONTINENT_SCALE, RTFTranslationKeys.GUI_BUTTON_SEED, RTFTranslationKeys.GUI_LABEL_PREVIEW_AREA)) {
                        String text = Component.translatable(key).getString();
                        translations.put(key, text);
                        if (Boolean.getBoolean("terraforged.clientqa.requireTranslations")) require(!text.equals(key), "Packaged production UI translation missing: " + key);
                    }
                    report.put("visibleTranslations", translations);
                    stage("preset-editor-ready", "Real legacy preset editor generated and uploaded its preview texture");
                    capture(client, "04-preset-editor-before.png", "change-slider");
                }
                case "change-slider" -> {
                    Slider slider = (Slider) field(editor, "continentScale");
                    require(slider.getWidth() > 8 && slider.getHeight() > 0, "Actual continent-scale slider has invalid bounds");
                    int before = editor.preset.getPreset().world().continent.continentScale;
                    Tile oldTile = previewTile();
                    MouseButtonEvent event = new MouseButtonEvent(slider.getX() + slider.getWidth() * 0.70, slider.getY() + slider.getHeight() / 2.0, new MouseButtonInfo(0, 0));
                    busy = true;
                    try { slider.onClick(event, false); slider.onRelease(event); }
                    finally { busy = false; }
                    int after = editor.preset.getPreset().world().continent.continentScale;
                    require(before != after, "Real slider input failed to update the actual preset setting");
                    require(previewTile() != oldTile, "Real slider callback failed to regenerate the preview tile");
                    report.put("sliderChange", Map.of("setting", "world.continent.continentScale", "before", before, "after", after, "input", "Actual AbstractSliderButton.onClick(MouseButtonEvent,false) and onRelease"));
                    sliderChanges.add(Map.of("before", before, "after", after, "preview", previewEvidence()));
                    phase = "changed"; ticks = 0;
                }
                case "changed" -> {
                    if (++ticks < SETTLE_TICKS) return;
                    report.put("previewAfter", previewEvidence());
                    stage("slider-preview-updated", "Actual setting changed and a replacement preview tile was rendered");
                    capture(client, "05-preset-editor-after.png", "repeat-sliders");
                }
                case "repeat-sliders" -> {
                    Slider slider = (Slider) field(editor, "continentScale");
                    int before = editor.preset.getPreset().world().continent.continentScale;
                    Tile oldTile = previewTile();
                    boolean restoreBaseline = sliderChanges.size() == 7;
                    double fraction = restoreBaseline ? (3000.25 - slider.getMin()) / (slider.getMax() - slider.getMin()) : 0.15 + (sliderChanges.size() % 4) * 0.20;
                    MouseButtonEvent event = new MouseButtonEvent(slider.getX() + 4 + (slider.getWidth() - 8) * fraction, slider.getY() + slider.getHeight() / 2.0, new MouseButtonInfo(0, 0));
                    busy = true;
                    try { slider.onClick(event, false); slider.onRelease(event); }
                    finally { busy = false; }
                    int after = editor.preset.getPreset().world().continent.continentScale;
                    require(before != after && previewTile() != oldTile, "Repeated real slider callback failed to replace setting/preview");
                    var evidence = previewEvidence();
                    sliderChanges.add(Map.of("before", before, "after", after, "preview", evidence));
                    if (sliderChanges.size() >= 8) {
                        require(after == 3000 && initialHeightDigest.equals(evidence.get("heightSha256")), "Restoring original setting did not restore exact preview height digest");
                        report.put("restoredBaselinePreviewDigest", initialHeightDigest);
                        stage("repeated-slider-previews", "Eight actual setting changes each regenerated a finite nonconstant 65536-cell preview");
                        phase = "close"; ticks = 0;
                    }
                }
                case "close" -> {
                    config.doneButton.onPress(new MouseButtonInfo(0, 0));
                    boolean appliedReady = parent.getUiState().getSettings().worldgenLoadContext().lookupOrThrow(RTFRegistries.PRESET).get(Preset.KEY).isPresent();
                    Screen immediateScreen = client.gui.screen();
                    boolean safeScreen = appliedReady || immediateScreen != parent && immediateScreen != config;
                    report.put("immediatePostDone", Map.of("appliedPresetReady", appliedReady, "screenClass", immediateScreen == null ? "null" : immediateScreen.getClass().getName(), "parentVisible", immediateScreen == parent, "editorVisible", immediateScreen == config, "safeScreenOwnership", safeScreen, "observedAt", Instant.now().toString()));
                    stage("post-done-screen-observed", "Applied preset ready=" + appliedReady + "; parent visible=" + (immediateScreen == parent));
                    require(safeScreen, "World creation became interactive immediately after Done before preset datapack validation completed");
                    require(Boolean.TRUE.equals(field(field(editor, "preview"), "closed")), "Production preview did not close its texture resources");
                    stage("preset-editor-closed", "Production close path released the actual preview resources");
                    phase = "preset-applied"; ticks = 0;
                }
                case "preset-applied" -> {
                    Screen waitingScreen = client.gui.screen();
                    if (waitingScreen instanceof ConfirmScreen confirmation) throw new IllegalStateException("Preset datapack validation opened a failure/confirmation screen: " + confirmation.getTitle().getString());
                    if (waitingScreen != parent) {
                        require(waitingScreen instanceof GenericMessageScreen, "Unexpected screen while waiting for actual preset application: " + (waitingScreen == null ? "null" : waitingScreen.getClass().getName()));
                        return;
                    }
                    var presets = parent.getUiState().getSettings().worldgenLoadContext().lookupOrThrow(RTFRegistries.PRESET);
                    if (presets.get(Preset.KEY).isEmpty()) return;
                    if (++ticks < SETTLE_TICKS) return;
                    Preset applied = presets.getOrThrow(Preset.KEY).value();
                    require(applied.world().continent.continentScale == 3000, "Actual Done callback applied the wrong preset settings");
                    require(parent.getUiState().getSettings().options().seed() == SEED, "Actual Done callback changed the selected seed");
                    report.put("presetApplication", Map.of("presetKey", Preset.KEY.identifier().toString(), "continentScale", applied.world().continent.continentScale, "seed", SEED, "actualDoneButton", true, "realDatapackReload", true));
                    stage("preset-datapack-applied", "Actual Done button exported the preset pack and reloaded it into the real world-creation context");
                    if (createFromUi) {
                        parent.getUiState().setName("TerraForged Client Qa");
                        createdWorldFolder = parent.getUiState().getTargetFolder();
                        var create = parent.children().stream().filter(child -> child instanceof Button button && button.getMessage().equals(Component.translatable("selectWorld.create"))).findFirst().orElseThrow();
                        phase = "world-loading"; ticks = 0;
                        busy = true;
                        try { ((Button) create).onPress(new MouseButtonInfo(0, 0)); }
                        finally { busy = false; }
                        stage("new-world-create-button", "Activated actual Create New World button with applied TerraForged datapack and selected seed");
                    } else if (qaWorld.isBlank()) {
                        report.put("terrainRendering", "not requested");
                        finish(client, true, null);
                    } else {
                        openQaWorld(client);
                    }
                }
                case "world-loading" -> {
                    if (client.gui.screen() instanceof ConfirmScreen warning &&
                        (warning.getTitle().equals(Component.translatable("selectWorld.warning.experimental.title")) || warning.getTitle().equals(Component.translatable("selectWorld.warning.deprecated.title")))) {
                        if (warning == handledCreationWarning) return;
                        handledCreationWarning = warning;
                        var proceed = warning.children().stream().filter(child -> child instanceof Button button && button.getMessage().equals(CommonComponents.GUI_YES)).findFirst().orElseThrow();
                        stage("new-world-experimental-warning", "Selected actual Yes callback for new isolated QA world");
                        busy = true;
                        try { ((Button) proceed).onPress(new MouseButtonInfo(0, 0)); }
                        finally { busy = false; }
                        return;
                    }
                    if (client.gui.screen() instanceof BackupConfirmScreen backup) {
                        if (backup == handledBackup) return;
                        handledBackup = backup;
                        var proceed = backup.children().stream().filter(child -> child instanceof Button button && button.getMessage().equals(Component.translatable("selectWorld.backupJoinSkipButton"))).findFirst().orElseThrow();
                        stage("copied-world-experimental-warning", "Selected real Skip Backup and Load callback for disposable copied world");
                        busy = true;
                        try { ((Button) proceed).onPress(new MouseButtonInfo(0, 0)); }
                        finally { busy = false; }
                        return;
                    }
                    if (client.level == null || client.player == null || client.getSingleplayerServer() == null) return;
                    if (++ticks < 60) return;
                    var server = client.getSingleplayerServer();
                    var uuid = client.player.getUUID();
                    cameraSetup = server.submit(() -> {
                        var level = server.overworld();
                        var randomState = (RTFRandomState) (Object) level.getChunkSource().randomState();
                        if (vanillaStage) {
                            require(randomState.preset() == null && randomState.generatorContext() == null, "Vanilla world inherited a TerraForged preset/context from the prior world");
                        } else {
                            require(randomState.preset() != null && randomState.generatorContext() != null && randomState.generatorContext().cache != null, "Integrated world has no active TerraForged generation context");
                            if (createFromUi) require(level.getSeed() == SEED, "Actual new custom world has the wrong selected seed");
                        }
                        Map<String, Object> sorter;
                        int globalCount;
                        try { sorter = verifyFeatureSorter(level.getChunkSource().getGenerator()); globalCount = globalModifierCount(); }
                        catch (Exception failure) { throw new IllegalStateException("Feature sorter consistency check failed", failure); }
                        var player = server.getPlayerList().getPlayer(uuid);
                        require(player != null, "Synthetic QA player missing from real integrated server");
                        int cameraX = player.getBlockX();
                        int cameraZ = player.getBlockZ();
                        level.getChunk(cameraX >> 4, cameraZ >> 4);
                        int height = level.getHeight(Heightmap.Types.MOTION_BLOCKING_NO_LEAVES, cameraX, cameraZ);
                        require(height > level.getMinY() + 1, "QA camera surface query returned an empty/unloaded column");
                        int cameraHeight = Integer.getInteger("terraforged.clientqa.cameraHeight", 48);
                        require(cameraHeight >= 8 && cameraHeight <= 128, "QA camera height must stay between 8 and 128 blocks");
                        player.setGameMode(GameType.SPECTATOR);
                        player.teleportTo(level, cameraX + 0.5, height + (double) cameraHeight, cameraZ + 0.5, Set.of(), -135.0F, 25.0F, true);
                        return Map.<String, Object>of("world", vanillaStage ? vanillaWorld : createFromUi ? createdWorldFolder : qaWorld, "customGeneratorActive", !vanillaStage, "surfaceHeight", height, "camera", List.of(cameraX + 0.5, height + (double) cameraHeight, cameraZ + 0.5, -135.0, 25.0), "mode", "spectator", "copiedWorldOnly", vanillaStage || !createFromUi, "createdThroughActualUi", !vanillaStage && createFromUi, "featureSorter", sorter, "worldSeed", level.getSeed(), "globalModifierCount", globalCount);
                    });
                    client.options.pauseOnLostFocus = false;
                    client.gui.setScreen(null);
                    phase = "terrain"; ticks = 0;
                }
                case "terrain" -> {
                    if (!cameraSetup.isDone()) return;
                    require(client.level != null && client.player != null, "Integrated level disappeared before terrain capture");
                    int settleTicks = Integer.getInteger("terraforged.clientqa.terrainSettleTicks", 200);
                    require(settleTicks >= 60 && settleTicks <= 1200, "QA terrain settling must stay between 60 and 1200 ticks");
                    if (++ticks < settleTicks) return;
                    require(globalModifierCount() == originalGlobalModifierCount, "World-scoped biome modifiers leaked into Fabric's global list");
                    report.put(vanillaStage ? "vanillaTerrainRendering" : "terrainRendering", cameraSetup.join());
                    report.put("terrainCaptureSettings", Map.of("renderDistance", client.options.renderDistance().get(), "settleTicks", settleTicks));
                    stage(vanillaStage ? "vanilla-world-rendered" : "custom-world-rendered", "Real integrated world remained loaded for " + settleTicks + " ticks after camera setup; global modifier count unchanged");
                    capture(client, vanillaStage ? "07-vanilla-after-custom.png" : "06-custom-terrain.png", "world-close");
                }
                case "world-close" -> {
                    busy = true;
                    try { client.disconnectWithSavingScreen(); }
                    finally { busy = false; }
                    stage("copied-world-saved", "Actual integrated-world disconnect and save completed");
                    if (!vanillaStage && !vanillaWorld.isBlank()) {
                        vanillaStage = true;
                        handledBackup = null;
                        client.gui.setScreen(new TitleScreen());
                        phase = "open-copy"; ticks = 0;
                    } else {
                        finish(client, true, null);
                    }
                }
                default -> throw new IllegalStateException("Unknown client QA phase: " + phase);
            }
        } catch (Throwable failure) {
            finish(client, false, failure);
        }
    }

    @SuppressWarnings("unchecked")
    private void openEditor(Minecraft client) throws Exception {
        Object page = field(config, "currentPage");
        WidgetList<PresetListPage.PresetEntry> list = (WidgetList<PresetListPage.PresetEntry>) field(page, "left");
        var first = list.children().stream().filter(entry -> entry.getWidget().isBuiltin()).findFirst().orElseThrow();
        list.setSelected(first);
        first.getWidget().onPress(new MouseButtonInfo(0, 0));
        require(config.nextButton.active, "Preset selection callback did not enable navigation");
        busy = true;
        try { config.nextButton.onPress(new MouseButtonInfo(0, 0)); }
        finally { busy = false; }
        Object selected = field(config, "currentPage");
        require(selected instanceof WorldSettingsPage, "Actual next-page callback did not open world settings");
        editor = (WorldSettingsPage) selected;
        require(editor.preset.isBuiltin(), "QA must edit a transient built-in copy without writing user presets");
        require(client.gui.screen() == config, "Actual preset editor is not the visible screen");
        phase = "editor"; ticks = 0;
    }

    private void openQaWorld(Minecraft client) {
        String world = vanillaStage ? vanillaWorld : qaWorld;
        require(!world.isBlank(), "worldOnly QA requires a copied QA world");
        phase = "world-loading"; ticks = 0; busy = true;
        try { client.createWorldOpenFlows().openWorld(world, () -> finish(client, false, new IllegalStateException("Copied QA world opening was cancelled"))); }
        finally { busy = false; }
    }

    private Tile previewTile() throws Exception {
        Object preview = field(editor, "preview");
        return (Tile) field(preview, "tile");
    }

    private int globalModifierCount() throws Exception {
        return ((List<?>) field(BiomeModificationImpl.INSTANCE, "modifiers")).size();
    }

    private static List<Map<String, Object>> verifyListTextFit(Minecraft client, Object page) throws Exception {
        List<Map<String, Object>> measurements = new ArrayList<>();
        EditBox input = (EditBox) field(page, "input");
        Component hint = (Component) field(input, "hint");
        require(hint != null, "Actual preset input hint is absent");
        int hintWidth = client.font.width(hint);
        int inputInnerWidth = input.getInnerWidth();
        require(input.getWidth() > 0 && hintWidth <= inputInnerWidth, "Actual preset input hint exceeds its rendered inner width");
        measurements.add(Map.of("field", "input.hint", "text", hint.getString(), "fontWidth", hintWidth, "innerWidth", inputInnerWidth, "widgetWidth", input.getWidth(), "widthRule", "Actual EditBox.getInnerWidth()", "fits", true));
        for (String name : List.of("openPresetFolder", "openExportFolder", "exportAsDatapack")) {
            Button button = (Button) field(page, name);
            Component message = button.getMessage();
            int textWidth = client.font.width(message);
            int innerWidth = button.getWidth() - 4;
            require(innerWidth > 0 && textWidth <= innerWidth, "Actual list button label exceeds its rendered inner width: " + name);
            measurements.add(Map.of("field", name, "text", message.getString(), "fontWidth", textWidth, "innerWidth", innerWidth, "widgetWidth", button.getWidth(), "widthRule", "Actual AbstractButton label margin of 2 pixels on both sides", "fits", true));
        }
        return measurements;
    }

    @SuppressWarnings("unchecked")
    private static Map<String, Object> verifyFeatureSorter(ChunkGenerator generator) throws Exception {
        generator.validate();
        var supplier = (Supplier<List<FeatureSorter.StepFeatureData>>) field(generator, "featuresPerStep");
        var getter = (Function<Holder<Biome>, BiomeGenerationSettings>) field(generator, "generationSettingsGetter");
        List<FeatureSorter.StepFeatureData> sorted = supplier.get();
        int references = 0;
        for (var biome : generator.getBiomeSource().possibleBiomes()) {
            var features = getter.apply(biome).features();
            for (int step = 0; step < features.size(); step++) {
                for (var feature : features.get(step)) {
                    require(step < sorted.size(), "Biome has features beyond the generator sorter steps");
                    var data = sorted.get(step);
                    int index = data.indexMapping().applyAsInt(feature.value());
                    require(index >= 0 && index < data.features().size() && data.features().get(index) == feature.value(), "Current biome feature is absent from the generator's feature sorter: " + biome + " step " + step);
                    references++;
                }
            }
        }
        require(references > 0, "Generator feature sorter check inspected no biome features");
        return Map.of("possibleBiomes", generator.getBiomeSource().possibleBiomes().size(), "steps", sorted.size(), "featureReferencesChecked", references, "identityConsistent", true);
    }

    private Map<String, Object> previewEvidence() throws Exception {
        Object preview = field(editor, "preview");
        Tile tile = (Tile) field(preview, "tile");
        require(tile != null, "Production preview has no generated tile");
        var texture = (net.minecraft.client.renderer.texture.DynamicTexture) field(preview, "texture");
        require(texture.getPixels() != null, "Production preview image was not allocated");
        int[] count = {0};
        float[] range = {Float.POSITIVE_INFINITY, Float.NEGATIVE_INFINITY};
        MessageDigest digest = MessageDigest.getInstance("SHA-256");
        tile.iterate((cell, x, z) -> {
            require(Float.isFinite(cell.height), "Preview terrain has nonfinite height");
            range[0] = Math.min(range[0], cell.height); range[1] = Math.max(range[1], cell.height); count[0]++;
            int bits = Float.floatToRawIntBits(cell.height);
            digest.update((byte) (bits >>> 24)); digest.update((byte) (bits >>> 16)); digest.update((byte) (bits >>> 8)); digest.update((byte) bits);
        });
        require(count[0] > 0 && range[1] > range[0], "Production preview terrain is empty or constant");
        return Map.of("textureId", field(preview, "textureId").toString(), "textureWidth", texture.getPixels().getWidth(), "textureHeight", texture.getPixels().getHeight(), "cells", count[0], "minHeight", range[0], "maxHeight", range[1], "continentScale", editor.preset.getPreset().world().continent.continentScale, "seed", SEED, "heightSha256", HexFormat.of().formatHex(digest.digest()));
    }

    private void capture(Minecraft client, String name, String next) throws Exception {
        Path file = output.resolve("screenshots").resolve(name);
        require(!Files.exists(file), "Refusing stale screenshot output; use a fresh isolated QA directory: " + file);
        busy = true;
        int width = client.gameRenderer.mainRenderTarget().width;
        int height = client.gameRenderer.mainRenderTarget().height;
        Screenshot.grab(output.toFile(), name, client.gameRenderer.mainRenderTarget(), 1, message -> client.execute(() -> {
            try {
                require(Files.isRegularFile(file) && Files.size(file) > 0, "Minecraft Screenshot API did not save " + file + ": " + message.getString());
                screenshots.add(Map.of("path", file.toString(), "sha256", sha256(file), "bytes", Files.size(file), "width", width, "height", height, "screenClass", client.gui.screen() == null ? "in-world" : client.gui.screen().getClass().getName(), "captureApi", "net.minecraft.client.Screenshot.grab"));
                stage("screenshot-" + name, file.toString());
                phase = next; ticks = 0; busy = false;
            } catch (Throwable failure) { finish(client, false, failure); }
        }));
    }

    private Map<String, Object> provenance() throws Exception {
        FabricLoader loader = FabricLoader.getInstance();
        require(!loader.isDevelopmentEnvironment(), "Client QA requires the packaged production launch");
        Path origin = Path.of(RTFCommon.class.getProtectionDomain().getCodeSource().getLocation().toURI()).toAbsolutePath().normalize();
        require(Files.isRegularFile(origin) && origin.toString().endsWith(".jar"), "Production class did not come from a JAR");
        String sha = sha256(origin);
        String expected = System.getProperty("terraforged.clientqa.expectedModSha256", "");
        if (!expected.isBlank()) require(expected.equalsIgnoreCase(sha), "Unexpected production JAR hash");
        return Map.of("minecraft", SharedConstants.getCurrentVersion().id(), "java", System.getProperty("java.version"), "loader", loader.getModContainer("fabricloader").orElseThrow().getMetadata().getVersion().getFriendlyString(), "modVersion", loader.getModContainer("reterraforged").orElseThrow().getMetadata().getVersion().getFriendlyString(), "modClassOrigin", origin.toString(), "modSha256", sha, "developmentEnvironment", false);
    }

    private void stage(String name, String detail) {
        Map<String, Object> stage = Map.of("name", name, "detail", detail, "at", Instant.now().toString());
        stages.add(stage);
        try {
            Files.createDirectories(output);
            Map<String, Object> progress = new LinkedHashMap<>(report);
            progress.put("status", "running"); progress.put("phase", phase);
            Files.writeString(output.resolve("progress.json"), JSON.toJson(progress) + "\n", StandardCharsets.UTF_8);
        } catch (java.io.IOException failure) {
            throw new java.io.UncheckedIOException("Unable to persist client QA progress", failure);
        }
        System.out.println("TERRAFORGED_CLIENT_QA_STAGE_JSON=" + COMPACT.toJson(stage));
    }

    private void finish(Minecraft client, boolean passed, Throwable failure) {
        if (finished) return;
        finished = true;
        report.put("status", passed ? "passed" : "failed");
        report.put("phase", phase);
        report.put("finishedAt", Instant.now().toString());
        report.put("cleanStopRequested", true);
        if (failure != null) {
            StringWriter trace = new StringWriter(); failure.printStackTrace(new PrintWriter(trace));
            report.put("error", Map.of("message", failure.toString(), "stackTrace", trace.toString()));
        }
        try { Files.createDirectories(output); Files.writeString(output.resolve("result.json"), JSON.toJson(report) + "\n", StandardCharsets.UTF_8); }
        catch (Exception writeFailure) { report.put("status", "failed"); report.put("resultWriteError", writeFailure.toString()); }
        System.out.println("TERRAFORGED_CLIENT_QA_RESULT_JSON=" + COMPACT.toJson(report));
        client.stop();
    }

    private static Object field(Object target, String name) throws Exception {
        for (Class<?> type = target.getClass(); type != null; type = type.getSuperclass()) {
            try { Field field = type.getDeclaredField(name); field.setAccessible(true); return field.get(target); }
            catch (NoSuchFieldException absent) { /* Continue through the actual class hierarchy. */ }
        }
        throw new NoSuchFieldException(target.getClass().getName() + "." + name);
    }

    private static String sha256(Path file) throws Exception {
        return HexFormat.of().formatHex(MessageDigest.getInstance("SHA-256").digest(Files.readAllBytes(file)));
    }

    private static void require(boolean condition, String message) {
        if (!condition) throw new IllegalStateException(message);
    }
}
