# Real procedural terrain pipeline

The explicit native factory now builds Horizon's actual `Pipeline` using the
scoped TerraForged router. Noise filling, aquifers, material surfaces, carvers
and feature decoration execute in Rust. The runtime reads no snapshots and
starts no Java process. This is a developer entry point; it has no automatic
world-choice hook or player package yet.

The wrapper forwards all 24 methods of the pinned MIT `SeededPipeline` trait.
`generate_pipeline_forwarding.py` generates the forwarding from that exact
source object and checks method names/order and the trait hash. The cache-release
method also clears the owned native tile, river and brush caches. Host cache
census fields retain their original meanings; they do not include native tile
bytes.

`horizon-native-procedural-chunks-v1.json` binds the tested optimized executable
to two isolated fresh processes with seed 8675309. The recorded terrain chunks
are `[0,0]`, `[-1,-1]`, `[64,-64]` and `[1024,-2048]`. They contain real registered
block states, nonempty ground/sky, stored biomes and in-range heightmaps. The
origin differs from the same-seed vanilla generator and regenerates identically
after clearing caches. A real 3x3 neighborhood then decorates its center:
57 placed-feature roots executed, none unsupported or failed, and no skipped
roots. Block/biome outputs and counters agree across both processes after
excluding measured elapsed time.

The tests explicitly check empty native tile caches after release and that the
weak kernel owner expires after dropping the generator and workers. The wrapper
also invokes the host's own cache-release method; this run records its census
before final cleanup. The player's resource hashes remain unchanged. No game
files are downloaded. The world library passes Clippy with warnings denied.

Stage with `stage_native_host.py --world`, then run `verify_native_chunks.py`
against the existing prepared resource root. Evidence paths must be new. The
world matrix in CI checks compilation/linting and the pinned forwarding API;
resource-dependent real terrain checks run locally. Timings in this first
mixed workflow are not controlled performance or FPS comparisons.

While adding this mode, compilation caught an assembler edit that copied the
world support module into the kernel's router slot. The tested v4 stage fixes
the adapter-copy loop; earlier ignored developer stages remain unqualified.
Initial forwarding generation also needed balanced array return types and
absolute paths for the child module. The documentation draft used DeepSeek;
Codex reviewed it against the actual reports and corrected its skipped-root
and cache-verification wording.

The native 17 classic biome variants/features and source-specific carver
configuration remain unfinished. Structures, spawn behavior, save/reload,
client rendering, launcher installation, controlled performance and native mod
combinations still need acceptance. This is not a ready Horizon modpack build.
