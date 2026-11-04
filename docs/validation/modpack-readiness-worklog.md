# Classic restoration, modpack validation and Horizon work

Started 2026-11-04 (America/Los_Angeles), following the user's request to restore all 17 classic biome variants, expand compatibility/performance testing on Fabric and NeoForge, and create a Horizon port.

This record tracks work in progress. The previous accepted experimental delivery remains dev11 until a newer exact artifact passes its required runtime checks.

## Acceptance work

- Register and export all 17 real biome definitions; preserve documented original parent changes while explicitly naming modern adaptations.
- Keep classic selection separate from custom vegetation, preserve legacy presets, cave biomes and third-party biome ownership, and test actual source codec/save/reload behavior.
- Restore masked Bryce/stone-forest pillar shapes. Verify the original terrace scalar math independently; record the geology and generation-stage adaptations rather than claiming identical unavailable Engine behavior.
- Test checksum-pinned rendering, optimization, biome and structure stacks on both loaders. Include an active shader pack, actual chunk generation/save/reload, structures and a real disposable loopback client/server join.
- Run the predeclared [performance protocol](performance-protocol.md): fixed workloads, repeated matched generator controls, heap/process memory and GC records, normal watchdog and saved-column digests. No universal pack, player-capacity or FPS claim follows from local chunk timings.
- Keep Java QA companions and third-party artifacts out of distributable production JARs.
- Build Horizon integration separately against its actual Rust/native-mod API and 26.3 pin. A prepared-world bridge must be described as bounded and cannot be advertised as a complete infinite native generator.

## Results and retained failures

- [Classic pillar arithmetic](classic-pillar-math.json): 16,418 raw-float terrace outputs match independently compiled pinned Noise2D source; 16,494 scalar/boundary assertions passed. This is not an in-game surface or visual verdict.
- [Compatibility artifact inventory](compatibility-26.2-artifacts.json): publisher version IDs and hashes recorded for the expanded mod/shader matrix. Downloading an artifact is not a compatibility pass. ModernFix community builds are explicitly identified as community ports.
- [dev12 package attempt](dual-loader-dev12-build.json): correctly rejected by the coherence gate because production sources changed during the builds. Neither package is an accepted test or delivery candidate.
- [dev13 packages](dual-loader-dev13-build.json): both loaders compiled with unchanged source/configuration snapshots. Initial [Fabric](server-fabric-dev13-export.json) and [NeoForge](server-neoforge-dev13-export.json) exports then failed on the same modern codec constraint: a placed-feature list mixed registered and inline holders. The biome bootstrap is being corrected to use registered placed-feature references. No benchmark numbers from this failed candidate are accepted.

The exact original Engine git object is still unavailable. The modern selector policy and reconstructed geological layer layout must remain distinguishable from authenticated classic equations and registry definitions.
