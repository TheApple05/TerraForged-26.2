# Procedural densities inside the Horizon graph

The qualified kernel can now bind real procedural cell values into Horizon's
compiled density graph. The `terraforged_native` bundle stages one
`terraforged-kernel` crate through the mixin tool's supported `[[crate]]` API and
one support module inside `horizon-core::density`. It does not activate a world
or replace the earlier finite prepared-world experiment.

The crate is assembled from the existing native core and the bounded preset and
noise-registry adapters. Only their import roots change; the algorithm sources
and two source-owned numerical lookup tables are preserved and hashed. QA
binaries, Java expected outputs and fixtures are excluded from the runtime
crate. Its sole registry dependency is the already-locked `serde_json` version.
No Minecraft resources are read or downloaded during source staging.

Bindings cover all ten cell fields, resolved two-dimensional noise graphs, and
the original clamp wrapper. Cell and noise functions declare X/Z dependencies.
Bindings use conservative infinite optimizer intervals because inherited legacy
min/max metadata is not a sound bound for the generated values.

The clamp child compiles in its own cache-ID namespace. It receives immutable
world fields and the caller's cache, sparse and plane-enabled modes through a
bounded pool of at most eight independent contexts. Cached results and plane
contents cannot cross compiler namespaces. Locks are released during evaluation;
errors propagate, and completed contexts release their world-field references.
No global world owners or background polling threads are introduced.

Two fresh Java server JVMs captured identical results from the exact dev23
production JAR. The oracle verifies its resolved noise bindings and production
class origins. It invokes genuine mapped `CellSampler` functions under accurate
sampling over actual filtered tiles, the original clamp/noise wrappers, and an
addition graph. The reference has 24,768 records: 10,368 scalar and 14,400 bulk.
It covers six long world seeds, including narrowing endpoints and a high-bit
collision, negative/positive tile boundaries, and three vertical positions.

Both optimized and debug Horizon graph runs matched every observed output bit
at the host's f32 interface. That qualification deliberately narrows the Java
result to f32; it does not prove equivalence of full Java f64 chunk evaluation.
The fixed oracle working set uses the supported bounded capacity of 32 tiles.
The production sampler default remains eight, with its earlier eviction tests.
Parallel graph evaluation and world-owner release guards also pass.

Focused tests verify parent/child cache-ID collisions, changing world-field
bindings, short-buffer rejection before writes, output tails, parallel readers,
error recovery, context-mode changes, and releasing fields after failures.
The first graph report used isolated uncached children. The follow-up enables
child caching explicitly and repeats the same numerical qualification.

Compiling the kernel as a host dependency also exposed 23 warnings under
Horizon's strict Clippy command. The bundle now inherits the host workspace's
existing argument-count/style policy. Local style fixes, small `Default` and
`is_empty` APIs, and scoped explanations for source constants/fallible graph
factory names resolve the other warnings. The `1.5708` cosine offset is retained
because replacing it with the standard pi/2 constant would change its bits.
The assembled host library passes `cargo clippy ... -- -D warnings`.

The first Java attempt rejected equivalent Gson float-valued and parsed decimal
primitives during configuration comparison. Comparing their serialized JSON
forms fixed that check; the failed reports remain preserved. An exploratory
native run with an eight-tile cache was stopped because repeated sweeps of the
16-tile oracle working set caused needless regeneration. It is not performance
evidence.

The initial documentation draft and a first-pass code review used the authorized
DeepSeek worker. Codex reviewed the output, fixed the cache-mode finding and
verified it locally. Suggested sharing of plane caches and concerns about locks
held during sampling were rejected: plane contents belong to a compiler
namespace, and the mutex guard is already released before the child runs.

Reproduce with `horizon/tools/stage_native_host.py --horizon <local-host-source>
--mixins <local-mixin-tool> --output build/qa/horizon/<fresh-name>`, then
`horizon/tools/verify_host_density.py --workspace <output>/staged --report
<new-report.json>`. Both tools refuse replacement of their evidence outputs.
The exact pinned host source object must already exist locally.

Remaining work includes resource decoding and world selection, the complete
26.2-to-26.3 router mapping, biome/surface/cave/feature integration, real chunks,
client rendering, save/reload, mod combinations and distributable packaging.
