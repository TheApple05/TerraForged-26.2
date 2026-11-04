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

Final dev23 work still required: complete saved-biome corpus; remaining base and active-shader clients; remaining optimizer/community-ModernFix/content cases; real loopback joins; Linux generation/reload on both loaders; final native-column checks; exclusive three-trial matched base/optimized benchmarks; archived actual images; final acceptance manifest, docs, CI and a single prerelease with both validated loader JARs/checksums. No accepted benchmark matrix has run yet.

The structure probe's original eight-region Repurposed Structures search found no mineshaft. The publisher JAR uses spacing one and frequencies 0.00115/0.0008 for overworld/ocean sets. The test now uses a bounded radius of 64 for that namespace, retaining eight for vanilla, and records actual native starts/blocks. No production structure frequency is changed.

Horizon remains a finite prepared-terrain bridge, verified through exact Java19/current-native comparison. Exact Java20 dense export passed, but its native probe/package is not final after the dev21 source change. The minimal Horizon hook PR is merged as `21679eb68f770d3ac1c2cc1c51a535eea37787ab`; its tree matches tested pin `9dd5c4093977c718c5c0349032fd3a5434e86afc`. Native infinite generation is unfinished; the dependency inventory and next milestone are in `docs/horizon/`. No infinite kernel implementation has been accepted.

The three parallel workers stopped at a service usage limit during this continuation. No retries or model substitutions were used to bypass that limit. Root continued the Java validation and preserved their code, fixtures, failures and review notes. No new release is published while the required gates remain open.
