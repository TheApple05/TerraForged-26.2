# Disposable production client preparation

`tools/qa/production_client.py` resolves the official Minecraft 26.2 descriptor and Fabric Loader 0.20.1 client profile. It verifies declared library/asset hashes, reuses matching Gradle or Minecraft cache files read-only, and downloads missing objects with at most four concurrent workers. New files, extracted DLLs, launch arguments and game state stay under ignored `build/qa/client`.

```powershell
python tools/qa/production_client.py --java-home PATH_TO_JDK_25 --jar build/libs/CANDIDATE.jar --report docs/validation/client-staged.json
python tools/qa/production_client.py --java-home PATH_TO_JDK_25 --jar build/libs/CANDIDATE.jar --launch --report docs/validation/client-run.json
```

The first command prepares and stages without opening Minecraft. Only `--launch` opens the native game window. It uses a new disposable game directory, a synthetic offline QA identity, token `0`, development mode disabled and a 3 GiB heap. No launcher account/profile/authentication data is read or changed. Parent/user UI review and clean in-game closure are deliberately separate from this tool.

The launcher prints its instance and owned process PID, writes a running report, and records the real JVM exit and packaged class origins after the window is closed. If the observation deadline expires, it preserves the interactive game window and reports that manual closure is still required. It does not infer visual correctness from successful loading or a clean process exit.

Mojang's metadata includes x64, ARM64 and x86 native JARs under Windows-only rules. Those declared libraries remain on the official classpath; explicit native extraction selects only the host architecture to prevent conflicting DLLs. The first staging attempt caught that collision and is retained in [the failed setup report](client-2026-11-03-staged.json); [the corrected staging report](client-2026-11-03-staged-fixed.json) succeeds. No game process was launched during those preparation checks.

Actual client boot, menu/customization controls, world creation, rendered custom terrain and visual parity still require explicit runtime/UI evidence.

## Separate client QA companion

`tools/qa/build_client_qa.py` compiles only `tests/client-runtime` against a selected packaged production JAR and the real 26.2 compile classpath. Its output is a second test-only mod; none of these classes enter the production artifact. `--extra-mod` stages it into a fresh isolated instance and `--client-qa` records its structured result after the real process exits.

```powershell
python tools/qa/build_client_qa.py --java-home PATH_TO_JDK_25 --production-jar build/libs/CANDIDATE.jar
python tools/qa/production_client.py --java-home PATH_TO_JDK_25 --jar build/libs/CANDIDATE.jar --extra-mod build/qa/client-runtime/terraforged-client-qa.jar --client-qa --qa-world PATH_TO_CLOSED_QA_WORLD --launch --wait-seconds 480 --report docs/validation/client-candidate.json
```

The companion uses the actual title and world-creation screens, resolves the registered normal-world `PresetEditor` factory, selects a real built-in preset row, opens the real world-settings page, and sends actual slider mouse events. Eight setting changes must each produce a finite, nonconstant preview. Restoring the original setting must restore the SHA-256 of all 65,536 raw preview height floats. Closing the editor must release its preview texture.

Immediately after the actual Done callback returns, the probe checks the visible screen and whether the applied preset exists in the parent's world-creation context. If validation is incomplete, neither the parent nor editor may be visible and interactive. It then allows vanilla's validation message screen while loading, reports failure confirmation screens, and waits for both the actual parent screen and applied `Preset.KEY` before creating the world. This gate covers the fast-click window that a readiness wait alone would hide.

After the real preset-list layout has rendered, it measures the input hint and three folder/export button labels with the loaded Minecraft font. The hint must fit `EditBox.getInnerWidth()`; each button label must fit its actual width minus the native two-pixel margins. These assertions catch labels that technically resolve but require scrolling or clip in the narrow list column. Measurements record the actual translated text and widget/font widths.

The optional `--qa-world` copies an already closed test world into this new instance's saves; it never opens or modifies the original. The companion opens the copy through `WorldOpenFlows`, handles the real experimental-world warning using its displayed button callback, verifies the integrated server has an active TerraForged preset/context/cache, and moves the synthetic QA player to a spectator camera. It waits for actual world rendering, captures it, then uses the real save/disconnect path and requests client shutdown.

