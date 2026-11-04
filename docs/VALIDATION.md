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

## Current results

- **Passed:** unchanged math/hash utility slice, 2,167 reference records across four independent Java 25 processes, shuffled/parallel comparison, and temporary mutation detection. See [core regression evidence](validation/core-regression.md) and [integrator verification](validation/core-reviewed.json).
- **Passed, scoped components:** 235 density adapter assertions against actual 26.2 classes; 216 legacy vegetation cases with 6,948 nested placements matching 1.20.1 RNG behavior; nine dimension configurations; 36 spawn-scope assertions; 29 slider event assertions; and 312 cache assertions including reproduction of the pinned original deadlock. Individual reports record their narrower scope.
- **Passed, packaged Fabric server:** real preset export, active custom generator checks, six full chunks / 54 sampled columns in each of two fresh runs, and save/reload matching the first run. Normal watchdog enabled, clean process exit, actual packaged JAR origin and hash verified. See [runtime results](validation/runtime-summary.md).
- **Under investigation:** the two fresh runs differ at one sampled tree-canopy column at a chunk boundary. Raw terrain cell fields match at all 54 samples; whole-world output determinism is not yet established.
- **Unverified:** full noise composition/reference terrain parity, full block seams, completed client gameplay/UI review, integrated/dedicated parity, NeoForge runtime, third-party modpacks, visual approval, and generation performance.

Results live under `docs/validation/` with their exact scope.
