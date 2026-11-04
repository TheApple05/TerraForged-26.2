# Independent TerraBlender wrapper review

Read-only review of the Java integration against TerraBlender 26.2 reference
`51bb3d64ffd5ce698f5bc2a86647f6c4fc59b858` and the actual local Minecraft 26.2
source bundle. This is separate from the native Horizon component. No
TerraBlender implementation was copied into that component.

No additional actionable issue was found for the supported fresh-world,
single-classic-wrapper topology. This source review supplements the recorded
runtime tests; it does not replace them.

- `MixinLevelUtils` unwraps the classic source both for `shouldApplyToBiomeSource`
  and all matching getter calls in `initializeBiomes`, so TerraBlender's own
  MultiNoise checks, cast and parameter initialization operate on the real
  delegate. Its tail publishes through TerraBlender's own deferred-biome API
  only after initialization, retaining the wrapper's variants and the modded
  delegate holders even when the original possible-biome set was memoized.
- `MixinNoiseBasedChunkGenerator` clones the actual initialized parameter list,
  recreates its uniqueness state, installs it on the cloned MultiNoise delegate
  and returns a classic wrapper around that delegate. The lifecycle matches
  TerraBlender's own per-chunk isolation. Either ordering of the two ModifyArg
  handlers leaves one isolated wrapped delegate; TerraBlender's handler does
  not clone a ClassicBiomeSource a second time.
- The invoker matches the real `MultiNoiseBiomeSource.parameters()` result.
  All new optional mixins/invoker are present in both the mixin JSON and the
  TB-only gate. The bootstrap-facing compatibility class references the names
  as strings and does not introduce a direct optional TerraBlender type.
- Product preset export and editor paths remove nested classic wrappers before
  adding one. Arbitrary hand-authored nested wrapper codecs are not part of
  this review's accepted topology.

The vanilla `doCreateBiomes` path composes Blender and below-zero-retrogen
resolvers before filling biomes. An empty Blender in a new world returns the
source directly, enabling the expected clone path. Nonempty blending/retrogen
can supply a resolver lambda that bypasses `instanceof`-based cloning in both
upstream TerraBlender and this adapter. Existing-world migration therefore
remains outside the verified scope; it is not newly certified by this review.

Fabric initializes TerraBlender during the server-constructor return before
Fabric biome modifications; NeoForge uses ServerAboutToStart. The reviewed
adapter leaves those lifecycle entry points intact.
