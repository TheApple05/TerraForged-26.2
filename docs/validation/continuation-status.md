# Current validation checkpoint

Updated during the 2026-11-04 continuation. This is a work-in-progress checkpoint, not a release acceptance record.

Current candidate: **0.1.0-dev.23+26.2**. Default source version and explicit package override both agree. Both coherent package builds pass with unchanged inputs.

- Fabric SHA-256: `1c9e5e7e8d286c145a647f466b2f908cda721d10672c40949864f1015570e886`.
- NeoForge SHA-256: `61ed42e28fbebbb1166b2a757e82ddc247038b70b19cdbdfc3a70a6621e80fce`.
- Full JDK: `C:/Users/Main/AppData/Local/Programs/Java/jdk-25.0.5+8`.
- Final server companions: `build/qa/runtime-dev23/terraforged-runtime-qa.jar` and `build/qa/neoforge-runtime-dev23/terraforged-neoforge-runtime-qa.jar`.
- Final client companions: matching `build/qa/client-runtime-dev23/` and `build/qa/neoforge-client-runtime-dev23/` directories.

Dev20 passed both base and full shader/content clients, both optimized dedicated stacks, 795 cache checks per loader, positive pillar generation/reload and all 36 native-column comparisons against dev18. Its complete biome corpus then failed the warm-beach source agreement on cold reload. Dev21 fixed that cold query but made global sampling and stronghold ring planning excessively expensive. Decision 0022 records the rejected startup attempts and dev22's scoped correction. Both exact dev22 cold reloads and fresh-world 801-check cache/scope probes now pass with clean exits.

Final qualified targets are in `classic-biome-dev20-qualified-targets.json`; the corrected water positions are in `classic-biome-dev20-qualified-water-targets.json`. Preserve the earlier rejected origins and failures. The corpus runner now omits the expensive old sparse approximate survey; it still requires all seventeen actual stored/reloaded variants and both positive pillars.

Dev22 completed all seventeen FULL/stored/source/save/cold-reload cases and both base/shader clients. Its NeoForge combined content/optimizer server then caught modified-parent biome data being exported and modified twice. Decision 0023 records the original-data API correction. The exact dev23 combined NeoForge content/structure export, generation and reload now passes; dev23 base exports and the Fabric base client also pass.

Dev23 functional qualification now passes: complete seventeen-biome corpora on both loaders; both base and both shader/content clients; all three compatibility cases per loader; actual combined-stack loopback joins; Linux generation and cold reload; all36 native-column replays per loader; 3,511 cache-lifecycle assertions per loader; and 30 archived byte-exact images. The aggregate `dev23-functional-acceptance.json` binds 67 evidence files. Main installation/validation/feature/porting docs have been refreshed.

Controlled benchmark session `39485` is active through `build/qa/run_quiet_benchmarks.py`. It waited for unrelated Cargo compilation to stop and began at 2026-11-04T13:17:06Z. The complete base matrix passes all12 matched trials and four cold reloads, with no compiler interference or budget violations. Custom new-region medians are about5.7-6.0 seconds per64 requested FULL chunks, versus about2.4-3.0 for matched vanilla controls; custom peak process resident memory is about1.3GiB. The optimization matrix is running. Do not start another JVM, Cargo/rustc or heavy workload until the controller exits. All final timings and control scope must remain exact.

Remaining Java gates: finish optimization matrix; inspect host/compiler-interference record; run exact23 opt-in `probeCacheReentry=true` checks on both loaders (the existing23 scope checks have798 assertions because they omit those three nested assertions); run complete aggregate acceptance; finish docs/CI; assemble and publish a single clearly labeled prerelease with both unchanged loader JARs/checksums. No new release is published yet.

The structure probe's original eight-region Repurposed Structures search found no mineshaft. The publisher JAR uses spacing one and frequencies 0.00115/0.0008 for overworld/ocean sets. The test now uses a bounded radius of 64 for that namespace, retaining eight for vanilla, and records actual native starts/blocks. No production structure frequency is changed.

Horizon exact23 export, current-native finite probe and deterministic `.hmod` packaging pass. `horizon-current-dev23.json` compares884,736 blocks,13,824 quartbiomes and2,304 heightmap/LOD columns. The minimal Horizon hook PR is merged as `21679eb68f770d3ac1c2cc1c51a535eea37787ab`; its tree matches tested pin `9dd5c4093977c718c5c0349032fd3a5434e86afc`. No game/resource downloads occurred. Native infinite generation is unfinished.

The isolated native-core scalar/vector/Perlin/Simplex foundation matches all34,976 independently captured Java records (56,080 values) in debug and release. New Seed/PosUtil/FastRandom modules and the frozen-utility checker have been drafted but deliberately remain uncompiled while timing runs. Run cargo fmt and `verify_native_utilities.py` only after the benchmark session closes; then rerun foundation against current complete crate in a new report and add the utility CI command once verified. Do not mislabel these utilities or the finite bridge as a complete cell/tile or infinite generator. The full dependency inventory/milestone is in `docs/horizon/`.

The three parallel workers stopped at a service usage limit during this continuation. No retries or model substitutions were used to bypass that limit. Root continued the Java validation and preserved their code, fixtures, failures and review notes. No new release is published while the required gates remain open.
