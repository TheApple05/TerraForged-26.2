# Bounded procedural column sampler

KernelSampler now supplies complete filtered cells at arbitrary block coordinates.
It constructs real tiles on demand; it never reads expected cells or prepared
chunk snapshots. Negative right shifts, masks and core/halo addressing retain the
source's behavior. Default options use factor3, eight cached tiles, four concurrent
builds, and the source's lifetime-derived one/two-chunk border. Explicit supported
limits are factors/borders 0..4, capacity 1..32 and concurrent builds 1..8.

Both debug/release match 196,608 core columns / 4,718,592 fields from the existing
independent Java kernel corpus across six world seeds. The comparison checks all
24 fields, including enum names and raw finite bits. It also observes eviction and
full-tile reload, parallel tiles, retained Arc validity after clear/release, bounded
cache counts, and real generation at additional coordinates outside that corpus.
The extra coordinates are native generation/cache checks; Java parity is claimed
only for the independent observed columns. Earlier full backing/halo and filter
qualifications remain in tile-filters.md.

The tile cache deduplicates same-key requests and permits different tiles to build
concurrently within a fixed slot limit. Only complete successful tiles publish.
Failure/unwind removes unfinished work and wakes waiting callers. LRU eviction
drops the owner's Arc; caller-retained values remain valid. Clearing advances a
generation: older work may finish for existing callers but cannot repopulate that
cleared generation. No background threads or process-wide cache owners exist.
Five focused lifecycle tests pass in debug and release. Their controlled builders
isolate ownership/publication behavior; the real sampler's Java comparison is the
terrain correctness gate. Quiescent release clears tile, river and brush caches;
active generation can still recreate its needed subordinate river/brush data.

An optimized Windows benchmark uses the exact hashed native executable, seed
8675309, the frozen default preset and registry, four 128×128 core tiles with
160×160 backings at local/distant positions, and three fresh trials per mode.
Median generation/filter time for four tiles is 0.519 seconds serial and 0.198
seconds with four callers. The warm pass reads all 65,536 core columns in about
3.3 ms, including a checksum of all 24 fields. All serial/parallel tile and warm
checksums agree. Process peak working set is 34,635,776 bytes (about33MiB).
The per-tile value in the JSON divides batch wall time by four; it is not an
individual parallel tile's latency. Parallel timing includes thread startup.
These measure the isolated optimized kernel, not density evaluation, Minecraft
chunks, a game client, save behavior, mod combinations or FPS. Host/compiler,
input/source/executable hashes and all trials are in
`horizon-native-kernel-benchmark-v1.json`; no modpack readiness is inferred.

A bounded external first-pass review request, DeepSeek job
`1fcf3b36ef4a49a898936b9f313da5d9`, hit its output limit. No draft was accepted and
the request was not repeated. Its reserved estimate was USD0.0051282, separate
from Codex usage. Codex performed the concurrency review, isolated the cache from
generation for focused lifecycle tests, and ran all actual sampler comparisons.

Run `verify_tile_cache_lifecycle.py --report <new-report.json>` and
`verify_kernel_terrain.py --stages sampled --report <new-report.json>`.
Both run in Windows/Linux CI. `benchmark_kernel_sampler.py` reproduces the local
kernel benchmark without replacing prior evidence. Native density, registered
biomes, surfaces/caves/decoration/structures, save reopen, real rendering, launcher
packaging and mod combinations remain required for a Horizon release.
