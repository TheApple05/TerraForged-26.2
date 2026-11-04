# Native classic pillar arithmetic

The Rust core now implements the classic pillar mask, original legacy terrace
and added-height arithmetic. It retains the Java expression order and Java
minimum/maximum behavior. This is numerical groundwork for native Bryce and
Stone Forest; it does not place blocks or install the feature.

The [independent comparison](../validation/horizon-native-classic-pillar-math-v1.json)
covers **24,037 records per profile**, with zero differences in debug and release.
Two fresh JVMs compile and execute the unmodified project
`ClassicPillarMath.java`, verify the class's origin, and agree before the fixture
is frozen. No game APIs or Minecraft downloads are required. Rust receives the
input TSV only, never the expected outputs.

The corpus includes 16,418 ordinary terrace samples at maxima 1 and 0.999,
38 terrace edge inputs, 6,859 mask cases and 722 integer-height cases. Signed
zeros, subnormals, infinities and NaNs are included. Finite/infinite/zero results
compare raw float32 bits; NaN results compare as NaN, without claiming NaN payload
parity. The ordinary terrace subset also matches the independently frozen
original Noise2D reference digest:
`f6775d1e014df64c8a3171a5dd07e1f76a20013345c3a4f38ac296d20989d9c1`.

Capture and verification tools refuse replacement fixture/report directories.
CI checks the frozen Java results on Windows and Linux. Targeted Clippy passes
with warnings denied and the existing host policy allowing source API arity
(`clippy::too_many_arguments`); the first standalone invocation exposed the
unchanged core's six arity warnings and is not claimed as an unconditional lint
pass. A routine DeepSeek first-pass review supplied no concrete defect; Codex
reviewed the draft and retained the observed cross-language results as evidence.

Pillar noise construction, palettes, biome-boundary tapering, block/fluid checks,
structure interaction, feature-step execution and natural scenery remain
integration work. Arbitrary terrace maxima beyond the two source values are
not qualified by this corpus. The [resource transformer](classic-biome-resources.md)
declares the custom algorithm, but declaration alone does not run it.

[Code checks](../validation/horizon-native-classic-pillar-codechecks-v1.json) also
record five passing existing core-library cache tests and the scoped lint policy.
