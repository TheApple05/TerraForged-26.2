# Terrain regions, composition and random draws

The native region module now matches the original jittered NATURAL-distance
selection, second-distance edge function and 1.5 power shaping. Terrain and ocean
populators, RegionLerper, Blender, both ContinentLerpers and weighted RegionSelector
are immutable owned compositions. Graph depth/expanded-node budgets also cover
their child noise graphs. This is the composition layer; full terrain factory
graphs, VolcanoPopulator and TerrainProvider construction are still required.

Two fresh genuine packaged Java game JVMs agree on 58,305 Cell records. Rust
debug/release match all 1,399,320 fields per profile: 24 region configurations,
27 composed populators, 13 by 13 coordinates and 11 edge controls. Every record
contains all 24 Cell fields. Tests cover seed wrapping, signed zero, +/-30 million,
dirty state, 2.5 weight rounding, skipped zero weights, all-zero fallback, unweighted
nodes, all three interpolations, exact blend endpoints, extrapolation and nested
compositions. Source nonfinite outputs are classified explicitly. Four parallel
readers and selector bounds/depth/node/weight rejection guards also pass.

Blending intentionally retains source field semantics. RegionLerper blends height,
erosion and weirdness, retaining the upper terrain type in intermediate cases.
Blender additionally selects terrain at its configured midpoint. ContinentLerpers
blend only height; other values come from the last applied child, including exact
endpoints. No generic interpolation helper silently broadens those mutations.

Selector allocation is explicitly limited to 65,536 entries and finite nonnegative
weights. Out-of-range selector queries return an error; there is no clamped fallback.
The owning full kernel must still validate query/configuration domains. Arbitrary
invalid inputs and process-global legacy registration history are not qualified.

Terrain shuffle and river construction use the original Java Random draw stream.
The core implements only the four needed Random methods, seed reset and descending
collection shuffling, using the [documented Java SE algorithm](https://docs.oracle.com/en/java/javase/25/docs/api/java.base/java/util/Random.html).
Two fresh real JDK25 JVMs independently agree on 2,718 records / 67,545 fields;
both Rust profiles match exactly. Nine signed64 seeds, eight bounds (including
rejection-heavy bounds), 48-bit seed aliases, resets, shuffle lengths 0..4096 and
post-shuffle draw checkpoints are covered. Unused Gaussian/double/long/byte APIs
and thread-shared mutable Random are not implemented or claimed.

DeepSeek routine work used jobs `28cb4476485942488cfce2c56ccddabe`
(call-site extraction, estimated USD 0.001347) and
`4c4e8518b9274fec9471924f5695f713` (RegionModule translation, estimated USD
0.0019071). Codex checked draw order against the actual bodies (roots precede
shuffle), corrected the guessed Rust module import, added owned derives, reviewed
the arithmetic and performed all independent capture/integration/verification.
These are DeepSeek estimates separate from Codex usage.

Fixtures and verifiers refuse replacement. Run `verify_native_noise.py --family
random` and `verify_native_registry.py --family populators` with new report paths.
No production Java source or published JAR was changed.
