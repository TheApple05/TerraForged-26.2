# 0021: Accept the target game's grass-block family as natural pillar footing

Date: 2026-11-04, America/Los_Angeles.

The first stronger dev19 natural pillar cases failed the strict stone-forest block check on both loaders. The filtered tile predicted positive heights, while the final chunk contained ordinary foliage above its old SURFACE baseline. A canopy-order explanation was plausible but was not established by final-state inspection.

A separate opt-in QA mixin observed the actual `ClassicPillarFeature` invocation. At the stone-forest target, all 89 positive shape/mask columns had grass-block footing at height 72, matching stored biomes and empty space above. Every one failed the feature's natural-footing predicate, and the feature returned false. At Bryce, the same probe observed valid terracotta footing and a true return; the block check recorded 79 added columns up to 25 blocks.

The publisher-hashed 26.2 server JAR provides the explanation. Its `data/minecraft/tags/block/dirt.json` contains dirt, coarse dirt and rooted dirt. Its separate `grass_blocks.json` contains grass block, podzol and mycelium. The resolved target API exposes `BlockTags.GRASS_BLOCKS`; grass is no longer covered by the old dirt-family assumption in this predicate.

The correction adds that grass-block tag to the existing natural stone/dirt/sand/terracotta footing predicate. The height/mask equations, biome taper, feature stage, hard-obstacle checks and empty-space requirement are unchanged. No canopy descent or foliage replacement is introduced. This avoids applying a broader speculative correction to an observed tag mismatch.

Final acceptance must repeat the same natural locations on both corrected exact packages, observe positive stone/grass/dirt columns and their caps, and compare the saved columns on reload. Feature-time diagnostics remain separate from the actual final-block evidence and visual review.
