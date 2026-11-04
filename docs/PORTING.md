# Porting plan

## Objective

Faithfully revive TerraForged for Minecraft Java 26.2. Preserve its generation algorithms and identity, and keep customization explicit. The reference includes the original stable generation and UI behavior as well as selected ReTerraForged improvements; those sources must remain distinguishable.

## Decisions already made

- Target: Java Edition **26.2**, explicitly confirmed by the user on 2026-11-03 (America/Los_Angeles).
- Repository: `TheApple05/TerraForged-26.2`, created independently with GitHub reporting `fork: false`. Private initially.
- Preserve source provenance and license notices. No upstream PR is planned.
- Audit branch contents before selection: a newer branch name is not evidence of completeness.
- Preserve algorithm behavior before deliberate terrain changes. Keep compatibility fixes separate from algorithm redesign.
- Deliver both Fabric and NeoForge for modpack use, as requested by the user. Fabric is the first runtime integration; each loader requires its own packaged validation.
- The user wants the general TerraForged experience rather than an exact remembered seed or preset. Classic source remains a reference, with inherited omissions explicitly inventoried.

## Milestones

1. **Reference and reproducibility:** pin source revisions, inventory features and experimental paths, preserve licenses, capture algorithm reference data, establish Java 25 tooling.
2. **Target build:** establish a pinned 26.2 toolchain and dependency graph; migrate registry, serialization, density-function, and loader integration. Record compile failures by subsystem.
3. **Generation:** exercise terrain, continents, climate, rivers, erosion, surface rules, caves, and structure integration in disposable target-version worlds.
4. **Customization:** migrate preset serialization, world selection, controls, preview, config persistence, and error reporting.
5. **Production validation:** test the actual packaged artifact in isolated client and dedicated-server installations; verify new chunks, save/reload, deterministic output, concurrency, and performance.
6. **Visual acceptance and release:** capture reproducible in-game comparisons, review with the user, publish a clearly labeled release only after the required gates pass.

## Completion criteria

- Every inventoried feature has a supported status and evidence, or an explicitly documented and agreed limitation.
- Target client and dedicated server load the packaged JAR and generate terrain with declared dependencies.
- Seed/config determinism, negative coordinates, neighbor boundaries, generation order, concurrency, and save/reload checks pass.
- Preset settings survive serialization and affect the intended generation paths.
- Visual review uses recorded seeds, coordinates, camera, config, versions, and screenshots. User acceptance is distinct from automated checks.
- Performance is measured on a fixed workload and environment; no unsupported claims of speed or compatibility.
- Installation instructions, limitations, attribution, artifact hashes, and reproducible verification commands are present.

## Current status

Both loader builds now have real production client/server evidence. Default and pinned Biomes O' Plenty + GlitchCore + TerraBlender stacks export presets, generate custom terrain, save/reload, and render in actual clients. Each content-stack export retains all 68 registered BOP biomes; 59 are present in the overworld source and actual sampled chunks contain BOP biomes. Results are version/hash scoped, with beta dependency labels retained.

The current candidate also protects Minecraft's asynchronous preset validation screen, fixing an independently reproduced premature Create World window. Exact final candidate checks are recorded in [runtime results](validation/runtime-summary.md). Build snapshots, retained failures, test-only companions, dependency hashes and screenshots are in `docs/validation/`.

The release gates remain open for full classic parity, every preset/control extreme, original-version visual comparison/user acceptance, multiplayer and broader mod combinations, Linux game runtime, and controlled generation-performance budgets. Existing-world migration is not established. The unavailable exact classic Engine source and the 17 missing classic custom variants are explicit reference/restoration gaps rather than hidden port completion claims.
