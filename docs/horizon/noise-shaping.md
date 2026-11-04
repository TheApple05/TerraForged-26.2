# Native shaping graphs

Sin, Power, PowerCurve, Curve, Gradient, Blend, Alpha, Boost, Steps and Threshold
now have numerical graph implementations. Their compute results and inherited
reported bounds match 85 genuine Java constructor graphs: 147,475 records and
147,645 fields per native profile, with zero differences in debug and release.
The corpus also exercises graph wrappers for all five preceding noise sources.

Two fresh production Fabric JVMs captured the expected values from the exact
published dev23 artifact. The freezer checks both successful clean process exits,
distinct PIDs, the production artifact hash, byte-identical captures, and original
class origins in full JVM class-load logs. No registry or mixin stubs substitute
for the genuine classes. The optional same-package oracle is confined to the
separate Fabric QA companion; it is absent from ordinary QA builds, both released
production JARs, and the NeoForge automatic module.

Cases cover zero/signed-zero/one and extrapolating sine alpha, fractional and
negative powers of positive noise, integer powers of negative inputs, constant
PowerCurve ranges, shifted bounds, both registered curve functions, gradient
boundaries, dynamic alpha, Boost's factory remapping and minimum iteration,
fractional/negative steps and reversed slope ranges, threshold equality, and all
three blend interpolations. Boost rejects counts above 1024 as an explicit native
resource limit; this guard is independently observed by the native probe.

The implementation preserves reported metadata even where it is not a valid
mathematical interval. Those endpoints must not become Horizon optimizer bounds.
The raw graph API does not yet implement registry decoding, the complete Noises
factory shortcut set, or mapAll visitor semantics. In particular the source's
PowerCurve visitor omits applying the visitor to its parent; that behavior remains
part of the pending visitor work. Arbitrary degenerate/nonfinite graph domains,
all possible inputs and complete libm platform equivalence remain outside the
finite corpus. This crate still cannot produce a native terrain tile or chunk.

Use `verify_native_noise.py --family shaping --report <new-path.json>`. The native
verifier never writes expected data. `freeze_noise_composition.py --family shaping`
is a separate Java-only capture freezer and refuses to replace frozen bytes.
