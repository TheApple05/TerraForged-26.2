# Classic biome and surface restoration research

Reference: original TerraForged `0.2.x` commit `3ada751ac25cd3eecee5d198870a3abb89734f52` (declares mod 0.2.16 / Minecraft 1.16.5). Selected modernization baseline: ReTerraForged `29239f1f164d19dbfcccaca4a8277d64e784207c` (0.0.6 / 1.20.1). Audit date: 2026-11-03, America/Los_Angeles.

This is source research. It does not claim that a classic reference installation or all target biome paths have run.

## Exact original registry inventory

[`ModBiomes.java`](https://github.com/TerraForged/TerraForged/blob/3ada751ac25cd3eecee5d198870a3abb89734f52/src/main/java/com/terraforged/mod/biome/ModBiomes.java) actively registers all 17 keys below. Their original namespace is **`terraforged`**. [`BiomeBuilders.java`](https://github.com/TerraForged/TerraForged/blob/3ada751ac25cd3eecee5d198870a3abb89734f52/src/main/java/com/terraforged/mod/biome/BiomeBuilders.java) defines the following parent and explicit changes; unlisted properties are inherited rather than guessed:

- **`terraforged:bryce`** — parent `minecraft:badlands`; Forge biome-manager type `DESERT`; remove tree configurations, add desert vegetation.
- **`terraforged:cold_steppe`** — parent `minecraft:giant_spruce_taiga`; manager type `COOL`; snow precipitation, temperature `0.2`, downfall `0.025`; remove trees, forest rocks, and taiga red/brown mushroom patches; add dead bushes, dense grass, and ferns.
- **`terraforged:cold_marshland`** — parent `minecraft:snowy_taiga`; manager type `ICY`; category `SWAMP`, snow precipitation, temperature `0.2`, downfall `0.5`; remove trees; add dead bushes, dense grass, and ferns.
- **`terraforged:fir_forest`** — parent `minecraft:taiga`; manager type `COOL`; add dead bushes, dense grass, and ferns. This method does not itself remove all parent trees.
- **`terraforged:flower_plains`** — parent `minecraft:flower_forest`; manager type `COOL`; no further explicit builder changes. Its name alone is not evidence that its parent trees were removed.
- **`terraforged:frozen_lake`** — parent `minecraft:frozen_river`; manager type `ICY`; remove trees.
- **`terraforged:frozen_marsh`** — parent `minecraft:snowy_taiga`; manager type `ICY`; category `SWAMP`, snow precipitation, temperature `0.14999`, downfall `0.15`; remove trees.
- **`terraforged:lake`** — parent `minecraft:river`; manager type `COOL`; remove trees.
- **`terraforged:marshland`** — parent `minecraft:swamp`; manager type `COOL`; category `SWAMP`, temperature `0.7`; remove trees; add dead bushes, ferns, dense grass, and default extra vegetation.
- **`terraforged:savanna_scrub`** — parent `minecraft:savanna`; manager type `WARM`; remove trees; add dead bushes and dense grass.
- **`terraforged:shattered_savanna_scrub`** — parent `minecraft:shattered_savanna`; manager type `WARM`; add dead bushes and dense grass. This builder does not explicitly remove parent trees.
- **`terraforged:snowy_fir_forest`** — parent `minecraft:snowy_taiga`; manager type `ICY`; category `ICY`; add ferns.
- **`terraforged:snowy_taiga_scrub`** — parent `minecraft:snowy_taiga`; manager type `ICY`; remove trees; add ferns.
- **`terraforged:steppe`** — initially copies `minecraft:giant_spruce_taiga`; removes trees, forest rocks, and taiga red/brown mushroom patches; sets its remap parent key to `minecraft:shattered_savanna_plateau`, category to `SAVANNA`, and ambience from `minecraft:savanna`; no precipitation, temperature `1.2`, downfall `0.05`; adds dead bushes and dense grass. No biome-manager type is explicitly set in this builder.
- **`terraforged:stone_forest`** — parent `minecraft:jungle`; manager type `WARM`; explicit weight `2`.
- **`terraforged:taiga_scrub`** — parent `minecraft:taiga`; manager type `COOL`; remove trees and taiga red/brown mushroom patches; add dead bushes, dense grass, and ferns.
- **`terraforged:warm_beach`** — parent `minecraft:warm_ocean`; manager type `WARM`; remove trees; category `BEACH`, temperature `1.0`.

### Original tags and weights

The old code uses Forge `BiomeDictionary` types and `BiomeManager` climate categories, not modern biome-tag JSON files. [`BiomeBuilder.java`](https://github.com/TerraForged/TerraForged/blob/3ada751ac25cd3eecee5d198870a3abb89734f52/src/main/java/com/terraforged/mod/biome/utils/BiomeBuilder.java) copies dictionary types from the parent and explicitly adds **`OVERWORLD`** when registering each variant. It does not declare a new hard-coded per-variant modern tag set. Listing guessed `#minecraft:is_*` tags as original tags would be inaccurate.

If a manager type is set, an explicit positive weight wins; otherwise `RARE` dictionary types receive weight `2`, `FOREST` types receive `5`, and others receive `10`. The exact inherited tag set depends on the original parent and installed Forge/mod registrations. Steppe's builder changes its remap parent after copying the original parent's dictionary types; that subtlety should be reproduced intentionally or documented as a correction.

## Original selection conditions

These are distinct layers, and their numeric domains must not be conflated.

1. [`BiomePredicate.java`](https://github.com/TerraForged/TerraForged/blob/3ada751ac25cd3eecee5d198870a3abb89734f52/src/main/java/com/terraforged/mod/biome/provider/analyser/BiomePredicate.java) classifies registered biome **metadata**, including names/categories and base temperature/downfall. Its constants are frozen `-0.4`, cold `0.2`, warm `1.4`, hot `1.7`; light rain `0.2`, moderate `0.4`, heavy `0.8`. Range comparisons include endpoints.
2. Named cold steppes have temperature at most `0.2`; other steppe candidates have temperature at least `0.2`, with the analyser explicitly excluding `COLD_STEPPE` from `STEPPE`. Desert includes category `DESERT`, or temperature at least `1.7` with downfall at most `0.2`. Savanna includes category `SAVANNA`, or temperature at least `1.4` with downfall at most `0.4`, then excludes desert, steppe, beach, mountain, and wetland candidates. Lakes are identified by the name `lake`; ordinary rivers exclude lakes. Wetlands use category `SWAMP`. Forest rainfall and precipitation categories further separate temperate rainforest, forest, taiga, and tundra groups.
3. [`BiomeAnalyser.java`](https://github.com/TerraForged/TerraForged/blob/3ada751ac25cd3eecee5d198870a3abb89734f52/src/main/java/com/terraforged/mod/biome/provider/analyser/BiomeAnalyser.java) builds weighted terrain/climate pools. It adds plains with weight `5` to temperate rainforest and temperate forest, flower forest with weight `2` to temperate forest, snowy taiga scrub with weight `2` to tundra, and taiga scrub with weight `2` to taiga. It excludes Nether/End/void, mountain edge, names containing `hills`, and biomes not classified as overworld candidates.
4. [`TFDefaultBiomes.java`](https://github.com/TerraForged/TerraForged/blob/3ada751ac25cd3eecee5d198870a3abb89734f52/src/main/java/com/terraforged/mod/biome/context/TFDefaultBiomes.java) supplies fallback terrain selectors. Beaches use snowy beach / beach / warm beach with selector thresholds `0.25` and `0.75`. Lakes use frozen lake / lake / lake with `0.15` and `1`. Rivers use frozen river / river / river with `0.15` and `1`. Land fallbacks are taiga scrub / plains / savanna scrub with `0.3` and `1.7`. These are constructor thresholds, not a complete reconstruction of the Engine selector's interpolation and normalized-climate semantics.
5. The wetland fallback overrides selection: only above `BiomeTemps.COLD`, moisture greater than `0.65` chooses its warm swamp fallback, greater than `0.4` chooses marshland, otherwise no fallback. The numeric value of that **Engine** `BiomeTemps.COLD` constant has not been authenticated; it must not be substituted with the similarly named mod metadata constant.
6. [`BiomeModifierManager.java`](https://github.com/TerraForged/TerraForged/blob/3ada751ac25cd3eecee5d198870a3abb89734f52/src/main/java/com/terraforged/mod/biome/provider/BiomeModifierManager.java) applies ocean, beach, coast, desert-color/wetland, wetland, warm-lake, mountain, and volcano modifiers. Wetland choices explicitly include marshland / cold marshland / frozen marsh; the warm-lake modifier receives desert lakes and the custom lake. Exact Engine-only tests and thresholds remain to be recovered.

### Engine reference availability

The original git tree pins `Engine` to **`ff665ac1dc3f12c7db48324177a3abb80ca9ac61`**, and `.gitmodules` points to `https://github.com/TerraForged/Engine.git`. That repository returned not found during this audit. A read-only clone of the public `lambdadog/terraforged-engine-1.16` mirror succeeded, but its history did **not** contain the pinned object and uses older package names. It is not accepted as the exact reference.

Recover the exact Engine commit from an authenticated archive, original source distribution, or a mirror containing that Git object before claiming its precise normalized climate, wetland, or spawn rules. The classic mod's own sources above are sufficient to establish concrete missing registrations and surfaces; Engine unavailability is not permission to invent thresholds.

## Original active custom surfaces

[`SetupFactory.createSurfaceManager`](https://github.com/TerraForged/TerraForged/blob/3ada751ac25cd3eecee5d198870a3abb89734f52/src/main/java/com/terraforged/mod/chunk/SetupFactory.java) actually wires:

- Replaced frozen-ocean surfaces: deep frozen ocean uses iceberg height/depth `30/30`; frozen ocean uses `20/15`.
- Appended **Bryce** surface on the custom bryce biome: seed offset `12341341`; terraced ridge noise with masked strength reduced by gradient; terracotta spires up to a nominal `30`-block added height, with explicit orange/ordinary/brown/yellow/light-gray/white terracotta strata.
- Appended **stone forest** on the custom stone-forest biome: seed offset `-2345768`; two terraced ridge fields at scales `50` and `55`; masked, slope-reduced stone columns up to nominal `50` added blocks, capped with dirt and grass.
- Appended **desert** treatment on desert / desert hills / desert lakes.
- Replaced **swamp** treatment on vanilla swamp and custom marshland.
- Appended **forest** treatment on forest / wooded hills / dark forest / dark forest hills.

[`MaskedSurface`](https://github.com/TerraForged/TerraForged/blob/3ada751ac25cd3eecee5d198870a3abb89734f52/src/main/java/com/terraforged/mod/api/biome/surface/MaskedSurface.java) uses biome-region edge multiplied by a river-mask mapping over `0..0.0005`. Restoring just the pillar shape without that transition mask would change the classic appearance and produce boundary artifacts.

`DunesSurface` and `SteppeSurface` classes also exist, but the inspected default setup does not register either. They are **source-present experiments/unwired paths**, not proof that all classic presets exposed them. Their restoration should be a deliberate optional feature after an executable reference establishes intended use.

The original has a `-DvanillaBiomes` compatibility option: `Environment.isVanillaBiomes()` controls `TFBiomeContainer.bakeBiomes`, remapping custom variants to the builder's parent IDs after decoration. Default source behavior enables this only when that system property is present. This is distinct from never registering variants or never generating their surfaces.

## Current selected baseline and 26.2 state

- Both `RTFBiomeData` and active `PresetBiomeData` retain the 17 names under **`reterraforged:`**, but their guarded bootstrap bodies register no biomes. `RTFBiomes.coldMarshland()` returns `null`. This gap was inherited from selected ReTerraForged 0.0.6 and is still missing.
- The real preset export recorded in `server-2026-11-03-export.json` contains no `data/reterraforged/worldgen/biome/` or `data/terraforged/worldgen/biome/` entries. A successful export therefore does not prove custom-biome restoration.
- Modern climate code maps the retained cell biome types to Minecraft temperature/humidity pairs (`cell/biome/type/BiomeType`, `cell/climate/ClimateModule`, `PresetNoiseRouterData`). It uses modern biome selection, not the original registered variant pool.
- Current `PresetSurfaceRuleData` composes vanilla overworld surface rules with optional strata. Vanilla eroded badlands, jungle, swamp, or meadow may supply broadly related scenery, but are not evidence that the classic bryce, stone-forest, scrub, lake, or marsh variants and masks are restored.

## Proposed restoration path with modern biomes retained

This is a proposal, not implemented behavior or a prerequisite to the first usable modern modpack build.

1. Define actual registered variants at the already-declared `reterraforged:*` keys, with a documented mapping from each original `terraforged:*` key. Preserve MIT attribution. Use modern biome builders/attributes and explicit modern parent mappings for renamed/removed 1.16 IDs. Do not claim existing-world migration merely because the names are mapped in documentation.
2. Separate the decision to select classic biome variants from `customBiomeFeatures`, which already controls vegetation/decoration. Provide an explicit compatibility option so modern vanilla and mod-added biome families remain available. Do not blindly replace the entire modern climate table with the old 1.16 set.
3. Implement one registry-aware variant-selection layer around the real modern biome resolver, driven by the retained cell terrain, river/lake/wetland, climate, and region fields. Preserve cave biomes and new vanilla families; default to leaving another mod's holder untouched unless explicitly opted into a replacement tag. Verify the actual `getNoiseBiome` path and `possibleBiomes` set together so `/locate`, structures, sampled biomes, and stored chunk biomes agree.
4. Add explicit selection tags and structure-feature eligibility for the new registered variants. Translate the authenticated original `OVERWORLD`/parent-type behavior into current tag APIs; verify each modern structure and vegetation family rather than guessing an inherited tag set.
5. Port the **active** classic surfaces and their masks as real terrain/surface features, beginning with the distinctive bryce and stone forest. Keep native modern cave/surface systems and new materials where appropriate; record seed offsets, shape equations, masks, and intentional height/biome adaptations.
6. Capture an executable classic reference and target comparisons: registry presence, distribution coverage, climate/terrain selection boundaries, feature/tree densities, masks at rivers/biome edges, neighboring chunks, save/reload, and screenshots. First judge intended terrain character and functionality; literal 1.16 block-for-block identity is a separate unsupported target across changed world height, caves, vegetation, registries, and game behavior.
