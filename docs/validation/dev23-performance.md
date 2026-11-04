# Dev23 controlled generation results

Both predeclared matrices pass: 24 fresh generation processes and eight cold reload processes, with three matched trials per generator/loader/stack. The quiet-host controller observed no compiler interference. Every requested new-region phase meets the 30-second budget, every process stays below the 6 GiB resident-memory budget, the normal watchdog remains enabled, and saves/exits/reload digests pass. No trial or outlier was discarded.

Host: Intel Core i9-11900K, Windows, 64 GiB physical RAM. Java 25 uses G1, a 4 GiB maximum heap and eight JVM-visible processors. Seed 8675309, the exact dev23 JARs, their own qualified stack exports and the separate test companions are unchanged. The [protocol](performance-protocol.md) specifies coordinates/order and the matched vanilla control, which keeps the mod installed with its custom preset/context absent.

Base stack medians for the first/second 64-request new regions:

- Fabric custom: 6.043 / 5.916 seconds; matched vanilla: 2.975 / 2.478 seconds.
- NeoForge custom: 5.814 / 5.662 seconds; matched vanilla: 2.953 / 2.411 seconds.

C2ME/Lithium/FerriteCore stack medians for the same requested workloads:

- Fabric custom: 4.179 / 4.325 seconds; matched vanilla: 2.303 / 1.731 seconds.
- NeoForge custom: 4.156 / 4.448 seconds; matched vanilla: 2.450 / 1.854 seconds.

Custom-generation process peaks range from about 1.30 to 1.34 GiB across both stacks. These are whole-process peak resident measurements, not the heap limit or a client/modpack memory requirement. Heap-pool peaks, per-phase GC collection/time, request median/p95, all three wall times and min/median/max are retained in each raw report. The resident reread phase is submillisecond to approximately 1 millisecond and is a cache-access measurement, not new terrain throughput.

The optimized stack measured lower new-region times on this host, while TerraForged remained more expensive than the matched vanilla generator. The matrix does not attribute the change to an individual optimization mod. Stack order and operating-system caches were not randomized or cleared. These are local requested-chunk workloads with native neighbor/tile work included; they are not FPS, total internal chunk counts, public-server/player capacity, minimum hardware or unbounded-exploration promises.

Raw immutable evidence: [base matrix](benchmark-dev23-base.json), [optimization matrix](benchmark-dev23-optimized.json), [host session](benchmark-dev23-host-session.json), and the [exact host driver](dev23-host-session-driver.py). The complete [acceptance checker](dev23-acceptance.json) binds every timing trial and first-trial cold reload to its exact artifact/report hashes.
