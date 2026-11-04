# Packaged runtime validation

These results use isolated disposable Minecraft Java 26.2 installations, not a development launch. Dates in the project log use America/Los_Angeles; machine records retain UTC timestamps. No existing user worlds, launcher profiles, accounts, or global Java settings are changed.

## Fabric candidate

The corrected candidate is `terraforged-revival-0.1.0-dev.2+26.2.jar`, SHA-256 `94471bb542a308982bfb29c7340ed14dbf3494922cfd9f935c5f32b0485e80bd`. It uses Java 25, Fabric Loader 0.20.1, and Fabric API 0.161.0+26.2. The separate QA companion is not included in the production JAR. This candidate predates the shared dual-loader service migration now in progress; later packages require fresh runtime checks.

- [Initial production boot](server-2026-11-03-initial.json): the earlier dev.1 package loads, saves a vanilla world, and exits cleanly. This establishes loading only.
- [Real preset export](server-2026-11-03-export.json): actual server registries export 1,934 files, including 1,933 JSON files and the custom preset. Three actual biome-decorator helper assertions pass.
- [Corrected fresh run 1](server-2026-11-03-dev2-generation-1.json) and [fresh run 2](server-2026-11-03-dev2-generation-2.json): custom preset registry entry, `RTFRandomState`, `GeneratorContext`, and `TileCache` are present; six full chunks and 54 column/terrain samples are recorded. Seed 8675309. Workload times were 2,780 and 2,668 ms, excluding installation/startup. Both use the normal 60,000 ms watchdog, save with flush, and exit 0 without forced termination. These small workloads are not performance benchmarks.
- [Save/reload](server-2026-11-03-dev2-reload.json): the first world reloads and matches its saved sample digest exactly, then saves and exits 0.

The requested chunks are `(0,0)`, `(-1,-1)`, `(7,0)`, `(8,-1)`, `(-8,7)`, and `(-9,8)`. Both fresh runs agree on all sampled terrain cell fields and biomes. Their aggregate sample digests differ at one column, `(15,0)`: grass at Y75 in the first run versus birch leaves at Y84 in the second. The first run and its reload agree (`be2e05a712f87baebbd82265961f3682d47c57bef40172276fce26af3b9fd2d2`); the second fresh run has digest `5fab505bc818ab96a66e2a65d644e5671bcc9029f1922af372df694d8fc00db1`. Neighbor decoration timing is being investigated. This is not recorded as a passing fresh-world determinism check.

## Failures retained and corrected

The [initial custom generation attempt](server-2026-11-03-generation-initial.json) hit the watchdog. An independent [thread-trace analysis](tile-cache-deadlock.md) identified a reentrant ForkJoin wait while `LazyCallable` held a non-reentrant lock. `CacheEntry` now waits on its already single-shot future without that lock. The [312-assertion regression](cache-reentry-2026-11-03.json) reproduces the original deadlock and passes for the correction. The successful normal-watchdog runs above use the corrected artifact. A successful watchdog-disabled diagnostic run remains recorded and is not used to excuse the failure.

The migrated slider initially bypassed its superclass event handling. [Before](slider-before-fix.json) and [after](slider-reviewed.json) reports preserve the actual event regression; 29 assertions now cover click, drag, keyboard, callbacks, labels, and bounds.

## Client and NeoForge gates

The Fabric production client reached its native window with the actual package. Desktop capture then timed out twice, so that owned QA process was stopped deliberately; [the report](client-2026-11-03-dev2.json) records an aborted review rather than a game crash or a passing visual test. A separate companion now uses the actual game screenshot API to inspect real screens and terrain. See [capture limitation](client-capture-limitation.json).

NeoForge 26.2 is requested alongside Fabric. Its loader bindings and independent build are in progress. A Fabric result does not establish NeoForge support. No published release, third-party biome-mod compatibility, full classic parity, multiplayer validation, or user visual acceptance is claimed here.
