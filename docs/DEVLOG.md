# Development log

Dates use America/Los_Angeles. This log records outcomes and evidence, not an assertion that the entire port works.

## 2026-11-03 — project initiation

- User authorized implementation, an independent GitHub repository, and ongoing documentation.
- Workspace was empty. Created a fresh Git history and private repository `TheApple05/TerraForged-26.2`; GitHub's creation response reports `fork: false`.
- Verified Mojang's version manifest lists `26.2` as a release and does not list `1.26.2`. User explicitly confirmed the 26.2 target.
- Cloned both upstream repositories under ignored `.reference/` for branch and history inspection.
- Started independent upstream/feature and target/toolchain audits.
- Original default branch README warns that it is stripped back. Early inspection found experimental terrain overrides in ReTerraForged's 1.20.2 branch; choosing a baseline requires source inspection rather than taking the newest branch automatically.
- Used the authorized DeepSeek Delegator for a bounded regression-checklist draft. Codex reviewed and adapted that draft into the validation protocol. Worker output is not evidence of passing tests.

### Baseline selection and import

- Selected ReTerraForged 1.20.1 / 0.0.6 at `29239f1f164d19dbfcccaca4a8277d64e784207c` after inspecting all branches. Imported the complete tree except upstream README and ignore rules in commit `b593342`.
- Original TerraForged 0.2.x at `3ada751ac25cd3eecee5d198870a3abb89734f52` declares 1.16.5 / 0.2.16 and serves as the classic reference. Its Engine dependency must be pinned when constructing the original runtime reference.
- Rejected the 1.20.2 plateau override and mostly disabled preset patching as the preservation baseline. The newer 1.21.10 `datagen-rework` omits important hydrology/configuration paths and changes licensing; no code from that branch was imported.
- Preserved original/revival MIT notices and the file-level LGPL-2.1-only LanguageProvider notice, with component license text and source provenance.
- Created GitHub issues [#1](https://github.com/TheApple05/TerraForged-26.2/issues/1) through [#4](https://github.com/TheApple05/TerraForged-26.2/issues/4) for reference parity, 26.2 generation migration, customization, and release validation.
- Chose Fabric as the first integration target using Java 25. Forge source remains in history/tree as a reference and is not advertised as supported on 26.2.

### First algorithm characterization

- Implemented dependency-free compilation of four unchanged production utility classes, with frozen reference outputs independently extracted from the pinned upstream commit.
- Passed 2,167 records in four separate Java 25 processes (serial twice, shuffled, four-worker parallel); the integrator reran verification successfully. Fixture SHA-256: `8070efecb62e736c3d22c64e8e22922ac468c0350804d3f8e0bfe615f3d35dc5`.
- Verified that a hash-kernel mutation in a temporary copy is rejected. No production algorithm change was made.
- Recorded inherited `NoiseUtil.floor(-1.0f) == -2` semantics rather than silently changing deterministic generation.
- Added explicitly scoped Linux/Windows CI for these kernel checks. CI outcome is separate from the verified local result.

Next: finish the full-source target compilation diagnostic, then migrate actual 26.2 API adapters using that evidence.

### Full source migration

- Pinned and resolved Java 25, Gradle 9.5.1, Fabric Loom 1.17.21, Loader 0.20.1, Fabric API 0.161.0+26.2 and target compile-only dependencies. Regenerated the wrapper and checked its published checksum.
- Recorded successive coherent full-source inventories: 432 errors in 388 sources initially, then 251 errors, then 8 errors after component integration, followed by **zero errors in all 393 active source files**. Error counts are compiler-pass inventories, not feature-parity percentages. All source roots remain included.
- Implemented explicit Fabric platform bindings, resource/bootstrap migration, map codec dispatch, density child traversal, registry cloning/export context, dimensions, vegetation/template/provider APIs, client extraction rendering and preview texture lifetime, and current chunk lifecycle hooks.
- Enabled a new reviewed target access widener; current non-obfuscated Loom requires the `official` namespace. The legacy access widener remains reference-only. Target sources were generated into ignored caches for signature review.
- The density adapters pass 235 assertions using actual 26.2 classes; the integrator independently reran them successfully. Observed inherited serialization defects are documented separately from arithmetic preservation.
- The migrated legacy random patch feature matches actual Minecraft 1.20.1 execution in 216 cases, including 6,948 nested placements, success flags, and final RNG state. Nine dimension-height configurations encode correctly; default complete dimension data matches actual 26.2 vanilla data.
- Adapted tree-soil eligibility explicitly because the modern vanilla dirt tag narrowed. Restored per-biome decorator identifier lookup and invalidated both feature caches after biome modifications.
- Migrated all editor pages and live-preview source APIs. Actual UI appearance, controls, texture updates, world creation and dedicated-server behavior remain runtime gates.
- Experimental packaging includes component license notices and requires `-PexperimentalPackage` for isolated validation. No release is being announced.

Next: audit effective runtime injections, boot the exact packaged artifact in a disposable production server, exercise real preset export/custom generation and save/reload, and inspect the client visually.

### Packaged generation and runtime corrections

- User clarified that practical 26.2 modpack use is the priority, with both Fabric and NeoForge required. Started a loader-neutral service boundary and an independent NeoForge module; neither a compile nor Fabric runtime result will be labeled as NeoForge validation.
- Reviewed effective 26.2 constructor, chunk-stage, spawn, reload, and optional TerraBlender injection signatures. Added scoped spawn-search context cleanup and 36 assertions, retaining the inherited unsupported islands preference in the feature inventory.
- Installed official game/loader dependencies into disposable production QA instances and verified downloaded hashes. The actual Fabric package boots and exports a custom preset datapack using loaded server registries.
- The first custom generation attempt hit the watchdog. Preserved the failure and identified a production cache self-deadlock caused by ForkJoin cooperative waiting while holding a non-reentrant lock. Corrected `CacheEntry` publication; 312 assertions reproduce the original failure and validate shared future reads, concurrency, failure propagation, and closing.
- Fixed a real slider event regression introduced by the API migration. Retained the failing report and passed 29 actual event assertions after correction.
- Packaged corrected dev.2 (`94471bb542a308982bfb29c7340ed14dbf3494922cfd9f935c5f32b0485e80bd`). Two fresh custom-world generation runs and one save/reload run pass under the normal watchdog, with clean saves and process exit. Fresh runs differ at one tree-canopy column on a chunk boundary; terrain cell fields match. Broader checks are investigating decoration completion rather than treating the aggregate digests as equal.
- Production client review was deliberately aborted after two desktop capture failures. The client had reached its native window; no visual acceptance is claimed. Added a separate actual-game screenshot companion for continued review.
- Recorded the 17 classic custom biome IDs absent from the selected revival baseline, distinguishing inherited gaps from current port failures. See [feature inventory](FEATURE-PARITY.md) and [runtime summary](validation/runtime-summary.md).
