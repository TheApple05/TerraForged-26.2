# 0016: Restore registered classic variants without replacing the modern biome resolver

Date: 2026-11-04, America/Los_Angeles. Implementation decision; packaged acceptance belongs to the separately recorded exact-artifact runtime reports.

## Reference and identifiers

The authenticated source is original TerraForged `0.2.x` commit `3ada751ac25cd3eecee5d198870a3abb89734f52`, especially `BiomeBuilders`, `ModBiomes`, and `TFDefaultBiomes`. Its 17 active `terraforged:*` variants are restored at the already-declared ReTerraForged `reterraforged:*` keys. This mapping does not migrate an existing 1.16 world or alias old block palettes.

`ClassicBiomes.VARIANTS` is the complete builder inventory. Each variant starts with the installed modern parent's actual biome data and retains its mobs, carvers, environment attributes, ambience and generation settings except the explicit builder changes below. Removed 1.16 parent names use these deliberate modern equivalents:

- `bryce`: badlands; remove tree features, add desert vegetation; append the restored masked Bryce pillar feature.
- `cold_steppe`: old growth spruce taiga, replacing giant spruce taiga; remove trees, forest rocks and taiga/old-growth mushroom patches; temperature 0.2, downfall 0.025; add dead bushes, dense grass and ferns.
- `cold_marshland`: snowy taiga; remove trees; temperature 0.2, downfall 0.5; add dead bushes, dense grass and ferns.
- `fir_forest`: taiga; retain parent trees; add dead bushes, dense grass and ferns.
- `flower_plains`: flower forest; retain parent trees and flowers, as the original builder did.
- `frozen_lake`: frozen river; remove trees.
- `frozen_marsh`: snowy taiga; remove trees; temperature 0.14999, downfall 0.15.
- `lake`: river; remove trees.
- `marshland`: swamp; remove trees; temperature 0.7; add dead bushes, dense grass and ferns. Modern swamp extra vegetation is retained from the parent; the active classic marshland swamp surface feature is appended once through a separate registered `classic_marsh_surface` placed alias. A shared placed surface holder produced a real dev14 feature-order cycle against vanilla swamp/global erosion ordering; that failure is retained. Variant-only aliases preserve each source's valid local ordering. The original setup did not wire this feature for cold marshland or frozen marsh.
- `savanna_scrub`: savanna; remove trees; add dead bushes and dense grass.
- `shattered_savanna_scrub`: windswept savanna, replacing shattered savanna; retain parent trees; add dead bushes and dense grass.
- `snowy_fir_forest`: snowy taiga; retain parent trees; add ferns.
- `snowy_taiga_scrub`: snowy taiga; remove trees; add ferns.
- `steppe`: copy old growth spruce taiga, remove trees, forest rocks and taiga/old-growth mushrooms; use savanna ambience; no precipitation; temperature 1.2, downfall 0.05; add dead bushes and dense grass. Its modern classification/structure tags come from windswept savanna, intentionally translating the original changed remap parent and category rather than copying taiga tags accidentally.
- `stone_forest`: jungle; retain parent vegetation; append the restored masked stone pillar feature.
- `taiga_scrub`: taiga; remove trees and taiga/old-growth mushrooms; add dead bushes, dense grass and ferns.
- `warm_beach`: warm ocean; remove trees; temperature 1.0; classify/tag as beach. Warm-ocean mobs, water ambience and inherited underwater features remain, matching the original's unusual parent choice.

Minecraft 26.2 does not retain the original independent `SNOW` precipitation enum in biome data. The original numeric temperatures and precipitation presence are retained, and current Minecraft temperature/height rules determine rain versus snow. This is a documented climate adaptation, not a claim of identical 1.16 snowfall at every elevation.

Natural pillar footing must include the target's separate `#minecraft:grass_blocks` family alongside stone, dirt, sand and terracotta. In 26.2 the dirt tag contains dirt, coarse dirt and rooted dirt; grass block, podzol and mycelium belong to the separate grass-block family. Exact dev19 placement telemetry found that checking only dirt rejected every positive-mask stone-forest grass footing. The next candidate corrects that eligibility without overwriting woody structures or introducing a foliage-replacement policy. Failed traces and fresh-chunk acceptance are retained separately.

Added vegetation uses five distinct registered placed-feature aliases with modern placements for dead bushes, plains/normal/savanna grass and large ferns. The inherited global vanilla feature order stays unchanged; adding the same shared vanilla holder in a contradictory position can otherwise create feature-sort cycles. A first dev13 export failed because 26.2 rejects mixed direct/reference holders within one feature-step list; that failure is retained, and the registered aliases correct the representation. Dev15 exposed a further issue: `PlacedFeature` is a record and the native sorter keys by structural equality, so a new registry ID alone does not distinguish otherwise equal aliases. Every alias now appends fixed `CountPlacement.of(1)`, preserving positions and random-number consumption while making its placement sequence distinct. Actual runtime checks reject structurally equal aliases. All nested registry references, including supported pack-supplied inline features, are rebound through the original codec and the new bootstrap context. Registration is independent of selection, so every exported TerraForged preset has all 17 definitions.

