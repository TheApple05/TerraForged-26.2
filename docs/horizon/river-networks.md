# Procedural networks and complete unfiltered cells

The native SimpleRiverGenerator and used BaseRiverGenerator path now generate
real root/fork/lake/wetland networks. Network traversal retains parent-warped
coordinates, river→wetland→lake order and recursive child order. Heightmap.apply
now executes terrain→rivers→climate, including post-river volcano type changes.
No prerecorded terrain or fallback sampler is involved.

Two fresh genuine packaged Java game JVMs agree on 78,972 records / 1,687,920
fields. Debug and release match all finite raw bits, integer/boolean values and
explicit NaN classifications. Four presets and six world seeds produce 72 maps,
1,920 tree nodes, 434 lakes and 562 wetlands. The corpus observes tree order,
river geometry, fork configurations, curves, warp parameters, root/child bounds,
original staged RNG checkpoints, 62,424 synthetic map cells and 3,888 complete
Heightmap cells. It includes zero-river, dense wetland, alternate scale/height,
seed-overflow and upper-32-bit world-seed collision cases.

The Java oracle obtains actual continent-cached maps, then separately invokes
the original root/shuffle/fork/wetland methods with a caller-owned Random. It
checks that staged construction matches the actual cached generator before
recording the RNG checkpoint. Expected values never come from Rust. The freezer
checks two process identities, original loaded class origins, immutable predefined
inputs, raw hashes and all record counts.

As a separate regression, all 2,904 unfiltered stage records / 69,696 fields from
the older kernel-v1 observations match in debug/release. This compares terrain,
rivers and both climate modes without modifying that preexisting oracle or
feeding expected cells to native code. Filtered-tile records are still excluded.

Construction uses an owned arena with stable IDs for Java builder identity.
Finished maps own trees with no cycles. The unused Java builder min/max bookkeeping,
unused additional-lake helper and unused alternate createFork entry point are
outside this supported construction path. Source root truncation, overlap tests,
lake insertion, random draw order and child Bounds.NONE are preserved.

Native cache ownership deliberately replaces Java's expiring global cache/poller:
32 maps per owning Heightmap, bounded LRU eviction, serialized misses and shared
Arc results. It creates no background threads. Tests cover repeat identity,
reverse queries, four parallel readers, capacity/invalid-count guards, LRU touch
and eviction, release of evicted/cleared maps, and concurrent same-key identity.
Dropping the owner drops the cache; retained caller Arcs remain valid. Java expiry
timing is not claimed equivalent, and a cold miss is serialized within one owner.
The native generator accepts riverCount 0..256 and limits construction to 65,536
nodes. Existing finite native preset/query guards continue to apply.

The immutable Network translation began with DeepSeek job
`3347447282524b34b027406746b6e8cd` (estimated USD 0.0013806, separate from Codex).
Codex corrected the intersection API name supplied in the task, reviewed every
mutation/coordinate order, implemented generator/cache/full sampler and performed
all independent capture, integration and verification.

Run `verify_native_registry.py --family river-networks --report <new-report.json>`
and `verify_kernel_terrain.py --stages unfiltered --report <new-report.json>`.
Windows/Linux CI includes both. These results qualify procedural cells, not a
complete Horizon world generator. Filtered tiles/halos, erosion/smoothing, native
density/biome/surface/cave/decorations/structures, real client/save and modpack
integration remain required. Released Fabric/NeoForge JARs are unchanged.
