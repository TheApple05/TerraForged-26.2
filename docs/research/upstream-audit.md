# Upstream source audit

Audit date: 2026-11-03 (America/Los_Angeles). Target: Minecraft Java **26.2**, confirmed by the user. This document records source inspection, not a claim that any upstream branch builds or runs on the target.

## Recommendation

Use ReTerraForged's **1.20.1** branch at `29239f1f164d19dbfcccaca4a8277d64e784207c` as the initial implementation snapshot. Use original TerraForged's **0.2.x** branch at `3ada751ac25cd3eecee5d198870a3abb89734f52` as the feature and visual reference for the classic mod. That original branch declares TerraForged **0.2.16**, Minecraft **1.16.5**. Keep the original 0.3.x and both newer ReTerraForged branches available as research references; importing the newest branch indiscriminately would lose important behavior.

This is a recommendation from code inspection. An isolated reference build and generated-world comparison must still establish the observable baseline. No original release JAR, generated reference world, or 26.2 JAR was tested as part of this audit.

The new GitHub repository can have independent history while preserving source provenance: import the complete selected tree into a fresh repository, record its upstream URL and exact commit, preserve notices, and make later porting changes in separate commits. Do not import upstream `.git` directories, local credentials, generated caches, or Minecraft binaries.

## Pinned upstream state