## Selection is separate from decoration

`miscellaneous.classicBiomeVariants` controls selection independently of `customBiomeFeatures`. Newly constructed built-in presets enable it. Missing fields in older serialized presets decode to **false**, preserving their modern biome source rather than silently enabling new biome selection in an existing saved preset. The Miscellaneous page exposes the toggle. Both duplicate preset models preserve the field through copy/codec conversion.

When enabled, compatible existing world presets whose overworld uses the normal noise settings retain their full serialized dimensions and source inside `reterraforged:classic_variants`. Nether, End, other dimensions and unknown generator types are left under their existing definitions. The complete delegate resolver runs first, including TerraBlender and biome mods. Only an explicit set of vanilla surface parents is eligible for a classic substitution. Third-party holders, cave biomes, mushroom fields, cherry grove, mangrove swamp, pale garden and modern mountain families are preserved. Unknown future vanilla families also stay with the delegate.

The wrapper serializes its complete variant holder set. `possibleBiomes`, stored chunk biomes, biome search and actual `getNoiseBiome` use the same wrapper; variants cannot be returned while missing from its advertised possible set. Global and chunk-cached climate samplers receive the owning `GeneratorContext`. There is no static global world registry or cross-world holder cache.

The retained cell's actual lake, wetland and beach classifications select hydrological variants. Other variants use a deterministic bit-mixed hash of the raw `biomeRegionId` within compatible modern parent families, retaining ordinary parents alongside them. The raw ID already interpolates modern climate. Dev16's actual 65,536-point survey reached 15 variants but missed cold steppe and flower plains despite observing 670 old-growth spruce taiga and 299 flower forest parents. Their raw selector bands correlated with those parent climate bands. The separate 24-bit `[0,1)` hash domain decorrelates the choice without altering raw region boundaries, climate fields, parent holders or terrain. Fixed integer arithmetic and raw float bits make it independent of traversal order and unrelated cell fields; nonfinite raw IDs/temperatures retain the delegate. The policy is an intentional **26.2 adaptation**, not recovered Engine selection code:

- Lake cells use frozen lake below normalized region temperature 0.25, otherwise lake.
- Wetland cells use frozen marsh below 0.2, cold marshland below 0.4, otherwise marshland.
- Beach cells above normalized region temperature 0.75 use warm beach.
- Badlands/eroded badlands, old growth spruce taiga, flower forest, swamp, savanna, savanna plateau and windswept savanna select their corresponding variant in the first 35% of the region selector domain.
- Taiga and snowy taiga select forest variants below 0.25, scrub variants from 0.25 to 0.55, and retain the modern parent above that boundary.
- Jungle selects stone forest below 1/6, retaining ordinary jungle in the remaining selector domain. This preserves rarity without claiming that the original Forge manager weight alone described the entire old biome distribution.

These fractions describe selector domains, not promised measured world-area percentages. Normalized region temperatures are explicitly distinct from biome metadata temperatures and the unavailable Engine's constants. Below zero the wrapper returns the original delegate biome; the modern cave system remains authoritative.

## Surfaces, tags and interoperability

Vanilla surface rules use exact biome holder sets. Registering a custom biome or adding a tag alone does not make a badlands, beach or swamp condition match it. `PresetSurfaceRuleData` therefore serializes the actual 26.2 vanilla surface rule, expands its `biome_is` lists with each variant's mapped surface parent, and decodes every holder through the new registry context. Other rule fields and branches are preserved. Steppe uses savanna surface eligibility; warm beach retains the warm-ocean sand surface family. This avoids replacing a whole surface graph with guessed grass/stone rules.

The additive `ClassicBiomeTagsProvider` inherits actual installed parent membership for modern structure, feature and other biome tags. Warm beach inherits classification from beach; steppe inherits classification from windswept savanna. All variants also belong to `#reterraforged:classic_variants`. This is a deliberate translation of old category/manager behavior; the original Forge dictionary was not a modern tag JSON inventory.

The wrapper is a new serialized biome-source type. Integrations that correctly use `BiomeSource` retain the delegate and its full possible set. Mods that hard-code an `instanceof MultiNoiseBiomeSource` expectation around the outer source need explicit compatibility testing. Passing one content stack does not certify all such integrations.

## Acceptance evidence and limits

`ClassicBiomeQa` checks actual loaded definitions, tree removals, setting/source agreement, possible membership and source codec roundtrip. Its optional regional survey samples the actual world heightmap and the real source/sampler, recording coordinates and distributions for each reached variant; synthetic policy tests are separate. Biome mods may replace vanilla parents intentionally, so their survey must not force classic variants over those mods' holders just to make a coverage count pass.

Required packaged checks include both loaders, new world generation and reload, actual classic variant reachability, representative FULL chunks and pillar materials/masks, content-stack ownership, structure eligibility and client rendering. Controlled performance tests must record their workload and resources separately from broad regional search. User visual acceptance and literal block-for-block original-world identity are separate questions.
