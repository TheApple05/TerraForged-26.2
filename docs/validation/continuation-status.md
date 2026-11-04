# Current validation checkpoint

Updated during the 2026-11-04 continuation. This is a work-in-progress checkpoint, not a release acceptance record.

Current candidate: **0.1.0-dev.22+26.2**. Default source version and explicit package override both agree. Both coherent package builds pass with unchanged inputs.

- Fabric SHA-256: `3e16001ac365c7652efcdf6b24d543be1eab1fe57065acf4caee2e7be0990447`.
- NeoForge SHA-256: `563717687b46c4d4aff497bfa93b55c40bf9100c28e3408f3ce7ce1e337573f4`.
- Full JDK: `C:/Users/Main/AppData/Local/Programs/Java/jdk-25.0.5+8`.
- Final server companions: `build/qa/runtime-dev22/terraforged-runtime-qa.jar` and `build/qa/neoforge-runtime-dev22/terraforged-neoforge-runtime-qa.jar`.
- Final client companions: matching `build/qa/client-runtime-dev22/` and `build/qa/neoforge-client-runtime-dev22/` directories.

Dev20 passed both base and full shader/content clients, both optimized dedicated stacks, 795 cache checks per loader, positive pillar generation/reload and all 36 native-column comparisons against dev18. Its complete biome corpus then failed the warm-beach source agreement on cold reload. Dev21 fixed that cold query but made global sampling and stronghold ring planning excessively expensive. Decision 0022 records the rejected startup attempts and dev22's scoped correction. Both exact dev22 cold reloads and fresh-world 801-check cache/scope probes now pass with clean exits.

Final qualified targets are in `classic-biome-dev20-qualified-targets.json`; the corrected water positions are in `classic-biome-dev20-qualified-water-targets.json`. Preserve the earlier rejected origins and failures. The corpus runner now omits the expensive old sparse approximate survey; it still requires all seventeen actual stored/reloaded variants and both positive pillars.

Final dev22 work still required: two-loader exports and full corpus; base and active-shader clients; optimized/community-ModernFix/content-structure matrices; real loopback joins; Linux generation/reload on both loaders; final native-column checks; exclusive three-trial matched base/optimized benchmarks; archived actual images; final acceptance manifest, docs, CI and a single prerelease with both validated loader JARs/checksums.

The structure probe's original eight-region Repurposed Structures search found no mineshaft. The publisher JAR uses spacing one and frequencies 0.00115/0.0008 for overworld/ocean sets. The test now uses a bounded radius of 64 for that namespace, retaining eight for vanilla, and records actual native starts/blocks. No production structure frequency is changed.

Horizon remains a finite prepared-terrain bridge, verified through exact Java19/current-native comparison. Exact Java20 dense export passed, but its native probe/package is not final after the dev21 source change. The minimal Horizon hook PR is merged as `21679eb68f770d3ac1c2cc1c51a535eea37787ab`; its tree matches tested pin `9dd5c4093977c718c5c0349032fd3a5434e86afc`. Native infinite generation is unfinished; the dependency inventory and next milestone are in `docs/horizon/`. No infinite kernel implementation has been accepted.

The three parallel workers stopped at a service usage limit during this continuation. No retries or model substitutions were used to bypass that limit. Root continued the Java validation and preserved their code, fixtures, failures and review notes. No new release is published while the required gates remain open.
