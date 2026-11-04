# Exact final dev11 dedicated content-stack checks

**Passed on both loaders:** exact dev11 JAR custom generation, save and disk reload with Biomes O' Plenty 26.2.0.0.28 beta, GlitchCore 26.2.0.0.0 beta and TerraBlender 26.2.0.0.2. Minecraft 26.2 / Java 25.0.5; Fabric Loader 0.20.1 + Fabric API 0.161.0+26.2, or NeoForge 26.2.0.93.

The four server processes keep the normal 60,000 ms watchdog, save and exit 0. Actual production class origin and installed JAR hashes are verified:

- Fabric: `9c72b55fe91202ba85bbfc1489eebc91b5a1a6b4412373130b73230659d792f0`.
- NeoForge: `7f1ba4f0dd38259b4995fc445afeae637aded12144cb813affc16b5c83158845`.

Each consumes the **qualified same-loader complete-stack export from dev10**, not a newly claimed dev11 dedicated export. [Actual class comparison](dev10-dev11-class-comparison.json) establishes that only `PresetConfigScreen.class` changed; every backend class is identical. The [strict server summary](modpack-biomesoplenty-dev11-summary.json) links the original dev10 export reports and verifies all 2,745 Fabric / 2,749 NeoForge consumed pack file hashes against those inventories. Dependency hashes/declarations remain in the [artifact manifest](modpack-biomesoplenty-26.2-artifacts.json).

In each run, the actual surface serializer entry is the expected MapCodec by type and identity, and the actual TerraBlender sampler/TargetPoint carries finite uniqueness. BOP has 68 registered biomes and 59 overworld possible-biome members. Seed 8675309 with the legacy-default preset requests six FULL chunks and observes 54 columns: 35 overgrown_greens and 19 prairie on each loader. These are actual generated BOP biome observations, not only registration.

Each saved world retains its own exact digest on reload:

- Fabric: `5cc0aef185570698c625da0f9bdf71377b9dbce75f341c145c276a03fd06ada0`.
- NeoForge: `7325be5d489c2f134b2ea12c33293dded651317f25dbe3c71b49e31e38fbb6b6`.

[Fabric generation](server-modpack-biomesoplenty-fabric-dev11-generation.json) / [reload](server-modpack-biomesoplenty-fabric-dev11-reload.json); [NeoForge generation](modpack-biomesoplenty-neoforge-dev11-generation.json) / [reload](modpack-biomesoplenty-neoforge-dev11-reload.json). Separate companions were compiled against the actual dev11 packages: [Fabric build](modpack-biomesoplenty-fabric-dev11-qa-build.json), [NeoForge build](modpack-biomesoplenty-neoforge-dev11-qa-build.json). No QA classes enter production JARs.

No port/serialization ERROR lines remain. The repeated known OSHI Windows Perflib counter lookup diagnostic is classified separately; no system settings were changed.

Reproduce with the same server helpers and repeated `--extra-mod` stack arguments described in the [dev10 matrix](modpack-biomesoplenty-dev10.md), selecting the exact dev11 production/QA JARs and the linked same-loader export10 directory for `--qa-mode generate --datapack`. Reload the returned instance with `--qa-mode reload --reuse-instance`, omitting `--datapack`. These tests do not claim all biomes reachable, arbitrary modpacks compatible, full fresh-world/cross-loader block identity, benchmark acceptance, multiplayer login or client visual acceptance. Client reports are separate.
