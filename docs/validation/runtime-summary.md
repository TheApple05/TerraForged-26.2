# Runtime validation status

Current **dev23** passes [functional acceptance](dev23-functional-acceptance.json) for both exact Java26.2 loader JARs. This binds 67 evidence files and 30 byte-exact native captures. [Readiness and limits](dev23-readiness.md) provide the detailed links.

All 17 natural variants, positive Bryce/stone pillars, saved holders/source agreement and cold reload pass on both loaders. Both real base and active-shader clients complete the editor/preview/world/save/subsequent-vanilla flow. Dedicated optimization, community ModernFix and combined content/structure cases pass with their own exported packs, and real combined-stack client joins synchronize all variants and receive terrain. Both loaders also pass WSL Linux generation/reload.

Exact final JAR cache lifecycle checks pass 3,511 assertions per loader with zero retained owners/polls. Real-world cell caches retain 24 raw fields and reduce repeated native-column calls from 384 to four. Independent dev18/final native-column replays match all 36 actual inputs per loader.

Both controlled performance matrices pass all 24 fresh trials and eight cold reloads with no compiler interference or budget violations. [Complete scoped acceptance](dev23-acceptance.json) binds 104 evidence files and 30 images. [Measured timings](dev23-performance.md) retain all trials and matched controls. The next experimental release is being assembled.

[Native visual review](visual-review-dev23.md) confirms visible pillars and nearby rendered shader terrain. Brief captures show incomplete distant LOD edges, so far-distance quality and FPS remain unaccepted. User aesthetic acceptance and original-version visual parity are separate gates.

Horizon26.3 remains separate: the exact dev23 finite bridge passes native block/biome/heightmap/LOD and failure checks, and isolated Perlin/Simplex modules match independent Java raw-bit fixtures. Infinite generation and full native client/save acceptance remain unfinished.

## Historical dev11 delivery record

The following older record remains useful history. It does not replace dev23 qualification.


These records use disposable **non-development** Minecraft Java 26.2 installations on Windows with Java 25. Existing user worlds, launcher profiles, accounts and global Java settings were not changed. Machine records retain UTC timestamps; the development log uses America/Los_Angeles.

## Current candidate: dev11

- **Fabric:** `terraforged-revival-0.1.0-dev.11+26.2.jar`, SHA-256 `9c72b55fe91202ba85bbfc1489eebc91b5a1a6b4412373130b73230659d792f0`. Loader 0.20.1, Fabric API 0.161.0+26.2.
- **NeoForge:** `terraforged-revival-0.1.0-dev.11+26.2-neoforge.jar`, SHA-256 `7f1ba4f0dd38259b4995fc445afeae637aded12144cb813affc16b5c83158845`. NeoForge 26.2.0.93.

[Coherent build evidence](dual-loader-dev11-build.json) records the full source sets, actual toolchains, unchanged before/after snapshots, exactly one loader provider, 277 translations, license notices and no packaged QA classes. Separate QA companions are never part of the production mod. [Installation instructions](../INSTALLATION.md) distinguish the two artifacts.

[Independent CI evidence](github-actions-2026-11-04.json) confirms both Ubuntu package jobs and Linux/Windows kernel/cache jobs passed for the committed source. CI artifacts are separate from the exact Windows runtime-hashed JARs. Run `python tools/qa/verify_candidate_evidence.py` with those local JARs present to verify both artifact hashes, all four client workflows, 28 unchanged archived PNGs and the dedicated-server evidence. [Delivery metadata](dev11-delivery.json) records the local two-loader test bundle and checksum.

## Client and world creation

Both loader clients exercise the actual registered editor, real translated widgets, four Font/inner-width checks, eight slider updates, a 65,536-cell preview digest restored after resetting settings, the real Done callback, datapack export/reload and Create New World callback. They generate/render/save custom seed `123456789`, then load/render/save vanilla seed `8675309` in the same process with no custom context left behind.

- [Fabric base workflow](client-dev11-full-flow.json) and [NeoForge base workflow](neoforge-client-dev11-fullflow.json).
- [Fabric content-stack workflow](client-dev11-biomesoplenty-full-flow.json) and [NeoForge content-stack workflow](neoforge-client-modpack-dev11-fullflow.json).
- Base custom/vanilla feature-sorter identity checks cover 2,702 / 2,587 references. The content stack covers 5,450 / 5,217 across 114 possible biomes. Native modifier assertions distinguish this preset's namespace from legitimate entries owned by other mods.
- Exact installed JAR origin/hash, non-development mode, actual game/dependency versions, clean process exit and critical runtime error checks are recorded per run. Real Screenshot API PNGs are retained under `images/`.

