# 0003 — Adapt density codecs and child traversal to 26.2

Date: 2026-11-03 (America/Los_Angeles). Status: implemented and verified for two production adapters only.

## Context

`ClampToNearestUnit` and `LinearSplineFunction` could not compile against Minecraft 26.2. `DensityFunction` now requires `mapChildren(Visitor)` and provides a recursive default `mapAll`. The old holder-helper codec constant is absent, and `KeyDispatchDataCodec` now wraps a `MapCodec`.

The target APIs and their traversal behavior were inspected directly with `javap` against the resolved 26.2 game classes. The bytecode of the default recursive visitor calls `node.mapChildren(recursiveVisitor)` before applying the caller's visitor to that resulting node. This is consistent with the [26.2 migration primer](https://github.com/NeoForged/.github/blob/main/primers/26.2/index.md).

## Decision

- Change these two record codecs to `MapCodec` using `RecordCodecBuilder.mapCodec` and `DensityFunction.CODEC` for child functions. Return `KeyDispatchDataCodec.of(CODEC)`.
- Implement `mapChildren` by applying its visitor to each immediate child and constructing the resulting record. Do not apply the visitor to the current node, and do not invoke a child's `mapAll` from `mapChildren`.
- Let the game's default `mapAll` supply recursion. It reaches the spline's input and every point-value child, including descendants inside a point value, exactly once per tree occurrence. The old spline implementation only applied the caller's visitor directly to each point value and could miss deeper descendants.
- Keep scalar evaluation, array evaluation, interpolation, clamp rounding, and min/max arithmetic unchanged. Reconstructing a mapped spline recalculates its bounds from the transformed point values, as before.

Direct-child and recursive traversal are tested separately. A direct visitor sees three immediate children of the tested spline and no grandchildren. The recursive test transforms three different vanilla constant leaves under four custom clamp nodes and one spline node; each is visited exactly once, and the deepest point-value leaf is changed.

## Separately justified serialization correction

The inherited spline used `Codec.pair(Codec.DOUBLE, DensityFunction.HOLDER_HELPER_CODEC)` for a point. Mechanical substitution of `DensityFunction.CODEC` compiled, but the actual target JSON codec failed when asked to round-trip a spline. DFU's `PairCodec` composes both codecs into the same encoding prefix; it does not automatically construct a two-element JSON array.

The observed failure included: `Do not know how to append a primitive value 1.0 to -1.0`, and the same failure when combining a location with a vanilla gradient object. The failed test is retained in [density-legacy-codec.json](../validation/density-legacy-codec.json).

Encode each point with explicit fields instead:

```json
{"location": -1.5, "value": -2.25}
```

`value` uses the real density-function codec and can also contain a vanilla function object. This corrects serialization while preserving the in-memory point representation and interpolation. A non-empty list is still required. Unsupported untyped point arrays fail explicitly; no compatibility parser is invented for an encoding the inherited codec could not produce for these configurations.

## Consequences and limits

The targeted production slice now compiles against actual 26.2 dependencies, bootstraps Minecraft's real built-ins, and passes JSON round trips and arithmetic/traversal tests. No game-class stubs or loader access modifications are involved.

These tests call the record codecs directly. Custom density-type registry dispatch, the remainder of the density system, Fabric startup, all terrain generation, and the distributable mod remain separate migration work.

Inherited quirks are visible rather than silently repaired: clamp arithmetic adds one unit after integer conversion, zero resolution yields positive infinity in the tested case, and spline bounds cast child bounds through float. Validation records preserve those behaviors; they are not endorsements of arbitrary or invalid settings.
