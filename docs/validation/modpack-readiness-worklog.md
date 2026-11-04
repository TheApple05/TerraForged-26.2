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

## Additional runtime findings

Dev14's complete exports exposed a shared swamp/marsh feature-order cycle during real generation. Merely assigning the same placed feature a different registered name did not resolve it: 26.2 `PlacedFeature` is a record, and native sorting uses structural equality. The next candidate uses a single-count placement marker on each alias; a constant count of one leaves the placement sequence and random consumption unchanged while making the feature value distinct. The [retained failure analysis](classic-biome-dev13-dev14-failures.md) distinguishes this shared issue from optimization-mod compatibility.

The same round found loader-specific dimension priority and the client's selected-dimension reload behavior. [Decision 0018](../decisions/0018-biome-source-dimension-handoff.md) records both counterexamples and the dev15 missing-dimension-view failure. The strengthened client gate now checks the actual source and all 17 possible holders, in addition to the prior UI/cache gates.

The [cache lifecycle regression](cache-lifecycle-2026-11-04.json) independently reproduces the upstream strong-owner/map-retention leak and passes 3,506 candidate assertions. The old dev14 client flow confirms bounded preview ownership and zero retained custom-world caches in a subsequent vanilla world, but its old biome-source count means it is not a restored-biome acceptance run.

Dev17's unforced biome survey reaches all 17 variants on both loaders, at the same 51,271-point stopping bound. The earlier dev16 selector reached only 15 because its regional choice reused the climate-correlated region ID directly. A separate deterministic hash of that region ID restores independent regional choices while remaining constant within each region. All nine supported built-in/independent-toggle export cases pass on both loaders.

Generating actual pillar chunks exposed a separate FEATURES-region boundary error: fuzzy biome lookups at a chunk edge can ask for a chunk outside the available region. Dev18 uses exact quart-biome queries for the center and taper probes, and the current ocean-floor height after earlier features. The rejected dev17 report is retained. A subsequent distant-chunk workload exposed repeated terrain computation during native structure-column sampling; that performance/correctness cache fix is being qualified for dev19. The watchdog remains at its normal 60 seconds.

[Decision 0019](../decisions/0019-terrablender-classic-source-lifecycle.md) records the content integration issue detected by stronger checks. The dev16 shader rehearsal has useful renderer and shader evidence but omitted BOP from its wrapped custom source, so it is excluded from content acceptance. Exact dev18 base clients pass on both loaders with actual selected/integrated classic sources, all 17 holders, bounded preview caches, an inner region of fully rendered chunks, clean saves, and zero custom caches in the subsequent vanilla world. Final candidate and content-stack runs remain separate gates.
