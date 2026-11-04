# Regression and acceptance protocol

**All checks below are planned unless a result record explicitly says they ran.** Passing an isolated noise test does not verify Minecraft chunk generation.

## Evidence rules

- Record source commit, Minecraft/loader/dependency versions, Java version, seed, settings, workload, environment, command, exit code, and output checksums.
- Preserve raw float bits for unchanged mathematical algorithms. Any tolerance or expected difference requires a written reason.
- Compare algorithms separately from vanilla biome labels. Registry and vanilla generation changes across versions must be classified rather than silently normalized away.
- Screenshots must identify whether they show reference Minecraft, target Minecraft, or a diagnostic field visualization.
- Failed or incomplete checks remain visible. A changed fixture requires review; never regenerate expected values just to make a test pass.

## Required scenarios

1. **Algorithm reference:** sample pinned upstream noise/math kernels at fixed seeds and positive, negative, fractional, boundary, and large coordinates. Compare raw bits to checked-in upstream-derived fixtures and report source hashes. Separate-JVM runs must agree.
2. **Sampling order and concurrency:** evaluate the same coordinate set serially, shuffled, and across workers. Compare results keyed by coordinate. Stress caches and terrain tiles separately once migrated.
3. **Negative chunk boundaries:** compare the same world-coordinate samples requested through neighboring chunks/tiles at `-17`, `-16`, `-1`, `0`, `15`, and `16`, in both axes. Adjacent terrain heights need not be equal; overlapping sample coordinates must agree. Verify full block seams in-game later.
4. **Terrain fields:** capture height, continent, climate, river, erosion, and terrain-type fields for fixed regions. Compare with the pinned reference and explain every changed field.
5. **Minecraft integration:** generate fixed-seed target worlds; sample real blocks, biomes, heights, cave/water behavior, surfaces, and structures. Record generation errors and distinguish vanilla changes from mod defects.
6. **Preset round-trip:** serialize and reload non-default presets, including bounds and invalid/legacy values. Verify every setting is retained or migrated with an explicit diagnostic, and affects the intended generation path.
7. **Client/server parity:** generate equivalent regions in an integrated server and dedicated server with identical seed/config. Compare normalized outputs and test that client-only classes do not leak into the server path.
8. **Packaged artifact:** install the exact final JAR and declared dependencies in isolated non-development installations. Record SHA-256, runtime provenance, launch/generation logs, and process exit. Test save/reload and subsequent chunk generation.
9. **Visual review:** capture reference and target screenshots with seed, config, coordinates, camera direction, time/weather, and versions. Cover continents/coasts, plains, mountains, rivers/lakes, biome transitions, and extreme presets. Record user feedback separately from technical checks.
10. **Performance:** benchmark a fixed pregeneration region after warm-up on the same hardware. Record wall time, chunk rate, memory, GC, threads, settings, and variance. Set budgets from measured reference runs; isolated microbenchmarks do not prove in-game speed.

## Current dev23 results

The exact **0.1.0-dev.23+26.2** packages pass the [functional acceptance checker](validation/dev23-functional-acceptance.json), binding 67 evidence files and 30 archived native images. [The readiness record](validation/dev23-readiness.md) provides the loader-specific links.

All seventeen restored variants generate naturally in FULL chunks, agree between stored holders and source queries, and retain their full-height sampled columns after cold JVM reload. Bryce and stone-forest tests observe real added columns, authentic terrace math, bounded heights and actual caps. The regional selector and nine geological palettes remain documented modern adaptations.

Both loaders pass real preset/editor/preview/Done/Create New World/custom-render/save/subsequent-vanilla flows. Actual shader clients retain every BOP overworld holder alongside the classics, use the pinned Complementary shader and write DH data. Near terrain is rendered; brief captures have incomplete distant LOD edges, so far-distance quality and FPS are not accepted.

Dedicated optimization, community ModernFix and combined biome/structure cases export and consume their own matching packs, generate, save and cold-reload. Real client joins to the combined content stacks synchronize all 17 definitions and receive terrain on both loaders. WSL Ubuntu base generation and reload also pass using the same JARs and hash-verified pack ZIPs. Public authentication, multiplayer load and Linux client rendering are separate untested scopes.

Exact cache checks pass 3,511 lifecycle assertions per package with no retained owners/queued polls. Real-world raw Cell comparisons retain 24 fields while reducing repeated structure-column heightmap calls from 384 to four. Genuine native structure-column inputs also match dev18 in complete sampled 384-block columns.

The predeclared [controlled performance protocol](validation/performance-protocol.md) requires three matched fresh-world trials for each generator/loader in base and optimization stacks, clean native saves/exits, normal watchdog, cold reload digests, wall-time/heap/GC/process memory and all trial values. The quiet-host session passes all 24 fresh trials and eight cold reloads with no compiler interference or budget violations. [Accepted timings](validation/dev23-performance.md) and [complete scoped acceptance](validation/dev23-acceptance.json) retain every trial and its raw report hashes. [The verified two-loader prerelease is published](https://github.com/TheApple05/TerraForged-26.2/releases/tag/v0.1.0-dev.23%2B26.2).

Scoped helper/reference checks and historical failures remain in `docs/validation/`. [Dev11 acceptance](validation/dev11-acceptance.json) describes the previous experimental delivery. [Decisions 0016-0023](decisions/0023-neoforge-original-biome-export.md) and the [worklog](validation/modpack-readiness-worklog.md) record the restoration counterexamples and fixes rather than rewriting earlier failures as passes.

**Remaining limits:** full original Engine/classic appearance parity; every composed noise/preset/control/import extreme; island-preference spawning; long exploration and full post-feature block determinism; old-world and cross-loader migration; public authentication/player load; arbitrary pack/shader combinations; finished far-LOD visuals; and user visual acceptance.

Native Horizon **26.3** source and validation have moved to [TerraForged-Horizon](https://github.com/TheApple05/TerraForged-Horizon). This protocol covers Java 26.2 Fabric/NeoForge; native acceptance is documented in that separate repository.
