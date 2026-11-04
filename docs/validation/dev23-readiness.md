# Dev23 modpack evaluation record

Candidate **0.1.0-dev.23+26.2**, frozen by [coherent build](dual-loader-dev23-build.json). Fabric SHA-256 is `1c9e5e7e8d286c145a647f466b2f908cda721d10672c40949864f1015570e886`; NeoForge is `61ed42e28fbebbb1166b2a757e82ddc247038b70b19cdbdfc3a70a6621e80fce`.

## Functional gates completed

- All 17 natural classic variants: actual FULL chunks, stored holders matching source queries, save/flush, cold JVM reload and matching full-height column hashes on [Fabric](classic-biome-dev23-fabric-corpus.json) and [NeoForge](classic-biome-dev23-neoforge-corpus.json).
- Positive Bryce/stone pillars and actual caps/materials on both loaders. [Scene images](client-dev23-pillar-images.json) show the real Fabric pillar world; [review](visual-review-dev23.md) states the scope.
- Real preset list/editor/preview, eight slider callbacks, baseline-digest restoration, selected classic source, Create New World, rendered custom terrain/save and a subsequent normal world with no custom context/caches: [Fabric](client-fabric-dev23-base.json), [NeoForge](client-neoforge-dev23-base.json).
- Optimization, community ModernFix, and combined biome/structure stacks: [Fabric matrix](compatibility-dev23-fabric.json), NeoForge [optimization](compatibility-dev23-neoforge-optimized.json), [community ModernFix](compatibility-dev23-neoforge-modernfix.json), [content/structures](compatibility-dev23-neoforge-content.json). Each case exports its own pack and checks generation, consumed file hashes and cold reload. Structures include explicit real placements and naturally located valid starts.
- Active shader/content clients: [Fabric](client-fabric-dev23-content-shader.json), [NeoForge](client-neoforge-dev23-content-shader.json), with all 59 BOP overworld holders retained alongside 17 classics and 55 vanilla holders. Actual DH database output is recorded. Far-LOD visual quality and FPS remain unaccepted.
- Real combined-stack client/dedicated-server joins: [Fabric](loopback-fabric-dev23-content.json), [NeoForge](loopback-neoforge-dev23-content.json). All 17 definitions synchronize; the client receives nonempty terrain, disconnects and both processes save/exit normally.
- WSL Ubuntu Linux Java25 base generation and reload: [Fabric generate](server-linux-fabric-dev23-generation.json)/[reload](server-linux-fabric-dev23-reload.json), [NeoForge generate](server-linux-neoforge-dev23-generation.json)/[reload](server-linux-neoforge-dev23-reload.json). Same production JARs and verified exported ZIP contents; normal watchdog. This is not a Linux renderer or performance claim.
- Exact packaged cache lifecycle: [Fabric](cache-lifecycle-fabric-dev23.json), [NeoForge](cache-lifecycle-neoforge-dev23.json), 3,511 assertions per package, zero retained owners/queued polls. Focused real-world probes retain 24 raw Cell fields and reduce 384 repeated lookup calls to four, including context/climate changes and nested publication.
- [Native structure-column replay](native-columns-dev18-dev23.json) compares 36 genuine original inputs per loader against dev18; full 384-block sampled columns agree. The correction to biome query scope does not silently replace native column results.
- [Hosted source CI](ci-dev23-source.json): both Ubuntu package jobs and Linux/Windows core regression jobs pass for production-source commit `b3da3829eb4e0bc501278cce48f5d890f7afa329`.

## Performance and delivery gate

The predeclared [controlled protocol](performance-protocol.md) passes three fresh trials per generator and loader in base and optimization stacks, matched vanilla controls, two 64-request new regions, resident rereads and cold saved-column verification. All 24 generation trials and eight cold reloads pass with no compiler interference or budget violations. [Results](dev23-performance.md) retain every measured value; [complete acceptance](dev23-acceptance.json) binds 104 evidence files and 30 images. Exact dev23 nested cache probes also pass 801 assertions per loader. Release assembly is in progress.

## Horizon is a separate target

The [exact dev23 native probe](horizon-current-dev23.json) passes the finite prepared bridge against the merged Horizon hook tree. It compares 884,736 block states, 13,824 quart-biome samples and 2,304 heightmap/LOD columns, checks missing/altered/unknown inputs and preserves Nether/End. The [native bundle record](horizon-package-dev23.json) excludes snapshots, game binaries and player resources.

The isolated Rust noise foundation additionally matches its independent Java fixture in debug/release; see [foundation report](horizon-native-noise-foundation-v1.json). Native infinite terrain, density/surfaces/caves/decoration, full native client/save and general Horizon modpack readiness remain unfinished.

## Remaining limits

This is a modern experimental port for new worlds. Exact original Engine geology and all classic visual details are not recovered. Island-preference spawning, every preset/import/control extreme, old-world or cross-loader migration, authenticated public servers, multiplayer load, long exploration, arbitrary pack combinations, far-LOD visual quality and user visual acceptance are outside the accepted scope. Earlier failures and rejected candidates remain in the repository.
