# 0008 — Prefetch before biomes, release after feature completion

Date: 2026-11-03 (America/Los_Angeles). Status: source migration implemented; runtime injection and generation verification remain required.

## Context

The old Fabric `MixinChunkStatus` targeted two intermediary-named methods on the old `ChunkStatus` class and mentioned removed chunk-loading failure types. Minecraft 26.2 executes these stages through `net.minecraft.world.level.chunk.status.ChunkStatusTasks` with `WorldGenContext`, `ChunkStep`, `StaticCache2D<GenerationChunkHolder>`, and `ChunkAccess` arguments. The real target signatures and bodies were inspected in the generated 26.2 game sources.

The terrain cache must survive more than density sampling. Erosion and snow features, biome/elevation chance modifiers, and Poisson placement still access cached tile data during decoration.

## Cache semantics

`TileCache.queueAtChunk` translates chunk coordinates into tile coordinates, then starts an asynchronous tile entry if prefetch is enabled. Demand-based provision still works when prefetch is disabled.

Despite the field name `refCount`, `TileCache.Entry.drop()` is **a completed-chunk counter**, not an active-reference decrement. It increments from zero and closes a tile when the number of drops reaches that tile's chunk count. Dropping separately after biomes, noise, surface, and features would count the same chunk repeatedly and could free a shared tile prematurely.

This migration does not change that cache policy, invent arbitrary evictions, or introduce per-stage reference accounting.

## Decision

- Replace the obsolete mixin with `MixinChunkStatusTasks` and update the Fabric mixin configuration. Repair the imported malformed/trailing list commas so the file parses as strict JSON. Set `JAVA_25` after verifying that the resolved Mixin `CompatibilityLevel` enum actually supports it.
- Queue the chunk's tile at the head of `generateBiomes`, before the target method invokes biome generation and before the following noise stage needs terrain fields.
- At the return of `generateFeatures`, replace its returned future with `future.whenComplete(...)`. The callback performs one `dropAtChunk` for that feature task, on successful or exceptional completion.
- Return the wrapped future to Minecraft. Successor stages therefore observe completion after the cleanup callback, rather than racing a detached completion handler.
- Preserve vanilla/non-TerraForged paths and contexts without a cache by leaving their returned future untouched.

The actual 26.2 feature task currently executes decoration synchronously and returns a completed future. Using completion rather than method return expresses the required lifetime correctly if this task becomes asynchronous or another mod returns an incomplete future. `CompletableFuture.whenComplete` also preserves the original exceptional outcome; the implementation does not convert failures into success.

## Limits and remaining checks

The callback applies to a returned future, including an exceptionally completed future. A synchronous throw before the feature method returns cannot be observed by a RETURN injection. Failures in earlier stages likewise do not run the feature-completion hook; the existing timed cache expiry remains responsible for abandoned entries. Retried generation and the cache's cumulative completion accounting need a separate cache-lifetime audit, rather than pretending this API migration establishes active leases.

Required verification includes Mixin target/application checks, actual generation with prefetch both enabled and disabled, feature failures, concurrent neighboring chunks, retry/cancellation behavior, and instrumentation showing that tiles remain readable until all decoration consumers finish. Neither compilation nor source inspection establishes those runtime outcomes.
