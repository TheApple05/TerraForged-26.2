# Source-owned trig and complete helper parity

Native NoiseUtil now preserves the original4096-entry sine lookup, its exact radians-to-index float constant, masked Java integer conversion, cardinal entries and the inherited cosine offset. The table is16KiB of mathematical lookup data from the MIT NoiseUtil source; it is neither Minecraft assets nor terrain snapshots. It belongs to the native crate's source data and does not depend on Java at runtime.

The read-only Java probe inspects the genuine production class's static table/constants without mutating them. Two fresh JVMs produce identical reference bytes. The source hash remains the original pinned NoiseUtil hash used by the pre-existing helper fixture. The installer verifies that source identity and exact table hash before copying the production lookup. Expected data is never generated from Rust trig functions.

Independent trig/sqrt coverage matches28,680 records and81,952 fields in debug/release. It includes neighboring representable floats around every table sample, positive/negative angles, signed zero, subnormal/normal boundaries, maximum finite input and double-sqrt-to-float conversion. This is a finite corpus, not every floating-point/nonfinite input or fractional-power proof.

The complete existing original Java helper fixture now also matches all2167 records and26,943 values in both profiles: hash/gradient/cell vectors, all recorded math columns, signed coordinate/float packing, seed sequences and the frozen RNG streams. That reference predates the native implementation and remains unchanged. This closes the earlier native subset gap for that fixture; it is not the complete cell/tile terrain pipeline or every public utility input domain.

The native verifier binds text source with LF normalization and binary lookup bytes exactly. Windows/Linux CI runs both corpora alongside the four base noise families and composed graph corpus. Fractional power, the remaining numerical nodes, registry decoding/visitors and native terrain activation remain unfinished. Published Java JAR bytes are unchanged.
