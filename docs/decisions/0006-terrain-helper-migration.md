# 0006: Terrain feature and provider APIs on 26.2

Date: 2026-11-03. Status: source adaptation implemented; full compilation and runtime verification pending.

## Serialization

26.2 dispatch codecs take `MapCodec` implementations. Convert the concrete chance modifiers, template placement/decorator types, biome modifiers, and structure rules to `MapCodec` while retaining their aggregate dispatch `Codec`s. Their field names, registry IDs, numeric ranges, chance arithmetic, and placement algorithms stay intact. Root custom registry declarations migrate to the matching generic types in the same integration pass.

Height-provider and placement-modifier type factories now return `MapCodec`s. Float providers changed more substantially: `FloatProvider` is an interface with `min()`, `max()`, `sample(RandomSource)`, and `codec()`, and the built-in registry directly stores map codecs. `LegacyCanyonYScale` now implements that interface and registers its real codec; its original random calculation and inherited declared bounds remain unchanged. The two runtime-only Poisson noise wrappers keep their existing unsupported serialization paths; changing the return type does not invent a codec for objects upstream never serialized.

## Feature placement and templates

- Replace block-position `ChunkPos` constructors with `ChunkPos.containing`, and use its record accessors. Coordinate conversion remains Minecraft's actual implementation, including negative positions.
- Chunk setters now take update flags rather than a moved-by-piston boolean. Use the ordinary two-argument `ChunkAccess.setBlockState`, whose target implementation delegates with `UPDATE_ALL` (3); it does not set `UPDATE_MOVE_BY_PISTON`. Heightmap/light handling remains in Minecraft's setter.
- Snow erosion uses a real Mixin invoker for private `Biome.getTemperature(BlockPos,int)` and passes the generator sea level. It retains the existing 0.25 temperature threshold; the public rain/snow predicates have different thresholds and would change the algorithm. The mixin must be applied in the packaged runtime before this path can work.
- `SpreadingSnowyDirtBlock` became `SpreadingSnowyBlock`; the snow-covered grass/mycelium conversion remains in place.
- Template neighbor shape updates pass the actual level, scheduled-tick access, positions, states, and level random source required by the new method. The update order remains input block then neighbor.
- Compressed template NBT now explicitly supplies `NbtAccounter.unlimitedHeap()`, preserving the previous unbounded reader behavior. Modern optional NBT accessors use the old default-zero/empty semantics. Palette decoding still uses Minecraft's real block registry and keeps upstream's per-entry air fallback.
- Vanilla tree decorator contexts now require `WorldGenLevel`. Narrow the custom decorator contract to that type; its only production caller already supplies a world-generation level. All vanilla decorator application, log/leaf tracking, and placement flags remain present.
- Register the independently implemented `legacy_random_patch` feature requested by the data migration. Its implementation and exact RNG checks are owned by the data migration, not evidence from this document.

## Preserving soil and vegetation rules

The removed `TreeFeature.isDirt` cannot be replaced blindly with the target `BlockTags.DIRT`. The cached **1.20.1** vanilla `data/minecraft/tags/blocks/dirt.json` includes dirt, coarse dirt, rooted dirt, grass block, podzol, mycelium, moss block, mud, and muddy mangrove roots. In the actual **26.2** game, `data/minecraft/tags/block/dirt.json` contains only the first three. The port retains the target dirt tag and explicitly accepts the other six legacy soils so template trees do not stop placing on grass.

The `minecraft:saplings` tag still exists in 26.2 data, although its `BlockTags.SAPLINGS` constant was removed. Keep the same tag ID using an explicit `TagKey`, retaining datapack extensibility rather than substituting an incomplete class or block list.

`LevelHeightAccessor.getMaxY()` is inclusive, unlike the old exclusive `getMaxBuildHeight()`. The overhead-clearance calculation therefore substitutes `getMaxY()` for the complete old `getMaxBuildHeight() - 1` expression; it does not subtract one twice.

## Biomes and modifier caches

Move sky, fog, water-fog, ambient cave sounds, and optional music from removed special-effect builder methods into actual environment attributes. Apply opaque alpha to legacy RGB values as vanilla 26.2's own biome builders do. Water, foliage, and grass colors remain in `BiomeSpecialEffects`. No replacement biome is fabricated: upstream's unused unfinished `coldMarshland()` remains an explicitly inherited missing feature, not completed parity.

Fabric selection now obtains the actual biome holder through `getBiomeHolder()`. Rebuild the target bonemeal feature cache using the same holder tag predicate as vanilla's constructor: `FeatureTags.CAN_SPAWN_FROM_BONE_MEAL`, followed by holder values. The former `Feature.FLOWER` type does not exist on the target. The root migration provides mutable mixin accessors for the new cache fields.

Also invalidate/rebuild the memoized placed-feature set after add/replace operations. This is an approved inherited consistency fix: updating the feature list while keeping an already computed set would leave `hasFeature` queries stale.

## Approved inherited decorator lookup correction

The original `TemplateFeature` used `ResourceKey::registry` to choose a per-biome decorator override. Every biome therefore produced the registry ID `minecraft:worldgen/biome`, not its own ID. Change that lookup to `ResourceKey::identifier`, through the production `decoratorsForBiome` helper. This restores the intended configuration behavior where plains and forest can select distinct overrides while unlisted biomes use the default list.

`tests/terrain/TemplateBiomeDecoratorProbe.java` exercises that production helper with actual target `MappedRegistry`, `ResourceKey`, and `Holder.Reference` objects, without game-class substitutes. It checks two distinct overrides and default fallback. Its initial standalone compilation exposed seven remaining errors in transitive noise/data code; the runtime probe was therefore not executed. The test is not yet recorded as passing.

## Verification evidence and remaining gates

Against the resolved real 26.2 classpath, a scoped Java 25 compile of **82 explicit files** (owned common/Fabric terrain helpers plus the temperature invoker) **passed with exit code 0**, zero errors, and 105 generated class files after the target access widener was refreshed. This used `--release 25 -proc:none -implicit:none`, the actual resolved classpath from `build/qa/target/compile-classpath.txt`, and both production source roots as source paths. It validates the explicit helper files; suppressing implicit compilation means it does not establish that every transitive source body compiles. Local output is under ignored `build/qa/terrain-helper/compile.log`.

An earlier scoped attempt failed on five root-owned `StrataRule` access errors before the access-widener refresh. A subsequent compile of the regression probe with implicit production dependency compilation enabled found seven remaining errors in other noise/data files. These observations are why the separate whole-source compilation gate remains necessary.

Inspect the next whole-source diagnostic after all workers freeze. Still required: custom codec round trips and registry dispatch, target mixin application, real biome modifications and tag loading, palette/NBT loading of bundled trees, template neighbor updates and tick behavior, snow/erosion surfaces, structures, save/reload, client/server packaged-JAR smoke tests, and visual comparisons against pinned references. Neither this source change nor a passing helper probe would establish complete terrain parity.
