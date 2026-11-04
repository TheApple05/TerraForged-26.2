# Terrain provider and volcano

The native provider now follows the original constructor order: build every
mixable factory before filtering weights, construct unmixables, register/combine
ordered pairs, append unmixables, then shuffle with the original Java Random
stream. Composite noises retain the source's raw component graphs and unscaled
base/height behavior. Terrain IDs live in a kernel-owned registry. The caller's
Seed cursor remains unchanged because the provider uses its root-offset copy.

VolcanoPopulator now constructs the original lookup height, cone, warps, power,
gradient and lowland noises. Its crater inversion and lowland blend preserve
float evaluation order. Some branches intentionally retain the Cell's incoming
terrain type; they are not replaced with an unconditional volcano assignment.
The source's water/river-mask conversion from pipe to outer volcano is preserved.

Two fresh packaged Java game JVMs agree on 230,292 records. Native debug/release
match all 5,521,692 fields per profile. Four settings and six signed32 seeds cover
642 ordered provider nodes, weighted selection at 17 identities, all 24 fields in
229,602 Cells, 101,400 volcano grid samples, constructor cursors and type conversion
thresholds. The genuine grid exercises outer volcano, pipe and retained incoming
terrain. Actual registered ground is checked against the predefined binding;
the original provider uses real datapack registry holders, not a substitute.
Native reverse query order and four parallel readers also pass.

The settings cover default fancy mountains, plain mountains, changed scales/seed
offset/weights and zero weights, including all-zero selector fallback. Duplicate
mixable types disabled in the weighted case preserve the original composite
registration order. Arbitrary prior global Java registry history remains outside
the owned-kernel qualification; no global mutable registry is introduced in Rust.

Expected data is losslessly gzip-compressed for repository storage. Both compressed
bytes and decompressed original Java output have independent hashes. Compression
does not reduce field coverage or originate any expected values from Rust.
Freezers and verifiers refuse to replace evidence.

Run `freeze_terrain_provider_reference.py`, then
`verify_native_registry.py --family terrain-provider --report <new-report.json>`.
Complete Heightmap/river/climate/filtered-tile construction and native chunks,
rendering, saves and launcher integration remain unfinished. Java production
source and published artifacts are unchanged.
