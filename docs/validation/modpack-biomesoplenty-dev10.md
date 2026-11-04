# Complete Biomes O' Plenty stack on both loaders

**Passed:** exact dev10 packages export a preset with the full content stack, load that exact newly exported pack, generate custom terrain, flush/save and reload on both Fabric and NeoForge. All six isolated production server processes exit0 with the normal60,000ms watchdog; actual production class origin and JAR SHA are checked. [Strict combined evidence](modpack-biomesoplenty-dev10-summary.json) includes copied-pack file-by-file SHA equality, runtime assertions, registered IDs, loaded classes and saved-world digests.

## Pinned stack

- Minecraft26.2, Java25.0.5; Fabric Loader0.20.1 + Fabric API0.161.0+26.2, or NeoForge26.2.0.93.
- Biomes O' Plenty26.2.0.0.28 **beta**, official [Fabric](https://modrinth.com/mod/biomes-o-plenty/version/MNZPk6V0) / [NeoForge](https://modrinth.com/mod/biomes-o-plenty/version/Kpuzuokl) files explicitly supporting26.2.
- GlitchCore26.2.0.0.0 **beta**, official [Fabric](https://modrinth.com/mod/glitchcore/version/SDUCBYRU) / [NeoForge](https://modrinth.com/mod/glitchcore/version/POAebwFo) files explicitly supporting26.2.
- TerraBlender26.2.0.0.2, the exact resolved same-loader Maven artifacts.

[Artifact manifest](modpack-biomesoplenty-26.2-artifacts.json) records download sources, published SHA1/SHA512 checks for the content/dependency downloads, SHA256 values and decisive in-JAR dependencies. BOP's JAR requires TerraBlender>=26.2.0.0.1 even though its marketplace dependency list omits that requirement. Downloaded third-party JARs and generated packs remain ignored QA files; no third-party code/assets were copied into the mod.

Production SHA256 values:

- Fabric: `6c698372e83d450464d56f896599f1b4cf7e838d5921a831e8a9b28256d663ae`.
- NeoForge: `87c45cfdc49e4e3b9427314aa0c58ad97e149cf852097741492768d9f40bfb9a`.

## What actually passed

Both exports retain **all68 registered BOP biome entries**;59 BOP IDs belong to the actual overworld possible-biome set. Fabric writes2745files; NeoForge writes2749. Each fresh generation instance receives a file-for-file identical copy of its own complete-stack export, verified against the export's recorded SHA inventory.

The seed8675309 legacy-default test requests6FULL chunks spanning negative/origin/tile boundaries and observes54columns. Every actual sampled biome on each loader is BOP content: `overgrown_greens` / `prairie` (Fabric35/19, Neo36/18). Thus this is actual content generation, beyond registry membership. Both worlds have a live TerraForged context/cache; the actual TerraBlender sampler and returned TargetPoint carry finite uniqueness. The actual `material_rule/terrablender:merged` registry value has the required MapCodec type and exact CODEC identity.

Each world reloads with its own unchanged digest:

- Fabric: `aae34f91f28ccfb39e038c809e9040da6efecde2699c1b7b4bd58d2ec572e815`.
- NeoForge: `7356e2badb0d1cc9b2e2a460f15759b9af3571c47dccee526a78be0afb80453c`.

Across loaders, all54 raw cell records/terrain types/cell-biome classifications match, while five surface blocks, two heights and one biome-at-observed-surface differ. [Exact cross-loader observations](modpack-biomesoplenty-dev10-cross-loader-observations.json) remain visible; full cross-loader block/biome parity is not claimed. The only ERROR-level log line in each process is the existing OSHI Windows Perflib English-counter lookup diagnostic; no port/serialization error lines remain and no system registry settings were changed.

The failed dev8 serialization attempts and dev9 injection attempts are retained. [Decision0014](../decisions/0014-terrablender-material-rule-codec.md) explains the upstream Codec/MapCodec defect, the final narrowly gated correction, and why an earlier injection selector failed. Passing dev10 does not erase those failures.

## Reproduction and scope

Build the separate Fabric/Neo runtime companion against the exact production JAR with `build_runtime_qa.py` or `build_neoforge_runtime_qa.py`. Supply the companion plus all three same-loader content/dependency JARs via repeated `--extra-mod` to `production_server_smoke.py` or `production_neoforge_server.py`.

1. Run `--qa-mode export` into a fresh instance. Confirm a passed callback, actual MapCodec identity, BOP inventories and complete file hashes.
2. Run a fresh `--qa-mode generate --datapack <that-export-instance>/qa-output/presetpack`, with the same four extra JARs.
3. Run `--qa-mode reload --reuse-instance <generation-instance>`, omitting `--datapack`. Retain the same exact stack and compare the persisted digest.

Exact instance/artifact/export paths and QA-companion hashes are in the [combined report](modpack-biomesoplenty-dev10-summary.json) and its six linked raw reports. Fabric report paths must start `docs/validation/server-`; both helpers keep the normal watchdog. Companion source/build provenance is retained in [Fabric](modpack-biomesoplenty-fabric-dev10-qa-build.json) / [NeoForge](modpack-biomesoplenty-neoforge-dev10-qa-build.json) records. No QA classes enter the production JARs.

This matrix covers one pinned stack, seed and preset, and two actually observed BOP biomes. It does not establish every biome's reachability, arbitrary modpack compatibility, long exploration, multiplayer login or performance acceptance. Client evidence is separate from this server matrix.
