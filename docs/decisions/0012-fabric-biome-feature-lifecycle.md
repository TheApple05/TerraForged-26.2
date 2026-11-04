# 0012: Preserve Fabric's feature rebuild lifecycle

## Trigger and cause

The packaged dev4 client completed repeated live previews and the actual preset Done/export/reload flow, then crashed while opening a copied custom world: `ChunkGenerator.applyBiomeDecoration`, feature index `-1`, list length `0`. The original report/log is retained under `build/qa/client/instances/20261104T080349Z-dc7f47/` and the client validation reports; passing dedicated-server generation did not cover this client load order.

Minecraft 26.2 `WorldOpenFlows.openWorldLoadLevelStem` loads the world stem and calls each `LevelStem.generator().validate()` before starting the integrated server. `WorldCreationContext.validate()` also validates generators. `ChunkGenerator.validate()` eagerly resolves its memoized `featuresPerStep` sorter. Fabric applies biome modifications later, at `MinecraftServer` constructor return.

Our Add/Replace modifiers intentionally edit ordered feature lists directly, preserving `Order` semantics. They reset the biome-local flower/feature suppliers, but bypassed Fabric's `rebuildFeatures` flag. Fabric consequently retained the pre-modification generator sorter. Its identity mapping returned `-1` for newly added features, causing the subsequent list access to fail. A dedicated server could avoid the failure because its first sorter read occurred after modifications.

There was also a process-lifetime ownership bug: Fabric's `getSortedModifiers()` returns its live global list. Appending datapack modifiers to it retains closures and holders from that registry view, allowing accumulation across world loads.

## Change

- A narrow Fabric-only accessor targets `BiomeModificationContextImpl$GenerationSettingsContextImpl.rebuildFeatures:Z`.
- After the existing ordered Add/Replace mutation, `markFeaturesChanged(modificationContext)` sets that native flag. Fabric's own `freeze()` rebuilds bone-meal/feature caches, and its finalizer replaces every affected level-stem generator's memoized sorter. No unknown features are skipped and the original Add/Replace ordering and data formats remain.
- The modifier-list redirect copies `getSortedModifiers()` into a fresh `ArrayList` before adding registry-owned datapack modifiers and sorting. Fabric's process-wide base list stays unchanged.

This uses an internal Fabric hook because its public generation context supports append/remove operations rather than the inherited prepend/replace ordering contract. The accessor is restricted to this exact boolean; a changed target fails through the required mixin configuration instead of silently losing features.

## Exact API evidence

Inspected the resolved **fabric-biome-api-v1 18.0.6+c7bd5b8e9e** [published sources](https://maven.fabricmc.net/net/fabricmc/fabric-api/fabric-biome-api-v1/18.0.6+c7bd5b8e9e/fabric-biome-api-v1-18.0.6+c7bd5b8e9e-sources.jar), SHA-256 `110ee8c40c97705ad62de83a11295208895e046743758a9ecb27d34e26d1f9a2`:

- `BiomeModificationContextImpl.shouldRebuildFeatures()` returns `generationSettings.rebuildFeatures`.
- `GenerationSettingsContextImpl` initializes that flag false; native `addFeature`/`removeFeature` set it true after mutation.
- `BiomeModificationImpl.finalizeWorldGen` checks it after `freeze()` and resets `LevelStem.generator().featuresPerStep` through the real `FeatureSorter.buildFeaturesPerStep` operation.
- `BiomeModificationImpl.getSortedModifiers()` returns its stored `modifiers` list directly.

## Verification status

Source diagnosis and strict mixin JSON/whitespace checks completed. The five changed Java files compile against the actual26.2/Fabric dependencies with exit0 ([isolated compiler record](../validation/fabric-biome-lifecycle-compile.json)); this does not verify mixin transformation or runtime behavior. The coordinated dev6 package and the same real client load flow are the required runtime gate; do not infer success from this source change alone. Post-start QA should also verify that every current biome feature maps to the same identity in its generator sorter, including an eagerly validated generator.
