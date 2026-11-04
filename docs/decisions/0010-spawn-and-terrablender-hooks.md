# 0010 — Restore scoped spawn search and audit optional TerraBlender hooks

Date: 2026-11-03 (America/Los_Angeles). Status: source adapters compile; helper tests pass; runtime injection and world outcomes remain separate gates.

## Spawn search

The imported server hook had a four-argument `setInitialSpawn` handler and an entirely commented implementation. The imported `Climate$SpawnFinder` mixin contained no active injections. Leaving these as apparently functional hooks would conceal missing spawn control.

The actual 26.2 server method has a fifth `LevelLoadListener` argument. Its normal-world path asks `Climate.Sampler.findSpawnPosition()` for the initial chunk and then retains Minecraft's safe-position search, height selection, load progress, and bonus chest placement. Debug-world branches do not make this call.

The restored redirect changes only that climate-search call when a TerraForged generator context exists:

- Resolve the selected preset's `SpawnType.getSearchCenter(context)`.
- For a sampler without climate targets, use that center as the initial position.
- Otherwise run the existing vanilla climate search inside `SpawnSearchContext.withCenter`.
- Keep vanilla behavior unchanged when no TerraForged generator context exists.

`SpawnSearchContext` snapshots the center and holds it in a thread-local for the synchronous climate search. Its `finally` restores an enclosing scope or removes the value. This prevents a failed search, nested call, or concurrent world from retaining another search's configuration.

The real `Climate$SpawnFinder` still exists in 26.2. The effective mixin changes its constructor's initial `getSpawnPositionAndFitness` coordinates to the scoped center. Its two coordinate-square arguments in the fitness distance bias are made relative to that center. The third square, the `2048` climate-fitness weight, is untouched. Vanilla radial refinement and climate fitness therefore remain in place instead of selecting a center and immediately biasing the search back toward world origin.

All three spawn-finder injection points and the server redirect have hard requirements. An incompatible target fails visibly instead of silently discarding the setting.

### Feature distinction

`CONTINENT_CENTER` now connects the existing continent-center calculation to the initial spawn search. `WORLD_ORIGIN` uses an origin-centered search. **`ISLANDS` still returns world origin in the inherited `SpawnType` implementation and has no island-specific preference.** This is an explicit remaining parity item; this migration does not claim to implement island spawning or prove actual continent-center spawn outcomes.

### Executed helper checks

[spawn-scope-initial.json](../validation/spawn-scope-initial.json) records **36 passing assertions** against actual Minecraft 26.2 classes, compile exit 0 and process exit 0. Checks cover vanilla/unscoped distance inputs, immutable center snapshots, successful return values, nested and outer exception cleanup, signed long subtraction at integer extremes, four simultaneous thread scopes, and cleanup before worker reuse.

Reproduce after resolving the target classpath:

```powershell
python tools/verify_spawn.py --java-home $env:JAVA_HOME
```

These tests exercise the actual new scope and distance helpers with real `BlockPos` classes. They do not apply Mixins or generate a world.

## Optional TerraBlender integration

Inspected the actual resolved `TerraBlender-fabric:26.2-26.2.0.0.2` class bytecode, including its added `IExtendedParameterList` API and `findValuePositional(TargetPoint,int,int,int)` implementation. The current game sampler and noise-chunk methods were checked against 26.2 classes/source.

The four compatibility mixins remain listed once in the common config and are gated by `MixinPlugin.shouldApplyMixin` through `TBCompat.isEnabled()`. They must not apply when TerraBlender is absent. No optional dependency is reclassified as verified runtime support.

Changes:

- `TBClimateSampler` now declares only uniqueness methods that its mixin actually implements. Remove the stray, unimplemented spawn-center setter; initial spawn is handled independently of optional TerraBlender.
- Climate samplers and target points implement their complete compatibility interfaces directly, with uniquely named backing fields. Do not leave a partially implemented soft interface that can produce `AbstractMethodError`.
- Sample uniqueness using a newly constructed `SinglePointContext` at the exact block coordinates corresponding to the sampler's quart-coordinate arguments. This matches the real target implementation and removes brittle captured-local assumptions.
- The noise-chunk mixin retains the global-to-cached uniqueness transfer, uses uniquely named state, and shadows the actual protected `wrap(DensityFunction)` signature.
- The parameter-list hook runs below TerraBlender's default priority, reads its actual tree count after initialization, and redirects the verified positional uniqueness call with the complete parent-method argument signature. The redirect now requires its target instead of silently allowing zero matches.
- Valid finite uniqueness is bounded to its intended `[0,1]` domain and mapped to a valid region index. Missing/non-finite custom uniqueness uses TerraBlender's real `IExtendedParameterList.getUniqueness` implementation. The old nonexistent-base-class shadow and unconditional fallback exception are removed.

The owned spawn/TerraBlender production files compiled against the real dependency classpath and the coordinator's current full compiled classes with exit 0. This checks Java APIs only; Mixin application, optional-mod bootstrap order, actual biome distributions, surface-rule composition, and other biome-mod combinations still require dedicated runtime evidence.
