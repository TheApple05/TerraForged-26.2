# Complete filtered tiles and zoomed previews

The native core now implements owned Tile backings, chunk-order population,
zoomed coordinates, erosion brushes/droplets, modifiers, smoothing, steepness,
beach detection and quart corrections. WorldFilters keeps the original order:
optional erosion→smoothing, required steepness→beach, then optional corrections.
No source chunk snapshots are read by native code.

Every record in the older independent kernel-v1 corpus now matches in debug and
release: 310,104 records / 7,442,496 fields, including 12 complete 160×160 tile
backings (128 core blocks plus 16-block halos) across six world seeds. The earlier
Java manifest's nativeImplementationPassed=false is retained as historical
evidence; new reports record this later implementation's pass. Finite float bits,
names, integers and booleans are compared, not rounded displayed values.

Separate fresh genuine Java game JVMs agree on a second predefined corpus:
125,104 records / 3,151,416 fields. It covers all brush indices/weights for sizes
16/32/48, seven individual/combined filter stages on 12 controlled arrays, all 24
cell fields, flat-index edge queries, modifier range/inversion (including two
nested inversions), three level pairs, normal/zero/strong filter settings and
explicit NaNs. Actual TileGenerator.generateZoomed results cover four cases with
optional filters on and off, signed seed extremes, negative/large coordinates and
zoom 0.25/1/2/4. Native reversed requests and parallel tiles match their original
results. NaN payloads are classified; finite values require raw-bit equality.

Source quirks are retained. Tile.getCellRaw checks the flattened index, allowing
some out-of-column neighbors to wrap rows; a strict x/z bounds check would change
the original filters. Smoothing updates cells in place. Erosion adds deltaSediment
to its droplet even when a masked cell prevents changing the terrain. Its brush
initialization reuses interior offsets in the original order. Nested inversions
retain each float subtraction rather than canceling them algebraically. Radius
zero and finite unusual rates are observed without silently substituting defaults.

Native TileBuffer supports tile factors/borders 0..4. Erosion backings accept
16..512; WorldFilters bounds droplets/chunk to 0..4096, lifetime to 0..1000,
smoothing iterations to 0..64, radius to 0..64, and requires finite rates/volumes.
These are explicit supported-input limits, not claims that Java rejects all
outside values. One owning WorldFilters caches its last immutable erosion brush
set with Arc sharing, matching the source's last-size reuse. Tile arrays are fresh
owned values, not a process-wide pool. A bounded production tile cache is separate
remaining integration work.

The basic modifier/smoothing/steepness draft used DeepSeek job
`f07108a9f8514216a44eef626874ee2e` (estimated USD 0.0025212, separate from Codex).
Codex removed Java float suffixes, corrected guessed optional-index/absent APIs,
preserved nested float inversions, reviewed traversal/arithmetic, implemented
tile/erosion/required filters and performed all independent capture/verification.

Run `verify_native_registry.py --family tile-filters --report <new-report.json>`
and `verify_kernel_terrain.py --stages full --report <new-report.json>`.
The frozen references and producer reports are immutable; neither verifier can
generate or replace expected values. Both comparisons run in Windows/Linux CI.

This qualifies the listed numerical tile behavior. It does not establish all
presets/layouts, visually seamless exploration, production cache behavior or native
performance. Native density, biome registration, surfaces, caves, decorations,
structures, client rendering, save reopen, launcher packaging and mod combinations
remain required for a Horizon release. The Java Fabric/NeoForge release is unchanged.
