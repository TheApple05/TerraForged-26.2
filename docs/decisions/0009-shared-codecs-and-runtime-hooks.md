# 0009: Shared serialization and runtime adapters

Date: 2026-11-03. Status: first full source compiler pass succeeded; runtime validation continues.

## Serialization

Minecraft/DFU dispatch now expects `MapCodec` implementations. Migrated the eight custom type registries and their terrain/noise/domain/curve implementations to map codecs. Scalar enum/value codecs remain ordinary `Codec` values where they are fields, rather than dispatch types. Record field names and calculation methods are retained.

Explicit schema repairs accompany this migration:

- Noise constants retain the compact numeric shortcut in the aggregate codec; the explicit typed constant form has a `value` field.
- The typed interpolation form wraps its scalar enum in `value`; normal interpolation fields remain scalar enum strings.
- The cache noise wrapper has a named `noise` field. Its prior scalar/holder codec was not a map-shaped dispatch implementation.
- The inherited `SimplexRidge` serializer was declared for and constructed `PerlinRidge`, despite being attached to the simplex record. It now describes `SimplexRidge`. Its sampling formula is unchanged. Registry/export round trips must verify that named noise types retain their identities.
- Density records use map codecs and the 26.2 child-traversal contract. The two directly tested adapters and the inherited spline schema defect are covered in [decision 0003](0003-density-traversal.md).
- Registry datapack export uses the lookup provider's `createSerializationContext`; constructing an independent `RegistryOps` discards the patched provider's universal owner and can reject valid registry references.

Legacy preset import compatibility is not established solely by these declarations. Real preset export, reload, and field comparison remain required. The complete source compilation gate does not prove every serialization path.

## Surface and cave integration

The custom strata rule uses a map codec and reviewed access to current surface context fields. Vanilla surface rules now receive the actual biome lookup. The removed weird-scaled sampler call uses 26.2's quantized spaghetti rarity helper, retaining its threshold/rarity structure while using the current vanilla noise implementation. This is a vanilla cave integration adaptation, not a claim of bit-identical cross-version cave blocks; the mod's noise and landform arithmetic has not been redesigned.

## Runtime hooks

- `ChunkMap` initializes registry context at the end of construction using its actual `level` and `randomState` fields, avoiding obsolete constructor parameter types.
- `NoiseChunk` initializes the tile reader immediately before the constructor obtains its noise router. Its fluid-picker argument modifier uses the actual parameter type/position and complete target signature, rather than the obsolete local name/index.
- Template reload follows the successful completion of `MinecraftServer.reloadResources`, updating the resource manager and clearing the cache. The prior optional obfuscated hook could silently never run.
- The removed vanilla executor-shutdown helper is replaced with bounded shutdown of this mod's own generation/cache executors, restoring interruption status on interruption.
- Optional TerraBlender mixins are listed once and filtered by mod availability. The plugin no longer supplies duplicate entries already in the configuration.
- Both mixin configurations use a verified `JAVA_25` compatibility level. The imported Fabric configuration contained invalid JSON separators, now corrected.

Required mixin application, loader initialization, save/reload, custom world generation, optional mod combinations, and client rendering still require actual packaged runtime evidence.
