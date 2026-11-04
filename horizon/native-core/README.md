# Native procedural kernel

This isolated crate starts the procedural Horizon port. It is not activated in
the prepared-terrain bundle and cannot yet generate a terrain tile or chunk.

The initial scalar/hash routines preserve Java wrapping integers, float operation
order, signed zero and the inherited negative-integer floor behavior. The probe
checks the supported columns of the existing independently captured, pinned Java
reference fixture. Unimplemented gradient/vector/noise/cell/tile stages remain
in the [native milestone](../../docs/horizon/infinite-native-milestone.md).

Run `python horizon/tools/verify_native_scalars.py` from the repository root.
The verifier checks both debug and optimized Rust output, retains every mismatch,
and never regenerates expected values from the Rust implementation.

Original ReTerraForged MIT attribution is retained with the source and in
`LICENSE-RETERRAFORGED`. The root Horizon component license also applies to new
integration code. This crate does not contain Minecraft binaries or resources.
