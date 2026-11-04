# Development candidate installation

Target: **Minecraft Java 26.2**, using **Java 25**. Current candidate: **0.1.0-dev.11+26.2**. These are development candidates for new test worlds; a stable release and broad modpack compatibility certification are still pending. See [runtime evidence](validation/runtime-summary.md) and [known feature gaps](FEATURE-PARITY.md).

## Choose the matching loader

- **Fabric:** install Fabric Loader 0.20.1 and Fabric API 0.161.0+26.2. Add the Fabric `terraforged-revival-â€¦+26.2.jar` to the instance's `mods` folder.
- **NeoForge:** install NeoForge 26.2.0.93. Add `terraforged-revival-â€¦+26.2-neoforge.jar` to `mods`. Fabric API is not a NeoForge dependency.

Install one loader's TerraForged artifact per instance. The repository's historical `forge/` directory is reference source, not another supported 26.2 loader.

## Create a custom world

1. Open **Singleplayer â†’ Create New World â†’ World**.
2. Keep the default world type and open **Customize** to enter **Presets & Defaults**.
3. Select **Default**, then use **>>** to edit it. The remaining pages cover world, caves, climate, terrain, rivers, filters, structures, and miscellaneous settings.
4. Set the seed and settings, then select **Done**. Minecraft shows its validation screen and returns when the generated preset datapack is ready.
5. Create the world and accept Minecraft's settings confirmation if shown.

The actual Default editor, seed, repeated continent-scale changes, Done/export/reload, world creation, terrain rendering, and subsequent normal-world loading have production QA records. Other presets and every combination of controls still need coverage. The live preview is a diagnostic map; in-game terrain remains the visual acceptance reference.

Saved preset JSON lives in `config/reterraforged/presets`. The **Preset Folder** button opens it. **Export Pack** writes a datapack ZIP under `config/reterraforged/exports`; **Export Folder** opens that directory.

## Dedicated servers and modpacks

Create and configure a new world through a matching-loader client, then copy its complete world folder to a stopped matching-loader server and set `level-name` to that folder. Keep the world datapacks and matching mod dependencies together. Do not put the QA companion JARs in a modpack.

Normal worlds remain available when a TerraForged preset datapack is absent. Switching settings does not regenerate existing chunks. Preset exports use loader-native biome-modifier registry paths; cross-loader migration of an exported pack is not yet validated.

The pinned tested content stack is Biomes O' Plenty **26.2.0.0.28 beta**, GlitchCore **26.2.0.0.0 beta**, and TerraBlender **26.2.0.0.2** on each matching loader. The mod contains a narrowly guarded correction for that TerraBlender release's surface serializer registration. See [stack evidence and exact hashes](validation/modpack-biomesoplenty-dev10.md). These checks do not certify every biome/content mod that uses TerraBlender. Add a modpack's other world-generation mods incrementally and use the [issue tracker](https://github.com/TheApple05/TerraForged-26.2/issues) for failures, including loader/mod versions, seed, preset, and the crash report.

## Build from source

Use a full Java 25 JDK. On Windows the helper selects Java for this invocation without changing global settings:

```powershell
& .\tools\gradle.ps1 -GradleArgs @('jar','-PexperimentalPackage','--console=plain')
& .\tools\gradle.ps1 -GradleArgs @('--project-dir=neoforge','jar','-PexperimentalPackage','--console=plain')
```

Fabric output is under `build/libs`; NeoForge output is under `neoforge/build/libs`. Each package includes license notices. `tools/qa/package_candidates.py` records source/configuration snapshots and SHA-256 values for a coherent two-loader build. The GitHub packaging workflow retains experimental artifacts and does not perform client visual acceptance.

To regenerate or check English translations from the actual Java provider, use `python tools/generate_language.py --java-home <JDK-directory>`; add `--write` to update the checked-in asset. The generator requires the resolved target classpath and compiled production classes from a previous build.
