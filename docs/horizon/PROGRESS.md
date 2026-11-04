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
center and edge/ocean search observations; see advanced-continent.md. Its native
river maps now use an owned bounded cache without global polling threads.
Region geometry, basic terrain/ocean populators, region/continent blending and
weighted selection now match Java too; region-populators.md records their scope.
The used Java Random methods and collection shuffle sequence are qualified.
All 20 regular/fancy terrain factory branches now match Java observations in
terrain-factories.md. VolcanoPopulator and TerrainProvider composite/seed/shuffle
ordering also match independently captured Java data; see terrain-provider.md.
Complete Heightmap terrain/climate construction now matches five-preset Java
stage observations; heightmap-climate.md records the boundary and explicit guards.
Terrain also matches all 726 terrain records from the older full kernel-v1 corpus.
The complete unfiltered Heightmap apply sampler now includes real procedural
river generation and carving. Full tile generation/filtering is also implemented.
River geometry, warps/domains, configurations, variance and fork conversions now
match independent Java observations; see river-foundations.md. River, lake and
wetland carvers now match 3,919,986 observed Java fields per native build profile;
river-carvers.md records their scope. Network generation/traversal now matches
72 Java maps and 1,920 nodes; river-networks.md records the details. The full
unfiltered sampler also matches all 2,904 terrain/river/climate stage records from
the preexisting kernel-v1 corpus in both native build profiles. All 310,104 records
from that corpus now match, including 12 complete filtered backings/halos; see
tile-filters.md. Independent filter/brush/zoom cases also match 3,151,416 fields.
The bounded production column/tile sampler now matches 196,608 original Java
core columns across six seeds, including negative addressing and eviction/reload.
Its failure/unwind/concurrent-publication lifecycle tests pass in both profiles.
An optimized kernel-only benchmark is recorded in column-sampler.md. Native
density bindings now have a separate real-graph qualification in
host-density-graph.md. Full router/world integration, seam characterization,
real client/save and mod combinations remain required; kernel timings are not
game/FPS acceptance.
The ten cell density fields and custom clamp/spline/conditional-cache/noise
primitives now match 430,254 independent Java fields in both profiles; see
density-primitives.md. They are not bound into Horizon's density graph yet.
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
