# Tile-cache deadlock observed during packaged generation

The original failure is preserved in the isolated Fabric server run `build/qa/server/instances/20261104T073040Z-9f7060/`. Its watchdog report is `crash-reports/crash-2026-11-03_23.32.03-server.txt`. This was an actual packaged custom-generation run: the QA callback had already verified the custom preset registry entry, `RTFRandomState`, and nonnull generator/cache. It was not a vanilla fallback.

## Independent trace review

The report shows a ForkJoin worker with two cache reads nested on the same stack. The outer read reaches `CacheEntry.create` through `LazyCallable.call`; the original `LazyCallable` holds its nonreentrant `StampedLock` write lock while calling `create`. `CacheEntry.create` then waits in `CompletableFuture.get`.

At original report lines 587-600, that waiting worker enters `ForkJoinPool.helpAsyncBlocker` and executes another `CompletableFuture.AsyncSupply`. The nested task performs biome/noise generation and reads the same tile-cache entry. The upper portion of that thread's stack waits for the entry's `StampedLock` through another `LazyCallable.call`. Other generation workers also wait on that cache entry.

The resulting cycle is concrete: the outer invocation owns the lock; its cooperative wait runs a nested reader; the nested reader cannot acquire the lock; it cannot return control to the outer invocation so the outer invocation cannot publish the result or release the lock. `StampedLock` is not reentrant. A larger timeout does not remove this cycle.

The server thread's synchronous `level.getChunk` call in the QA `SERVER_STARTED` callback exposes the blocked work, but the lock cycle is inside the production tile-cache path. Moving the callback to a later tick would alter scheduling and might conceal the race; it would not fix the observed cache reentry.

## Correction and evidence boundary

Root changed the actual production `CacheEntry.call` to wait on its already single-shot `Future` without holding `LazyCallable`'s lock, then publish the shared nonnull value through the volatile field. Multiple readers can await the same task; this does not schedule duplicate terrain generation. The rest of `LazyCallable` retains its synchronization for computations which actually require single creation.

The root-owned `tools/verify_cache.py --prove-baseline-deadlock` regression reportedly reproduces the pinned original reentry timeout and passes 312 corrected-class assertions, including concurrent shared-object reads and failure/close behavior. See that tool's persisted report for exact execution evidence; the trace analysis here independently verifies why the fix addresses this cycle.

A watchdog-disabled run that happened to finish in approximately 2.7 seconds demonstrates the intermittent scheduling nature of the race, not that the original timeout was normal slow generation. Keep the original failed run and follow with repeated runs of the corrected packaged artifact, including normal-watchdog generation and save/reload digest comparison. Passing the deterministic cache regression alone does not establish complete world-generation or modpack compatibility.