For a complete new-world creation path, pass `--jvm-arg=-Dterraforged.clientqa.createFromUi=true` instead. After the actual Done button exports and reloads the preset datapack, the companion presses the actual Create New World button and handles its experimental warning once. The resulting integrated world must contain the selected seed and active custom generation context. `--jvm-arg=-Dterraforged.clientqa.requireTranslations=true` also requires packaged English labels to resolve. `--qa-vanilla-world PATH` copies a separate vanilla test world and loads it after the custom world has saved; its context must remain vanilla. Both worlds' actual feature-sorter indexes must contain every current biome feature by object identity, and Fabric's global modifier count must remain at its title-screen baseline.

The title-to-world-only control uses `--jvm-arg=-Dterraforged.clientqa.worldOnly=true` with `--qa-world`, bypassing the editor to distinguish generation failures from UI side effects. Each stage persists `qa-output/progress.json`, so a fatal game exception still leaves evidence from stages that completed.

The default quick profile renders six chunks and places the camera 48 blocks above the loaded spawn surface. A closer review profile uses `--render-distance 12 --jvm-arg=-Dterraforged.clientqa.cameraHeight=20 --jvm-arg=-Dterraforged.clientqa.terrainSettleTicks=400`. The camera always loads the player's current spawn chunk before querying surface height and rejects an empty-column result. Actual coordinates, surface height, render distance and settling duration remain in the report; the companion retains its 420-second bound.

All images come from Minecraft's own `Screenshot.grab` framebuffer capture, with hashes, screen classes, dimensions and stage records. This is the fallback after two failed native window-capture attempts. It is evidence of the real game rendering rather than a replacement renderer. The parent reviewer still inspects images; automated assertions do not establish visual parity, all control combinations, other mods or NeoForge support. A completed UI creation path is claimed only for a run that explicitly records the corresponding stage and actual integrated-world seed.

The first packaged runs exposed real defects: [dev2 duplicate background blur](client-dev2-double-blur.json) and [dev3 concurrent lazy noise-holder initialization](client-dev3-preview-holder-race.json). Failures remain documented alongside later corrected results.

The first dev4 world attempt also exposed a [QA driver mistake](client-dev4-driver-warning.json): repeatedly pressing a still-visible warning during its asynchronous transition opened the same world multiple times. The driver now guards each screen identity before activating a callback. A corrected driver then exposed a separate [real biome-decoration failure](client-dev4-biome-decoration.json), also reproduced by the world-only control. These are reported separately to keep driver failures distinct from production defects.

## Optional bounded terrain scene survey

`build_client_qa.py --scene-survey --output build/qa/client-scenes/terraforged-client-scenes-qa.jar` selects a separate Fabric test entrypoint, `tests/client-runtime/terraforged/qa/scene/SceneSurveyMod.java`. The validated editor/full-flow source and NeoForge adapter remain unchanged. Launch this companion with `production_client.py --client-qa --qa-world PATH_TO_CLOSED_CUSTOM_WORLD --render-distance 12`; the world is copied before opening.

The survey verifies the actual custom context and seed 123456789, then directly samples the production heightmap at 256 points spaced 512 blocks apart. Only missing river/coast categories receive a further 256 local refinement points each, for at most 768 samples and a search bound of 20 seconds. Search code contains no chunk/tile requests. Only selected mountain, river and coast areas then request actual full chunks; the ordinary client loads its visible surroundings.

Each candidate is checked against its actual filtered cached tile cell, with terrain flags, biome, surface block and numerical cell fields recorded. Missing categories or changed classifications remain explicitly unverified. Screenshots wait 400 ticks at the actual scene, use Minecraft's framebuffer API, and record the seed, camera/target coordinates and time. The driver releases the mouse and holds the actual first-person spectator pose during settling, then verifies the rendered camera's position and rotation before capture. It saves the copied world and exits within the overall bound of 420 seconds.

The first dev8 survey verified all three terrain categories but its requested server teleport rotation did not ensure a stable rendered camera pose; its mountain and river images are retained as [pose-unverified evidence](client-dev8-scenes-pose-unverified.json). Later camera verification addresses that QA limitation without changing production terrain.
