# World-local 26.3 router plan

The native adapter now expresses the MIT `PresetNoiseRouterData` terrain formulas
in Horizon's eight-field 26.3 density schema. It supplies height/climate fields,
the original height quantization and offset/depth formulas, zero base 3D noise
and jaggedness, cave probability terms, and a surface search appropriate to the
new router interface. The structure beardifier is explicit in the final root.

This is a version adaptation, not a rename of the old 15-field router. In
particular, `chunk_surface_level` must return block heights. It wraps a top-surface
search over the source preliminary-density expression with the 0.390625
threshold and eight-block vertical step, then the current 16-block X/Z
interpolator. Assigning the old preliminary-density value directly would give
the wrong units to aquifers and surface rules.

Vanilla cave/spline helper definitions are read from the player's already
prepared 26.3 resources. Their bytes are not copied into the repository. Helpers
needed before an override receive distinct local aliases, preventing a
self-reference when applying a probability term. The native scope supplies
definitions through Horizon's real resolver, including newly named references.
`GeneratorData` and its shared maps remain immutable.

`horizon-native-host-router-v1.json` records 22,464 scalar values for all eight
fields over six seeds, 36 X/Z points and 13 heights. Debug and release agree
bit-for-bit. Checks require finite outputs, a surface result within the world
height range that ignores sample Y, unchanged vanilla offset evaluation after
leaving the native scope, empty released tile caches and release of the kernel
owner. Input resource hashes remain unchanged.

`horizon-native-router-climate-v1.json` compares five actual router outputs with
the independent genuine Java mapped-cell capture: 14,040 values per profile,
zero differences. It also checks that the surface varies and final density has
both solid and air signs. The capture covers a limited area; its observed
surface range of 8 to 88 is not a statement about the whole world's elevation
range. Timings in integration reports include compilation and are not benchmarks.

The assembled world library also passes Clippy with warnings denied. To
reproduce, stage with `stage_native_host.py --router`, run
`verify_host_router.py` with the existing prepared resource root, then run
`verify_router_climate.py` on that run's outputs. The tools refuse replacement
of evidence. No game files are fetched. CI continues to exercise source-level
graph/registry qualification; this resource-dependent integration runs locally.

The initial plan explicitly requires height 320, depth 64, sea level 63 and the
default noodle probability. It rejects unsupported vertical envelopes. Cave
helpers use the current 26.3 implementations; original full f64 cave/depth/
surface/chunk equivalence is not established. Vanilla 26.3 aquifer, material and
ore configuration has not yet been exercised with this plan in real chunks.
Source carver settings, classic biome/feature integration and custom presets
remain further work.

This milestone does not select or activate a world. Real terrain generation,
decoration, structures, spawn, save/reload, native client rendering, performance
and mod combinations remain required before calling the Horizon port ready.
