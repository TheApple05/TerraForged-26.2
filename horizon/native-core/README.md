# Native procedural kernel

This isolated crate starts the procedural Horizon port. It is not activated in
the prepared-terrain bundle and cannot yet generate a terrain tile or chunk.

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
Other composed noises and the cell/tile pipeline remain unfinished in the
[native milestone](../../docs/horizon/infinite-native-milestone.md).

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
0..32 and report unsupported counts; arbitrary graph/parameter domains are not
certified by these finite vectors. The crate has no external dependencies and
forbids unsafe code.

Original ReTerraForged MIT attribution is retained with the source and in
`LICENSE-RETERRAFORGED`. The root Horizon component license also applies to new
integration code. This crate does not contain Minecraft binaries or resources.

White/Cubic and three ridge/billow variants also match 128,458 independent Java
fields per profile, including reported bounds and high octave behavior. See
[noise sources](../../docs/horizon/noise-sources.md); use `--family sources` and
`--family power` for the source corpus and fractional-power foundation.
