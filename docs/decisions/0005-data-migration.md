# 0005: Preserve preset and vegetation semantics across the data API migration

Date: 2026-11-03. Status: implemented; full-mod runtime and generated-pack validation remain required.

## Scope and evidence

This pass migrates data generation under `data/worldgen`, excluding the noise-router/surface-rule implementations and the authoritative settings model maintained separately. It preserves custom trees, grass probabilities, erosion/snow processing, swamps, preset controls, tag memberships and configured vertical bounds. No sources, features or template lists were excluded to make compilation easier.

API signatures were inspected in the actual Minecraft 26.2 JAR and Loom-generated sources. Reference behavior for removed random patches was inspected and then **executed** using the actual named Minecraft 1.20.1 game JAR. No Minecraft source or game JAR is redistributed. See the [official 26.2 metadata](https://piston-meta.mojang.com/v1/packages/09165f570b16394790c4e37c93b0918729615d17/26.2.json) and [NeoForged's vanilla migration primer](https://github.com/NeoForged/.github/blob/main/primers/26.2/index.md).

## Removed random patches: preserve the exact random stream

Minecraft removed the old `RandomPatchFeature`/configuration. The 1.20.1 helper supplied horizontal spread 7 and vertical spread 3 with the requested attempt count. Every attempt draws two bounded integers per axis and subtracts them in X, Y, Z order, then executes the nested feature using the same random source. All attempts run, including after a successful placement.

Modern count/triangle-offset combinators can reproduce the probability distribution but change the mapping of random draws to offsets. That would alter seeded vegetation positions and downstream random state. `LegacyRandomPatchFeature` therefore implements the observed semantics directly, preserving nested-feature interleaving, success aggregation and sub-feature discovery. Both configured-feature paths use it; existing selection weights and attempt counts are unchanged.

The [cross-version parity report](../validation/legacy-patch-parity-2026-11-03.json) compares **actual old and new feature executions**, not two copies of the same reference implementation. Across 216 cases and 6,948 nested placements, coordinates, nested random draws, success flags and final random states yield the same digest:

```text
1c70db1cb5f55a9ea98abf103f089a5912f92bc8cf69d7b46e68a3c7afc34e9a
```

The fixture covers six seeds including signed-long extremes, positive/negative origins, attempt counts 0/1/24/48/56/64, and spreads 7/3, 0/0 and 1/9. It uses a controlled recording child feature and a minimal world proxy. This proves patch scheduling/random-stream parity under those conditions; it does not prove in-world vegetation or survival behavior.

Reproduce after resolving the target classpath:

```powershell
python tools/qa/legacy_patch_probe.py --java-home PATH_TO_JDK_25
```

The reference named 1.20.1 Loom cache must exist. The runner resolves its matching library versions and verifies each against the official version descriptor's SHA-1, instead of accidentally running old game classes with new Authlib/DFU dependencies.

## Configured features and tags

The removed disk state-provider wrapper is replaced by 26.2 `RuleBasedStateProvider`, retaining sand-over-air sandstone rules and all disk targets/radii/heights. Short grass uses its current block name. Bush trees now pass the explicit dirt provider that the old constructor supplied implicitly. The obsolete duplicate erosion entrypoint uses the same explicit erosion/snow configuration already present in the authoritative preset path, rather than nonexistent constructors.

The forest replacement targets the actual current `trees_birch_and_oak_leaf_litter` placed-feature key used by vanilla forest generation. TerraForged's custom tree replacement remains active; the custom templates themselves are unchanged. Vanilla's newly added leaf-litter tree decorator is not introduced into the old custom templates by this API rename and needs visual/integration review.

Block tags use the current `TagsProvider`/resource-key appender API. Soil, clay, sediment, erodible and rock memberships, including the optional dirt tag, are preserved. Biome-scoped legacy modifiers explicitly select the whitelist behavior already used by the authoritative modifier path.

## Dimensions

`OverworldDimensionData` expresses the current vanilla overworld's environment attributes, clock, timelines, lighting, infiniburn and monster light rules, while preserving the preset's `minY = -worldDepth` and total/logical height `worldDepth + worldHeight`. Both preset entrypoints use the same helper. This migrates the removed effects/monster-setting constructor fields into their actual 26.2 systems.

The [dimension probe](../validation/dimension-data-2026-11-03.json) verifies complete serialized equality with the actual vanilla 26.2 overworld at the default bounds, then checks bounds and encoded bounds for nine depth/height combinations. Reproduce with `python tools/qa/dimension_data_probe.py --java-home PATH_TO_JDK_25`.

Use `provider.createSerializationContext(JsonOps.INSTANCE)` when encoding builder-generated registries. Constructing a fresh `RegistryOps` adapter loses the builder's special holder owner and rejects valid references. A decode round-trip is not claimed by this standalone test: the vanilla bootstrap lookup does not load actual block/timeline tags. That check belongs in the generated-pack/runtime test.

## Registry patches, export and duplicate preset models

Modern `RegistrySetBuilder.buildPatch` requires a cloner factory and returns separate full and patch-only views. `RegistryPatches` registers codecs for vanilla dynamic/dimension registries and all four custom registries, then returns the full view so preview generation can resolve unchanged vanilla entries through correctly rebound references. Pack export currently consumes that full provider too; verify generated content and consider selecting the patch-only view separately to avoid unnecessarily exporting unchanged entries. No filtering has been introduced silently.

Pack generation uses `DataGenerator.Cached` with the original cache root and provider factories targeting the requested `PackOutput`, preserving the distinction between temporary/cache and exported-pack directories despite the now-private pack constructor.

The source tree contains a superseded duplicate `preset.Preset` alongside authoritative `preset.settings.Preset`. The old model now delegates patch generation through a codec conversion to the authoritative one, with a reverse round-trip check that throws if any serialized legacy setting is lost. The modern model's extra controls receive its declared codec defaults. This compatibility bridge still needs an end-to-end legacy-preset/runtime test; compilation alone is not evidence that every historical preset is accepted.

## Remaining gates

- A coherent full-source compiler pass after peer migrations settle.
- Generate real preset and mud-swamp packs; check registry references, codecs, tag loading and metadata, including default/mutated presets.
- Decode/reload those packs in the packaged mod on both client and dedicated server.
- Compare terrain, tree/survival placement, vegetation density, surface disks, snow and erosion visually at recorded seeds/coordinates.
- Verify the full registry view does not create unwanted vanilla overrides in exported packs; preserve a separate patch-only export if needed.

These changes and isolated probes do not establish a playable port, packaged-runtime success or complete feature parity.
