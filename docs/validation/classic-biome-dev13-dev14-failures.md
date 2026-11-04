# Classic restoration: retained dev13/dev14 failures

Date: 2026-11-04, America/Los_Angeles. These are rejected candidates, not passing restoration evidence.

Both dev13 exports failed while serializing newly added vegetation. Minecraft 26.2 requires homogeneous holder kinds within a biome's feature-step list; mixing registered inherited references with new direct placed-feature holders was invalid. The failed records are [Fabric export13](server-fabric-dev13-export.json) and [NeoForge export13](server-neoforge-dev13-export.json). Dev14 replaces those additions with five separately registered vegetation aliases and successfully exports complete base presets: [Fabric export14](server-fabric-dev14-export.json), [NeoForge export14](server-neoforge-dev14-export.json).

Those successful exports did not establish working generation. The dev14 Fabric server encountered a native `FeatureSorter` cycle involving `minecraft:swamp` and `reterraforged:marshland`. The reused swamp surface holder acquired opposite ordering relative to the globally applied erosion feature in the two sources. A separate optimized-stack run reproduced the same cycle; that observation does not identify C2ME as the cause. The correction assigns marshland a distinct registered `classic_marsh_surface` placed alias and avoids copying this mod's previously applied erosion/snow/swamp modifier footprints back into a new variant definition. The failed Fabric runtime record remains [survey14](server-fabric-dev14-classic-survey.json).

Dev15 still reproduced the native swamp/marshland cycle on both loaders. Inspection of the actual 26.2 classes identified why changing the registry ID alone was insufficient: `PlacedFeature` is a record, and `FeatureSorter` uses `Object2IntOpenHashMap` with structural equality. A reference alias with identical configured-feature holder and placement list collapses back onto its original graph vertex. This applies to the vegetation aliases too. The next correction appends fixed `CountPlacement.of(1)` to every alias: one input position still yields one output position, and the constant provider consumes no random values, while the placement sequence becomes structurally distinct. Runtime acceptance now explicitly rejects structurally equal alias/original values. The retained failed runs are [Fabric survey15](server-fabric-dev15-classic-survey.json) and [NeoForge survey15](server-neoforge-dev15-classic-survey.json).

The dev14 NeoForge dedicated server generated its six baseline FULL chunks and saved them, but the new classic acceptance check rejected the run because the enabled setting and actual source disagreed. The exported `data/minecraft/worldgen/world_preset/normal.json` contained the intended `reterraforged:classic_variants` wrapper; the same NeoForge export also contained `data/minecraft/dimension/overworld.json` with the old, unwrapped multi-noise source. Vanilla's dimension bake gives the supplied dimension entry priority over the selected world preset. Fabric's provider did not export that dimension entry. This loader-specific export difference requires patching the overworld dimension as well as the world preset, preserving its original generator and delegate. See [NeoForge survey14](server-neoforge-dev14-classic-survey.json).

Client validation also needs to inspect the currently selected dimensions after asynchronous datapack application: vanilla can rebind the existing selected source while leaving the newly patched world preset unapplied. A successful editor/world-creation flow with the old possible-biome count is insufficient evidence of restored classic selection. The strengthened gate requires setting/source agreement and actual classic possible-biome membership before creation.

The [95-assertion policy check](classic-biome-policy-dev14.json) passed against the exact Fabric dev14 artifact. It covers synthetic selector branches, boundaries, family preservation and settings codecs. It deliberately makes no claim about real biome reachability, source application, FULL chunk generation or appearance, and does not override these runtime failures.
# Dev19 positive-pillar acceptance found a real grass-footing defect

The exact dev19 Fabric and NeoForge packages both passed registry/source/codec and the actual all-17 source survey. Strong natural ridge targets were then located using the real source/sampler and production ridge objects. A forecast only selected test locations; it was never counted as placed-block evidence.

Both strict positive-pillar runs failed on stone forest. A retained diagnostic run observed **79 actual added Bryce columns**, maximum 25 blocks, using terracotta, orange, brown and yellow bands. Stone forest had 89 positive filtered-tile shape/mask columns, maximum predicted height 48, but zero actual added stone columns.

Read-only QA mixin telemetry immediately before the real `ClassicPillarFeature.place` invocation established the cause: all 89 stone candidates had matching stored biome, empty space above the current heightmap, and `minecraft:grass_block` footing. The production guard checked `BlockTags.DIRT`, which does not include grass block in this target registry, and rejected every footing. The real feature returned false. Final foliage above those columns initially suggested a canopy interaction; invocation-time evidence disproved that explanation for this fixture. No biome was forced and no test blocks were placed.

The failed candidate evidence remains immutable:

- `server-fabric-dev19-pillar-positive.json` and `server-neoforge-dev19-pillar-positive.json`: strict failure on both loaders.
- `classic-biome-dev19-pillar-diagnostics.json`: SURFACE-to-FULL block/material and actual filtered-tile observations.
- `classic-biome-dev19-pillar-placement-trace.json`: real feature HEAD/RETURN observations.
- `classic-biome-dev19-pillar-locations.json`: selected natural coordinates and explicitly labeled pre-filter forecasts.

The correction must be verified on fresh chunks from the next exact packaged candidate, with the positive-pillar assertion still enabled. Existing failed worlds cannot supply a fresh SURFACE baseline.

