# Horizon procedural port checkpoint

Java26.2 and Horizon26.3 are separate targets. The Fabric/NeoForge dev23 modpack
candidate is published together in the
[dev23 prerelease](https://github.com/TheApple05/TerraForged-26.2/releases/tag/v0.1.0-dev.23%2B26.2).
Its exact JARs, qualification evidence and limitations are preserved. Native work
does not rebuild those release artifacts.

The finite prepared-terrain Horizon bridge and its minimal constructor hook are
verified intermediate work. The prepared package is excluded from the Java
release. It does not provide infinite native generation.

The isolated native foundation now includes independently observed scalar/hash/
coordinate/seed/RNG behavior, all 41 numerical noise types, five domains, two
curve functions, original graph traversal, and the bounded registry adapter.
Inherited codec/nonfinite limits are explicit in noise-registry.md. Cell values,
height levels, coordinate layout, terrain inheritance/composites and the original
11-category climate palette also have genuine Java fixture coverage. The cell
preset model and source noise factories are covered in preset-model.md. These
are finite numerical qualifications; none is an activated Horizon generator.

The MULTI_IMPROVED/default continent geometry now matches independent Java cell,
center and edge/ocean search observations; see advanced-continent.md. Its river
cache is not implemented. The next required milestone is constructing the full
terrain/river/climate cell pipeline in the original seed order.
Region geometry, basic terrain/ocean populators, region/continent blending and
weighted selection now match Java too; region-populators.md records their scope.
The used Java Random methods and collection shuffle sequence are qualified.
All 20 regular/fancy terrain factory branches now match Java observations in
terrain-factories.md. VolcanoPopulator and TerrainProvider composite/seed/shuffle
ordering also match independently captured Java data; see terrain-provider.md.
Complete Heightmap terrain/climate construction now matches five-preset Java
stage observations; heightmap-climate.md records the boundary and explicit guards.
Terrain also matches all 726 terrain records from the older full kernel-v1 corpus.
River networks/carving and filtered-tile construction remain required. There is
no full apply sampler until the real river stage is ported and qualified.
River geometry, warps/domains, configurations, variance and fork conversions now
match independent Java observations; see river-foundations.md. River, lake and
wetland carvers now match 3,919,986 observed Java fields per native build profile;
river-carvers.md records their scope. Network generation remains unfinished.
Then compare every field against the frozen kernel-v1 observations, complete
filtered tile backings/halos, seams, cache ownership, query order and parallel work.
Never substitute height-only fill, prerecorded terrain or a snapshot fallback.

After that, connect the complete kernel to real native density, biome, surface,
cave, decoration and structure execution. The 17 classic registered biome IDs,
world-save reopen, actual client rendering, native mod combinations and launcher
packaging remain required acceptance work. The palette's 11 internal climate
categories are not those 17 registered biomes.

All expected numerical observations originate from the exact published Java JAR
or original source-owned data, independently of Rust output. Capture/freezer and
verification tools refuse replacement. Failed reports remain alongside corrected
runs. Windows/Linux CI reruns the native families and both Java loader builds.

The Horizon resource boundary still applies: use only the user's own locally
prepared26.3 resources. Do not fetch Minecraft game assets for this port. No
native chunk/client/save acceptance has been claimed.
