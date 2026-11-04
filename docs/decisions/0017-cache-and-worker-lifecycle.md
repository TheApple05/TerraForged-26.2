# 0017: Bound worker configuration and release generator-owned caches

Date: 2026-11-04. Applies to shared Fabric/NeoForge code.

## Reproduced problems

`PerformanceConfig.read` ignored its file and always returned defaults, while the
declared `threadCount` never affected the actual terrain executor. Its old maximum
tile exponent of eight also permitted very large quadratic cell allocations.

The old `CacheManager` kept every registered cache in a static strong list.
`Cache.close()` canceled its polling task but retained map entries and never
unregistered the owner. Preview regeneration repeatedly invoked that global
close operation. River caches constructed `Cache` directly and their repeating
scheduled callbacks retained them outside the manager as well. No world-stop
path released those caches.

The [regression record](../validation/cache-lifecycle-2026-11-04.json) compiles the
exact selected upstream lifecycle classes and runs the same old-API probe against
upstream and candidate. Upstream retains one owner and one map entry after close;
the candidate retains neither. The separate candidate probe passes 3,506
assertions, including 500 repeated cycles, 400 concurrent create/clear cycles,
scope isolation, constructor failure cleanup and canceled scheduler queue removal.
The later [exact packaged factory-race check](../validation/cache-lifecycle-dev16-factory-race.json)
passes 3,511 assertions and deliberately blocks a factory while closing its cache;
closure waits for insertion, clears ownership, and preserves the returned value.

## Decision

- Read a documented UTF-8 properties file, create defaults only when absent, and
  report malformed/unknown/out-of-range input without overwriting it. Existing
  call sites log the error and use safe defaults. Supported bounds are tile
  exponent 1..5, batches 1..20, and workers 1..twice the JVM-visible CPU count,
  capped at 256. A normal restart after editing is the supported workflow.
- Resize the shared terrain `ThreadPoolExecutor` safely rather than replacing an
  executor that owns queued work. Apply the requested worker count in actual
  world initialization and preview generation, including the prefetch decision.
- Register every `Cache` constructor exactly once, including direct river-cache
  construction. Closing is idempotent: cancel the poll, remove the scheduled task
  immediately, clear storage, and unregister the strong owner.
- Clear storage without invoking value-close callbacks. A reader or in-flight
  future may still own a Tile and its pooled arrays. Cache shutdown must not reset
  those arrays underneath it. Native per-chunk completion/drop remains responsible
  for normal Tile release; discarded ownership can otherwise be collected.
- Capture caches created synchronously while constructing a `GeneratorContext`.
  That context closes its own captured river caches and its own Tile cache.
  A failed construction releases already-created caches. Other contexts remain
  usable. The manager snapshot/close operation never holds its owner-list lock
  while waiting for cache-map locks or invoking callbacks.
- Preview replacement/close releases its own previous context and Tile. It no
  longer globally cancels unrelated world caches. At native `stopServer` tail,
  after chunk draining and level closure, release each stopped world's context.
  Named terrain/cache-maintenance executors retain the existing application-exit
  shutdown hook.

## Validation boundary

The deterministic lifecycle probe establishes ownership/storage/scheduler
cleanup and preservation of live values under its tested concurrency. It does
not prove every game shutdown path or arbitrary mod interaction. Packaged
production client tests must additionally observe bounded owner counts across
repeated previews and release after custom-world shutdown. Controlled chunk
generation, save/reload, memory/GC measurements and matching optimization stacks
remain independent runtime gates described in the
[performance protocol](../validation/performance-protocol.md).
