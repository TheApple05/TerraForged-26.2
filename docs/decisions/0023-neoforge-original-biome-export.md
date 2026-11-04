# 0023: Derive NeoForge variants from original registry data

Date: 2026-11-04, America/Los_Angeles.

The dev22 combined NeoForge BOP/Repurposed Structures/C2ME/Lithium/FerriteCore server failed during spawn generation with a feature-order cycle involving warm beach. Its renderer-stack client had passed, so the server's running-world export path needed separate inspection.

The qualified server export shows the distinction directly: serialized `minecraft:warm_ocean` has no Repurposed Structures features, while derived `reterraforged:warm_beach` contains `repurposed_structures:dungeons/deep` and `repurposed_structures:dungeons/ocean_warm`. The common builder read `Biome.getGenerationSettings()` from a running NeoForge world. That getter exposes modified data, while NeoForge's normal biome codec retains the original registry data. Copying the modified getter into a new biome made those additions part of the new biome's original data. Native modifiers then ran again on load.

`PlatformServices.biomeForExport` supplies the appropriate loader data view. Fabric keeps its existing view. NeoForge reconstructs the biome from `modifiableBiomeInfo().getOriginalBiomeInfo()`, preserving original climate settings, temperature adjustment, effects, generation and mob settings plus the biome's immutable environment attributes. The classic builder uses that view for its parent and steppe ambience.

This retains actual pack-provided original data and lets deferred native modifiers apply once according to their own holder/tag eligibility. It does not remove, deduplicate or reorder another mod's live modifier registry. The same correction avoids baking modified mob spawn lists into a derived original biome. Existing explicit classic builder changes and holder rebinding remain intact.

The exact dev23 combined NeoForge export/generation/structure/save/reload case passes. Its full artifact and dependency hashes are in `compatibility-dev23-neoforge-content.json`. Earlier failing exports and the feature-cycle traceback remain visible. Other final-candidate runs are separate gates; one corrected stack does not establish universal modpack compatibility.
