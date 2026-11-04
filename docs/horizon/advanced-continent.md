# Native advanced continent geometry

`AdvancedContinent` translates the original MULTI_IMPROVED continent geometry,
including AbstractContinent's edge/ocean searches. It owns immutable noise graphs
and applies only the original continent fields to an owned Cell. Skipped cells
retain their prior values, including dirty values. The default cell (0,0) is never
skipped. Constructor seed consumption wraps at signed32 and matches the original
seven draws. The original river-cache constructor consumes no Seed.next calls;
river construction/cache behavior remains separate and unfinished.

Two fresh packaged Java game JVMs agree on 26,113 records. Rust debug and release
match all 610,898 fields per profile: four predefined settings, six signed32 seed
cases (including constructor wrap), 23 by 23 coordinates out to +/-30 million,
25,392 fresh/dirty Cell snapshots with all 24 fields, nearest centers, coastal
edges, 576 edge/ocean search pairs and 49 bisector/line helper cases. Native
reverse query order and four parallel readers also preserve results. This checks
geometry, not the complete terrain, river, climate or filtered-tile pipeline.

Source quirks are preserved: corrected centers cast toward zero; distance-to-edge
doubles its initial bracket before refining; continent variance uses the original
clamped map; qualifying coastal distances are corrected twice. Source noise factory
defaults and explicit float operation order remain covered by independent fixtures.

The native constructor explicitly bounds scale to 2..i32::MAX/4 and finite setting
values. Queries reject nonfinite or unrepresentable neighbor-grid coordinates.
These are native limits, not a claim that every source configuration is supported.
Other continent implementations, rivers/cache, spawn policy and whole-kernel seed
construction order remain unqualified.

The first two capture runs omitted the generated noise datapack and failed while
constructing GeneratorContext (missing terrain/ground). Both failed producer
reports are retained. The corrected runs load the actual datapack, use the original
packaged classes, close every context/captured cache, and agree byte for byte. No
fixture was made from either failed run or from native output.

Run `freeze_continent_reference.py` with two genuine producer reports, then
`verify_native_registry.py --family continent --report <new-report.json>`. Both
tools refuse to replace prior evidence. The runtime companion is test-only and is
excluded from the published Java artifacts.