The stricter immediate-Done probe [reproduces a dev10 race](client-2026-11-04-dev10-post-done-before-fix.json): the parent Create World screen was visible while the applied preset was absent. Dev11 retains vanilla's loading screen until actual settings are ready; [decision 0015](../decisions/0015-preset-validation-screen-ownership.md) records the fix and acceptance gate. This is separate from the earlier tests that merely waited before trying to create a world.

## Dedicated servers and representative content stack

The pinned stack is Biomes O' Plenty **26.2.0.0.28 beta**, GlitchCore **26.2.0.0.0 beta**, and TerraBlender **26.2.0.0.2**, with matching loader builds. [Artifact metadata](modpack-biomesoplenty-26.2-artifacts.json) contains published hash verification, SHA-256 values, exact URLs and decisive in-JAR dependency declarations. BOP's JAR requires TerraBlender even where marketplace dependency lists omit it.

The [dev10 complete-stack checks](modpack-biomesoplenty-dev10.md) export 2,745 Fabric / 2,749 NeoForge files and consume each loader's own newly generated pack. All 68 registered BOP biome entries are written, 59 BOP biomes remain in the overworld source, and all 54 sampled columns contain actual BOP `overgrown_greens` or `prairie` biomes. Actual custom preset/context/tile-cache, corrected serializer identity, finite uniqueness, flushed save and clean exit pass. Each saved world's recorded digest matches its disk reload.

[Exact dev11 dedicated checks](modpack-biomesoplenty-dev11-summary.json) repeat generation/reload with the new JAR hashes and qualified same-loader dev10 packs, checking every consumed file hash. [Class-byte comparison](dev10-dev11-class-comparison.json) proves only the client preset screen changed; all backend classes are identical. These are new dev11 server executions, not renamed dev10 results. The normal 60,000 ms watchdog stays enabled.

The first real content stack exposed a TerraBlender registration defect that loading TerraBlender alone did not catch. [Decision 0014](../decisions/0014-terrablender-material-rule-codec.md) retains failed dev8 exports, a failed dev9 injection selector, exact bytecode diagnosis and the narrow successful registration-boundary correction. Original surface rules are preserved rather than stripped or silently skipped.

## Terrain, concurrency and visual scope

[Expanded Fabric boundary checks](server-stress-summary.md) finish 144 FULL chunks before sampling 576 columns. Raw terrain fields/classifications and biome observations agree across opposite request orders; each saved recorded digest survives reload. The corrected later [dev8 TerraBlender workload](server-2026-11-04-dev8-terrablender.json) and [reload](server-2026-11-04-dev8-terrablender-reload.json) also exercise 144 FULL chunks, finite actual sampler/target uniqueness and normal-watchdog saves. Workload times are observations, not controlled performance benchmarks.

Complete post-feature block determinism is **not** established. Trees, coral, ores and dripstone vary with feature order, and an explicit vanilla-generator control also shows these interactions. [Content-stack cross-loader observations](modpack-biomesoplenty-dev10-cross-loader-observations.json) preserve the exact 54-column comparison: terrain fields/classifications agree, while some surface observations and one biome-at-observed-height ID differ. No cross-loader block/biome identity claim follows.

The [bounded scene survey](client-dev9-scenes.json) searches 256 direct heightmap samples, then generates actual selected areas and verifies filtered mountain/river/coast classifications and rendered camera poses. [Mountain](images/client-dev9-scenes/08-mountain.png) and [river](images/client-dev9-scenes/09-river.png) views were inspected. The snowy coast view has weak shoreline framing, so clear coastline visual coverage remains unverified. These dev9 captures are labeled separately from final dev11 client evidence. No original-version visual comparison or user acceptance is claimed.

## Retained failures and remaining gates

The records include the original watchdog/cache self-deadlock, slider callback/precision defects, duplicate screen blur, lazy-holder preview race, stale Fabric feature sorting and cross-world modifier ownership. Focused regressions and actual corrected-package runs accompany each correction. Desktop capture failures and QA-driver/camera mistakes are explicitly separated from mod failures. See the development log and numbered decisions.

Still open: full classic restoration (including 17 missing classic custom biome variants), every preset and control extreme, imported/invalid configuration diagnostics, original reference screenshots/user review, multiplayer, Linux game runtime, other biome/structure/performance/render mod combinations, controlled performance budgets, and existing classic-world migration. These experimental builds are a working modern port within the tested scope, not a stable release or universal modpack certification.
