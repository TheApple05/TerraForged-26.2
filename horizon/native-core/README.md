# Native procedural kernel

This dependency-free crate implements the procedural cell and filtered-tile
kernel for the supported TerraForged presets. It is not activated in Horizon
and does not yet produce native game chunks. Current scope and remaining work:
[port checkpoint](../../docs/horizon/PROGRESS.md).

The complete default kernel matches all 310,104 independent Java records /
7,442,496 fields in debug/release, including 12 complete filtered tile backings
and halos. Separate corpora cover terrain/climate configurations, river networks,
filter stages and zoomed tiles. See [tile qualification](../../docs/horizon/tile-filters.md).
Public modules include Heightmap, RiverGenerator, owned river caches, TileBuffer
and WorldFilters. Expected terrain is never supplied as an input to the core.

The adapter crate loads bounded preset/registry JSON separately. Core arithmetic
uses original Java float order, signed zero, wrapping integers and observed RNG/
trigonometry/rounding semantics. It retains the original MIT attribution, uses no
external dependencies, forbids unsafe code and contains no game assets or binaries.

## Foundation capture history

The following notes describe the initial arithmetic qualification, before the
later graph/cell/river/tile implementations linked above.

The initial scalar/hash routines preserve Java wrapping integers, float operation
order, signed zero and the inherited negative-integer floor behavior. The probe
checks the supported columns of the existing independently captured, pinned Java
reference fixture. The vector tables and Perlin/Simplex static/fractal modules
also match 34,976 independent Java records (56,080 values) in debug and release.
This includes all 296 vector entries, negative boundaries, signed32 seed
endpoints, zero/one/multiple octaves, interpolation modes and seed behavior.
Perlin2/Simplex2 now match a second independently captured 34,976-record corpus
(56,080 values per profile). These retain the 24-gradient Perlin variant and
Simplex2's exact normalization/scaler, not aliases of the first families.
The [native milestone](../../docs/horizon/infinite-native-milestone.md) retains
the full implementation and acceptance contract.

Run `python horizon/tools/verify_native_scalars.py` from the repository root.
The verifier checks both debug and optimized Rust output, retains every mismatch,
and never regenerates expected values from the Rust implementation.

Run `python horizon/tools/verify_native_noise.py --report <new-report.json>` for
the Perlin/Simplex foundation. The committed expected fixture came from two
fresh runs of the genuine packaged Java23 classes, whose origins and source
hashes are checked. `capture_noise_reference.py` is the separate Java-only
capture tool; the native verifier never invokes it or updates expected values.
Add `--family second` to the native verifier to check Perlin2/Simplex2. Its
independent expected fixture is `noise-reference-v2.tsv`; neither verifier
replaces prior reports or reference bytes. `capture_noise_reference.py` accepts
the same family option and still runs two genuine fresh Java processes.
`generate_noise_vectors.py` checks all vector literals against the source hash
already frozen by the original core fixture. Each verifier refuses to overwrite
existing evidence.

Perlin deliberately uses its stored seed and ignores its compute-time seed;
Simplex uses the compute-time seed. The unusual negative-integer floor and
duplicate gradient entries are retained. Constructors bound octave counts to
the supported source-specific ranges and report unsupported counts; arbitrary graph/parameter domains are not
certified by these finite vectors. The crate has no external dependencies and
forbids unsafe code.

Original ReTerraForged MIT attribution is retained with the source and in
`LICENSE-RETERRAFORGED`. The root Horizon component license also applies to new
integration code. This crate does not contain Minecraft binaries or resources.

White/Cubic and three ridge/billow variants also match 128,458 independent Java
fields per profile, including reported bounds and high octave behavior. See
[noise sources](../../docs/horizon/noise-sources.md); use `--family sources` and
`--family power` for the source corpus and fractional-power foundation.

All 41 registered noise types, five domain types and two curve functions now have
finite numerical fixture coverage in debug/release. See
[final numerical modules](../../docs/horizon/noise-final-modules.md). This does
not by itself qualify later visitors, registry loading or cell/tile stages; their
separate evidence is linked from the checkpoint. Native chunks/client/save remain unfinished.
