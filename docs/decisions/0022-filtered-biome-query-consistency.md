# 0022: Resolve biome queries from completed terrain on a cold cache

Date: 2026-11-04, America/Los_Angeles.

The dev20 stored-biome corpus exposed a cache-state dependency. Its first approximate warm-beach location resolved to savanna scrub after terrain filtering. A nearby location qualified against the completed tile generated and stored warm beach correctly, but after process restart the saved chunk still contained warm beach while the source returned lukewarm ocean. Both loaders reproduced the failure. Retargeting the survey alone was insufficient.

`WorldLookup.applyCell` previously preferred an existing filtered tile but otherwise evaluated an approximate unfiltered heightmap. Global `CellSampler` retained whichever result was first sampled. `ClassicBiomeSource` also resolved the parent before obtaining its own terrain cell, so even warming that cell afterward could leave the parent based on the old approximation.

The first dev21 correction made every global sample request completed terrain. It passed the rejected reload, but broader fresh-world tests exposed excessive work during initialization. An actual thread dump showed concurrent stronghold ring refinements calling the wrapper's biome search and generating remote filtered terrain. The dev21 startup failures are retained; that candidate is not accepted.

Dev22 scopes completed-terrain sampling to direct classic-wrapper biome resolution. The wrapper completes its cell first, then resolves the complete delegate inside a balanced accurate-sampling scope. An independent one-entry weak-owner cache stores those accurate global samples; ordinary spawn/climate heuristics retain the fast default sampler. Nested scopes and exception cleanup are checked with real lookups. The per-NoiseChunk structure-column cache retains its approximate path and bounded 16-entry table. No heightmap equations change.

Parent climate and regional selection therefore use the same completed terrain. Other namespaces retain the complete delegate's holders; below-zero queries use its accurate cave result without classic replacement. The delegate codec and independent selection toggle remain intact.

The native concentric-ring refinement calls the unwrapped parent source for its coarse preferred-biome search. That narrowly targeted hook avoids generating hundreds of remote regions for stronghold planning. It retains the native search, predicate, random source and sampler; actual structure generation keeps its own checks. Ordinary direct biome queries and player biome searches are not redirected. The exact 26.2 synthetic method and call descriptor were verified against the publisher game class, and the strict hook loads on both loaders.

This can make the first query into a new terrain tile more expensive. Normal biome generation already requests those tiles; arbitrary biome searches must now pay their actual terrain cost. The tile cache is bounded to 256 entries by `StampedBoundLongMap`. Final performance and optimization tests must use this corrected exact artifact rather than extrapolate from the old fast approximate-query behavior.

The old 51,271-point sparse source surveys are retained as historical approximate-query evidence. The final corpus instead checks all seventeen natural variants through actual FULL chunks and source agreement on both initial generation and cold process reload. This is stronger stored-world evidence without requiring thousands of widely separated full terrain tiles merely to repeat the old survey.

Both exact dev22 cold reloads pass the rejected warm-beach case with unchanged saved columns. Fresh worlds on both loaders also pass 801 cache/raw-bit/scope/lifecycle checks and finish initialization normally. The complete two-loader corpus, clients and performance checks remain separate acceptance gates.
