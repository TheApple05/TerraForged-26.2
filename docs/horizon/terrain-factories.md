# Original terrain factory graphs

The core now translates Populators.java's steppe, plains, plateau, both hills,
dales, badlands, torridonian, all mountain variants/chain, fancy erosion, border,
deep ocean, shallow ocean and coast construction. Raw graphs, shared caches,
settings scales and constructor seed order are preserved. VolcanoPopulator and
TerrainProvider's composite ordering/shuffle are separate unfinished work.

Two fresh genuine packaged Java game JVMs agree on 108,361 records. Native debug
and release match 1,598,425 fields per profile across 20 branches, four setting
cases and six signed32 seeds. The corpus observes 58,080 complete dirty Cell
snapshots, 49,368 raw base/height/erosion/weirdness records, 408 graph metadata
records and 480 constructor cursors. Both regular and fancy mountain branches
are exercised. Finite floats match raw bits; nonfinite values are classified.
Reverse query order and four parallel readers also pass.

The tiny-horizontal case deliberately exercises the source's division-by-zero
scale behavior. Its 4,356 NaN field classifications are preserved, not silently
replaced with terrain. This is not a supported full-kernel preset; the complete
generator still needs configuration/domain validation. Default, scaled and zero
vertical-scale cases have no NaN observations in this corpus.

Factories use Java Math.round for horizontal scales, distinct from the legacy
NoiseUtil negative-half rule. The core now has a separate round_java helper,
observed at 25 literal float bit patterns, including neighbors of half values,
signed zero, integer limits and nonfinite casts. RegionSelector uses that helper
too; its prior complete composition corpus still passes both profiles.

DeepSeek draft jobs were `8d977a04c65947e5a5af642f358191de` (estimated USD
0.0028437) and `1ffde6bdd1394436aa9d2a3af06949cf` (estimated USD 0.0020526).
Codex discarded the first draft's invalid Rust/extra wrappers and erroneous
dales expression, then implemented those methods directly from Java. The second
draft's plateau/badlands structure was retained after correcting float syntax,
helper namespaces, ownership, constant wrapping and a shadowed helper name.
Codex implemented the other factories and performed all integration and genuine
Java verification. These estimates are separate from Codex usage.

Run `freeze_terrain_factory_reference.py` with two genuine reports and then
`verify_native_registry.py --family terrain-factories --report <new-report.json>`.
Frozen observations remain independent of native output and cannot be replaced.
No production Java source or published JAR was changed. This remains a cell
kernel foundation, not an activated Horizon chunk generator.
