# Controlled generation and compatibility protocol

This protocol was set before the new classic-biome candidate's measurements.
Results belong to exact packaged artifacts, not development runs. A successful
test does not certify arbitrary packs, hardware, shaders or player counts.

## Controlled workload

- Minecraft Java 26.2, matching Fabric or NeoForge production installation,
  explicit JDK 25, G1 GC, `-Xms1G -Xmx4G -XX:ActiveProcessorCount=8`.
- Fixed seed 8675309, default modern dimension height, unchanged candidate and
  separate test-only QA companion. Each trial uses a new disposable world/JVM.
- Three trials per generator and loader, alternating custom and vanilla order.
  The control retains TerraForged and the exact same extra mods, with its custom
  preset and generator context absent. This measures a generator baseline; it
  does not establish the cost of installing versus removing the mod.
- First region: 64 FULL chunk requests at chunk origin (1024,1024), x-major order.
  Second region: 64 FULL chunk requests at (2048,-2048), same order. These are far
  from initial spawn. Native neighbor generation and initial terrain-tile work
  are included in wall time. Requested chunks per second is not a count of every
  internal chunk or terrain cell generated.
- Between the regions, reread the first 64 resident chunks. This measures cached
  access, not newly generated terrain. A separate JVM reload checks the first
  trial of each generator/loader against its own 64 full-height column hashes,
  heights and biome holders.
- Report all trials, phase wall times, individual request median/p95, observed
  biome IDs, heap-pool peaks, GC collections/time and native process peak resident
  memory. Heap-pool peaks need not occur simultaneously and are not summed.
  Windows `PeakWorkingSetSize` or Linux `/proc` `VmHWM` covers the whole Java
  process including startup and shutdown. No forced GC or discarded outliers.
- Run native Minecraft benchmarks sequentially, with no competing task-owned
  client/server or compiler workload. Record host CPU, RAM, OS and Java arguments.
  Operating-system caches are not cleared, so a first-new-region result is not
  a claim of cold disk/cache performance.

The predeclared local budget is completion with the normal 60-second watchdog,
clean saves and exit 0, no OOM/deadlock, each 64-request new-region phase below
30 seconds, and native process peak resident memory at most 6 GiB with the 4 GiB
heap. These are local regression/packaging gates, not minimum hardware promises.
Failures remain in their raw reports and are excluded from successful timing
summaries until their cause is resolved and a new, separately named trial runs.

## Pinned compatibility stacks

The [artifact inventory](compatibility-26.2-artifacts.json) records immutable
publisher version IDs, exact target/loader declarations, dependencies and every
published file hash. Third-party binaries remain in ignored QA storage and are
not redistributed with TerraForged.

1. Base: candidate plus matching loader/API and the separate QA companion.
2. Optimization: base plus C2ME, Lithium and FerriteCore, matched per loader.
3. Content/structures: optimization plus the separately pinned BOP/TerraBlender/
   GlitchCore stack and Repurposed Structures (plus its Fabric MidnightLib
   dependency). Export a fresh same-stack preset and verify its use.
4. Client rendering: the pinned Sodium, Iris and Distant Horizons builds plus an
   explicitly activated pinned Complementary Reimagined shader pack. Shader
   activation, actual terrain rendering and LOD output need positive runtime
   assertions; merely loading their mods is insufficient.
5. ModernFix: upstream has no 26.2 publisher build in the recorded query. The
   explicitly labeled ModernFix-mVUS community port is a separate optional
   compatibility test, never represented as the upstream release.

Structure probes use native generation and placement, record changed block
hashes, and distinguish explicit placement (which bypasses biome eligibility)
from bounded native locate followed by a valid naturally generated FULL-chunk
start. Registry presence alone is not structure-placement evidence.

## Reproduction

`tools/qa/resolve_compatibility_stack.py --download` verifies the pinned publisher
files. Build separate loader QA companions against the candidate JARs, then use
`tools/qa/benchmark_matrix.py --help` for the sequential matrix. The driver
refuses to overwrite an existing individual trial report, logs failed runs,
checks native shutdown/provenance before using timings, and retains exact hashes.
Use a new label for any justified repeat after a fix.

Performance configuration is now a validated UTF-8 Java properties file at
`config/reterraforged/performance_internal.conf`. `tileSize` accepts 1..5,
`batchCount` 1..20, and `threadCount` 1..twice the JVM-visible CPU count (capped at
256). Missing files receive documented defaults; malformed files are retained
and logged while safe defaults are used. Restart after editing. The shared
terrain executor honors the configured worker count in worlds and preview.
The runtime probe verifies file/default/error behavior and three real concurrent
workers, then restores the effective pool size before measured phases.
