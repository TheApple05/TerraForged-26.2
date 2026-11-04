# 0019: Preserve TerraBlender initialization through the classic biome wrapper

Date: 2026-11-04, America/Los_Angeles.

The classic source delegates first, then replaces only explicitly eligible vanilla biome families. That policy alone does not preserve TerraBlender integration: TerraBlender 26.2.0.0.2 recognizes `MultiNoiseBiomeSource` directly in `LevelUtils`, and its chunk hook also checks that concrete source type. Wrapping it prevented initialization and the native per-chunk parameter-list clone.

The dev16 renderer/shader rehearsal reached both worlds and exited cleanly, but its custom source exposed only 72 possible biomes (55 vanilla plus 17 classic). The 59 expected overworld BOP biomes were absent. Its old automated result is retained as a renderer/UI rehearsal, not content compatibility acceptance. The stronger content gate requires the complete BOP membership and actual BOP samples; client readiness and shader state are checked separately.

The optional integration now unwraps only `ClassicBiomeSource` while TerraBlender decides eligibility and initializes its delegate. At completion it publishes the initialized delegate's possible biomes on the outer source through TerraBlender's native deferred-biome API, including when Minecraft previously memoized that outer set. Before actual chunk biome filling, it performs TerraBlender's parameter clone and uniqueness recreation on a cloned delegate and reattaches the classic wrapper. This retains both the classic selector and native per-chunk isolation.

All three mixins, including the private parameter accessor, are in the existing presence-gated TerraBlender mixin list. Base installations must boot without loading TerraBlender types. No other wrapper type is unwrapped and no third-party biome is mapped to a classic variant.

The implementation was checked against the publisher JARs and the primary TerraBlender 26.2 source (`51bb3d64ffd5ce698f5bc2a86647f6c4fc59b858`). The registry-aware codec QA initializes a decoded source through the real `LevelUtils` lifecycle before comparing its possible set; a bare codec decode cannot reconstruct TerraBlender's runtime initialization by itself. Exact package reports define the accepted loader and dependency versions.

This is independent of the older guarded TerraBlender material-rule MapCodec correction in decision 0014. Both corrections must retain their own regression coverage.