- [Original TerraForged 0.2.x](https://github.com/TerraForged/TerraForged/tree/3ada751ac25cd3eecee5d198870a3abb89734f52): commit `3ada751ac25cd3eecee5d198870a3abb89734f52`, authored 2022-10-23, declares 1.16.5 / 0.2.16 / Forge 36.2.39. Commit message includes crash protection for extreme low terrain settings, biome-container repair, and disabling async structure search by default. This branch is newer than the available 0.2.13 tag (`a2a6e346dc0dfa33edadeaf274b4d764ba670707`). Do not label the tag as 0.2.16.
- [Original TerraForged default 0.3.x](https://github.com/TerraForged/TerraForged/tree/6dd607ebfd41a3b7274e090b1f2cf565d53d1d44): commit `6dd607ebfd41a3b7274e090b1f2cf565d53d1d44`, authored 2022-06-16, declares 1.19 / 0.4.0-alpha-1 / Java 17 in `common.gradle`, Forge 41.0.22 in `gradle.properties`. The README still says development for 1.18 and explicitly describes a stripped-back, potentially unstable branch. Build metadata is the more precise target evidence.
- [ReTerraForged 1.19.4](https://github.com/racoonman2/ReTerraForged/tree/1.19.4): declares 1.19.4 / 0.0.4, Forge 45.1.2, Fabric API 0.86.1+1.19.4. Older integration reference, not the chosen snapshot.
- [ReTerraForged 1.20.1](https://github.com/racoonman2/ReTerraForged/tree/29239f1f164d19dbfcccaca4a8277d64e784207c): commit `29239f1f164d19dbfcccaca4a8277d64e784207c`, authored 2024-01-09, declares 1.20.1 / 0.0.6. Selected preservation baseline.
- [ReTerraForged default 1.20.2](https://github.com/racoonman2/ReTerraForged/tree/98fd3a28f2ebd638319e797a9138a3418484150f): commit `98fd3a28f2ebd638319e797a9138a3418484150f`, authored 2024-02-16, declares 1.20.2 / 0.0.7. Contains later performance and compatibility work, but also obvious unfinished generation changes described below.
- [ReTerraForged datagen-rework](https://github.com/racoonman2/ReTerraForged/tree/8ad3b857b9afc726dfefdf6f20b8b083c15fe11e): commit `8ad3b857b9afc726dfefdf6f20b8b083c15fe11e`, authored 2026-06-03. This is a separate three-commit history with its entire project nested under `ReTerraForged/`. It declares Minecraft 1.21.10 / mod 0.1.0 / Java 21 / Fabric and NeoForge. It is more recent than the user's initial description suggests, but its feature omissions and changed license make it unsuitable as a drop-in preservation baseline.

All pins above were read from local Git clones fetched from the supplied repositories during this task. Dates are commit author dates; they are not a release chronology or a stability guarantee.

## Why the default 1.20.2 branch is not the initial baseline

At the pinned 1.20.2 commit:

1. [`world/worldgen/heightmap/Heightmap.java:190`](https://github.com/racoonman2/ReTerraForged/blob/98fd3a28f2ebd638319e797a9138a3418484150f/common/src/main/java/raccoonman/reterraforged/world/worldgen/heightmap/Heightmap.java#L190) creates blended terrain and oceans, then unconditionally replaces the terrain variable with `Populators.makePlateau(...)`. This bypasses the just-constructed combined terrain pipeline.
2. [`data/preset/settings/Preset.java:48`](https://github.com/racoonman2/ReTerraForged/blob/98fd3a28f2ebd638319e797a9138a3418484150f/common/src/main/java/raccoonman/reterraforged/data/preset/settings/Preset.java#L48) comments out preset, noise, biome modifier, configured feature/carver, placed feature, biome, dimension, and density-function registry patches. Only noise-settings patching remains active in that method.
3. `BuiltinPresets.makeLegacy1_18()` throws `UnsupportedOperationException("TODO")`; structure settings are commented out of the active preset record. Several new command and surface paths are unfinished.

Later commits do include potentially useful work: structure beardifier corrections, river/aquifer changes, tile-size fixes, World Preview compatibility, terrain-noise adjustments, and generation performance improvements. Evaluate these individually after an executable reference exists. The issues above should not be accepted as desired behavior merely because this is the default branch.

## Why the 1.21.10 datagen rewrite is research only

Paths in this subsection are relative to the nested `ReTerraForged/` directory at the pinned `datagen-rework` commit.

- `common/src/main/java/raccoonman/reterraforged/preset/pages/TerrainOptions.java` contains only an empty terrain page.
- `preset/pages/ClimateOptions.java` forces `CUSTOM_BIOME_GENERATION` to `false` with a TODO and does not add that option to its category.
- `world/worldgen/layer/terrain/continent/ContinentLayer.java` is entirely commented out.
- A source search for `rivermap`, `hydraulic`, `droplet`, and `river` finds no classic river-network or hydraulic-droplet generator. The `river` matches are a biome tag and commented preview coloring. This source tree is not evidence of retaining the legacy hydrology pipeline.
- `data/preset/registry/RTFNoises.java` does retain named landform noises for steppe, plains, dales, hills, torridonian, plateau, badlands, mountains, and mountain chains. This does not establish equivalent continent, river, erosion, biome, or configuration behavior.
- `build.gradle:24` unconditionally loads a local `.env` file, although the shown properties are unused in that script. A reproducible public build should not need local secrets or an absent `.env`.
- Root `LICENSE` is GNU GPL version 3, while Fabric metadata still says MIT. Do not assume the older MIT license covers new code from this branch. Resolve the actual source licensing and distribution choice before copying newer code. Reading its modern API usage is separate from importing its implementation.
- The branch adds Lithostitched integration and optional mod compile dependencies including TerraBlender, Distant Horizons, Oh The Trees You'll Grow, and Tektosyne. These are additional integration and dependency-validation work.

## Feature inventory and preservation boundaries

### Original 1.16.5 reference

The following features are evidenced by source, not yet by runtime results:

- A full custom generator pipeline: `src/main/java/com/terraforged/mod/chunk/TFChunkGenerator.java`, with separate terrain, biome, surface, carver, feature, mob, structure, and stronghold stages under `chunk/generator/`.
- Terrain and world configuration inherited from the Engine settings model. `chunk/settings/TerraSettings.java` adds miscellaneous, dimension, and structure settings and serializes them through a codec. Preset management and in-game GUI live under `chunk/settings/preset/` and `client/gui/`.
- Climate-aware biome selection and custom biome variants. `biome/ModBiomes.java` registers bryce, cold steppe, cold marshland, fir forest, flower plains, frozen lake/marsh, lake, marshland, savanna scrub variants, snowy fir forest/taiga scrub, steppe, stone forest, taiga scrub, and warm beach. These are concrete parity items, not automatically present in modern ports.
- Custom surfaces such as bryce, desert, dunes, icebergs, steppe, stone forest, swamp, plus terrain-column erosion and snow processing.
- Feature modification and template/tree placement through the bundled `FeatureManager/` source tree; Forge API extension events are described by the branch README.
- The core continent/river/landform/filter algorithms live partly in the external Engine project, not entirely in this mod repository. `Engine` is a gitlink pinned to **`ff665ac1dc3f12c7db48324177a3abb80ca9ac61`**. The legacy visual reference requires that exact Engine source and its dependencies or an authentic matching distributed artifact.

### Original modern alpha

The 0.3.x head is a separate generation architecture, not a complete substitute for the classic mod. Its sources contain terrain and climate assets/registries, a custom `worldgen/Generator.java`, continent/noise/erosion code, terrain blending, structure-terrain adaptation, noise caves, and vegetation. `data/ModTerrains.java` defines steppe, plains, two hill types, dales, plateau, badlands, torridonian, three mountain families, dolomites, and non-fancy ridges. This is useful supplemental algorithm evidence.

Its checked-in `client/ui/Previewer.java` is a Swing development preview. Do not mistake that for proof that the classic in-game preset editor survived. Its README explicitly warns of stripped-back functionality.

### Selected ReTerraForged 1.20.1 snapshot

The selected tree has **375 Java source files** under `common/src/main/java` and **106 resource files** under `common/src/main/resources`. Paths below are under `common/src/main/java/raccoonman/reterraforged/`:

- `world/worldgen/cell/heightmap/Heightmap.java` composes terrain regions and mountain chains with deep ocean, shallow ocean, and coast. Its `apply` method performs terrain, river, and climate stages; it does not contain the later unconditional plateau overwrite.
- `world/worldgen/cell/terrain/Populators.java` and `terrain/provider/TerrainProvider.java` retain landform algorithms for steppe, plains, dales, hills, torridonian, plateau, badlands, mountains, and volcanoes, including blending and weighting.
- `world/worldgen/cell/continent/` contains simple, advanced, fancy, and infinite continent implementations. `cell/rivermap/` retains river networks, lakes, and wetlands; `cell/heightmap/Heightmap.java:52` applies the rivermap.
- `world/worldgen/WorldFilters.java` and the tile/filter infrastructure preserve erosion, smoothing, and postprocessing paths. This source presence does not prove tile boundaries or parallel determinism; test them explicitly.
- `world/worldgen/cell/climate/` and `biome/` adapt climate/erosion/weirdness to modern Minecraft biome selection. Exact 1.16.5 custom-biome parity is not established.
- `data/worldgen/preset/settings/Preset.java` has active registry patching for preset, noise, biome modifiers, structure rules, configured features/carvers, placed features, biomes, dimension type, density functions, and noise settings.
- The preset model covers world, caves, climate, terrain, rivers, filters, structures, and miscellaneous settings. `Presets.makeLegacyDefault()` explicitly exists, but its name alone does not establish identical legacy terrain. The 1.20.1 tree also has older duplicate preset classes under `data/worldgen/preset/`; preserve the active imports during migration rather than editing similarly named classes blindly.
- `client/gui/screen/presetconfig/` includes live preview, preset list/import/export, world, climate, terrain, river, filter, cave, and structure pages. The modern Minecraft GUI API migration remains substantial.
- `world/worldgen/feature/template/` and resources retain custom tree/template placement. Inventory all NBT assets and compare their placement before claiming vegetation parity.
- Structure-rule and biome-modifier registries, TerraBlender adapters, and Forge/Fabric platform wrappers are present. Compatibility must be tested with actual target-version dependency artifacts before it is advertised.

ReTerraForged-specific features and modern cave/biome integration must be tracked separately from original-mod parity. Newer world heights, noise routing, aquifers, carvers, registry codecs, spawn finding, and structure integration are not expected to be byte-identical to Minecraft 1.16.5. Any intentional behavioral adaptation needs an explicit decision and visual evidence.

## Dependencies and port risk

Original 0.2.x uses Java 8, ForgeGradle 5.1+, MixinGradle 0.7.32, Forge 36.2.39, a local Engine subproject, a Noise2D compile dependency, JCenter, and the historical TerraForged Maven repository. Original 0.3.x instead pulls `com.terraforged:Engine:0.3.0`. An old build failing to resolve a retired repository is not evidence that terrain algorithms need replacing.

Selected ReTerraForged 1.20.1 declares Java 17, Architectury plugin 3.4-SNAPSHOT, Architectury Loom 1.4-SNAPSHOT, Fabric Loader 0.15.3, Fabric API 0.91.0+1.20.1, Forge 47.2.19, TerraBlender 3.0.1.2, NightConfig TOML 3.6.7, and Shadow 7.1.2. The terrain/noise engine is largely incorporated into the common source tree, reducing reliance on the old TerraForged Engine Maven artifact. None of those old loader/API pins should be silently reused for 26.2.

The common mixin configuration directly targets noise chunks, random state, chunk map, utility pools, biome generation settings, surface system, structures, server lifecycle, and spawn finding; there are optional TerraBlender mixins and a client screen invoker. The access widener and all injection descriptors require review against 26.2 game classes. A build that compiles while disabling the generator mixins would not constitute a working port.

Known inherited limitations include `config/PerformanceConfig.java`, whose `read` method returns defaults and contains a TODO rather than parsing its supplied file. Several `UnsupportedOperationException` methods are platform/Architectury placeholders intended for transformation, not all missing runtime features. Classify each call path before treating every throw or `return null` search hit as a defect.

## Notices and provenance

- Preserve original TerraForged's MIT notice (`Copyright (c) 2020 TerraForged`) and source headers, including later 2021 notices.
- Preserve selected ReTerraForged's MIT notice (`Copyright (c) 2023 ReTerraForged`). The new repository should clearly describe itself as an unofficial continuation and credit upstream contributors.
- **File-level exception:** selected `common/src/main/java/raccoonman/reterraforged/client/data/LanguageProvider.java` begins with `Copyright (c) Forge Development LLC and contributors` and `SPDX-License-Identifier: LGPL-2.1-only`. Do not describe the entire imported tree as uniformly MIT. Preserve the header and include the applicable license text and component attribution; verify packaging boundaries before distribution. This is an observed source notice, not a legal determination.
- The GPLv3 `datagen-rework` branch is a separate provenance category. No code from that branch should be labelled as MIT without source-specific evidence.
- Generated Minecraft sources, downloaded game assets, and third-party dependency binaries belong in ignored caches, not this independent source repository.

## Small exact-source validation slice

For initial Java 25 algorithm characterization without Minecraft or invented stubs, these selected 1.20.1 files compile using only the JDK:

- `common/src/main/java/raccoonman/reterraforged/world/worldgen/noise/NoiseUtil.java` (nested vector records and math/hash tables; no external imports).
- `common/src/main/java/raccoonman/reterraforged/world/worldgen/util/Seed.java`.
- `common/src/main/java/raccoonman/reterraforged/world/worldgen/util/PosUtil.java`.
- `common/src/main/java/raccoonman/reterraforged/world/worldgen/util/FastRandom.java`.

Compile these exact imported sources in a separate characterization task and compare recorded outputs across supported JDKs. Cover negative coordinates, signed integer extremes for position packing, seeded sequences, hash outputs, and float-bit results. This is algorithm characterization only; it does not validate complete terrain or a mod artifact.

`NoiseUtil.floor(float)` subtracts one for all negative values, including exact negative integers. `PosUtil.pack(float,float)` casts floats to integer coordinates, whereas `packf` preserves float bits. Record these inherited semantics before changing them: a mathematically motivated cleanup can alter noise lattice selection and river/terrain output. A deliberate defect fix must be versioned and measured, not hidden inside a mechanical migration.

Noise module implementations such as `noise/module/Simplex.java` are not standalone: they depend on Mojang codecs, the `Noise` interface, registry wiring, and Minecraft's serialization interfaces. Do not replace those dependencies with pretend implementations to obtain a green test.

## Evidence still required

- Isolated packaged baseline runs with authentic dependencies and frozen presets.
- Original/selected-reference screenshots and maps with seed, coordinates, exact settings, render distance, game/mod versions, and capture method recorded.
- Per-feature parity acceptance criteria, including the original custom biomes and surfaces that modern branches may omit.
- 26.2 client and dedicated-server packaged-JAR smoke tests, world creation, save/reload, structures, and command/configuration coverage.
- Tile/chunk-boundary comparisons, generation-order determinism, parallel generation stress, negative coordinates, and configuration boundary cases.
- Generation timing and memory measurements using identical seed/region/settings and clearly separated warmup/cache effects.

No completion claim should be based solely on this source audit or a successful Java compilation.
