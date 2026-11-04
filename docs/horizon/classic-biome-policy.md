# Native classic biome policy

The Rust policy matches **154,936** independent observations from the exact
published Java dev23 Fabric JAR in both debug and release. The JAR SHA-256 is
`1c9e5e7e8d286c145a647f466b2f908cda721d10672c40949864f1015570e886`.
This is the deliberate 26.2 revival policy from `ClassicBiomeSource`, not a
reconstruction of unavailable original Engine biome selection.

Two fresh JVMs execute the packaged class, verify its code-source origin and
agree byte for byte. The frozen corpus covers all 17 variant selection paths,
33 parent families (including caves, other modern vanilla biomes and a modded
biome), nine terrain identities, 26 raw region inputs, 19 temperatures and
8,218 region-hash inputs. Threshold cases use an inverse integer hash to select
inputs around the source's 24-bit choice boundaries. Nonfinite region/temperature
values keep the parent; inherited lake terrain does not become the exact
`TerrainType.LAKE` singleton. Wetland flags and beach categories retain their
source semantics.

See [policy evidence](../validation/horizon-native-classic-biome-policy-v1.json)
and `horizon/tests/fixtures/classic-biomes-v1`. Rust receives only inputs, never
the expected observations. `capture_classic_biome_reference.py` refuses changed
production JARs and replacement capture/fixture directories; the verifier checks
the frozen hashes and refuses replacement reports.

The first capture parser rejected Minecraft's logger-prefixed origin markers;
that successful Java process and its output remain in ignored QA storage. The
corrected capture reads the markers from the combined logger stream, then
requires two new processes to agree before freezing.

This policy qualification does not restore the native biome registry, features,
tags, natural reachability or client/save behavior. A world-owned host mapper is
being integrated before the pipeline's catalogs are built. Its wrapper must
preserve parent cave biomes below quart Y zero and use the completed filtered
cell for surface selection. Natural generation and feature acceptance follow
that integration separately.
