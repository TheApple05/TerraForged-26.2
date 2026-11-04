# River geometry, configuration and warp foundations

The core now includes original river geometry, bounds/intersections, shortened
rivers, RiverWarp and generation domains, plus river/lake/wetland configuration
and Variance. These are owned inputs/helpers for the forthcoming carvers and
network generator. They do not generate or carve a river network yet.

Two fresh genuine packaged Java game JVMs agree on 35,055 records / 178,781 fields;
native debug/release match all finite raw bits and explicit NaN classifications.
The corpus covers 45 geometries, 22 warp objects, 14,850 warp samples, 1,944 domain
records, 648 fork-width conversions, integer/height boundaries, squared-length
overflow, zero-length geometry, signed zero, atan2, extended intersections, bounds,
shortening, random checkpoints and configuration defaults.

Source quirks are retained. Variance.of stores its second argument directly as a
range, including the apparently reversed/min-max call sites. Constant variance
still consumes a random draw. River squared length is squared after rounded sqrt.
Shortening resets to the default radius. Fork conversions retain distinct Levels
overloads and wrapping integer operations. GenWarp ignores its continentScale
argument. RiverWarp.NONE is a distinct sentinel from a newly constructed all-zero
warp; directly requesting a sentinel offset still follows the source arithmetic.

Position APIs use float pairs instead of packing and immediately unpacking a long.
Both coordinates are compared against the actual Java packed-position API; finite
bits are preserved. Generated NaN payloads are classified, not claimed bit-identical.
Arbitrary mutable Java object aliasing and every numeric/configuration domain are
outside this value/geometry qualification.

Routine configuration boilerplate used DeepSeek job
`c219a28def274f2c9da1afcc44eaa731` (estimated USD 0.0039666, separate from Codex).
Codex corrected the guessed module import, integer-to-float comparison, wrapping
negations/subtractions, and reviewed the direct Variance range semantics before
integration. Codex implemented geometry/warps, built the independent Java oracle
and performed all capture/verification. No production Java or released JAR changed.

Run `freeze_river_foundation_reference.py`, then
`verify_native_registry.py --family river-foundations --report <new-report.json>`.
The expected corpus is independent, immutable and losslessly compressed. Actual
river/lake/wetland carving, network construction, caching and filtered tiles remain
required before exposing a full native cell sampler.
