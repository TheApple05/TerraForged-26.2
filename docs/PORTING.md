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

The dev23 exact Fabric and NeoForge packages pass functional qualification: all 17 natural saved/reloaded variants, restored visible pillars, base and active-shader clients, pinned optimizer/community-ModernFix/content/structure stacks, real combined-stack client joins and WSL Linux generation/reload. [Functional acceptance](validation/dev23-functional-acceptance.json) binds the packages to 67 evidence records and 30 native captures. [Current readiness](validation/dev23-readiness.md) states the actual limits.

Both controlled performance matrices pass all 24 fresh generation trials and eight cold reloads, with no compiler interference or budget violation. The predeclared workload/budgets are unchanged. [Complete scoped acceptance](validation/dev23-acceptance.json) binds 104 evidence files and 30 images. [Dev23 is published as a prerelease](https://github.com/TheApple05/TerraForged-26.2/releases/tag/v0.1.0-dev.23%2B26.2), with both loader JARs, checksums and validation notes. All seven remote assets were downloaded and hash-verified. Every failed candidate and rejected locator/controller input remains available with its cause and correction.

The 17 formerly missing classic definitions are restored. The modern regional selector and geological palette reconstruction remain adaptations because the exact original Engine object is unavailable. Full classic surface/appearance parity, island-preference spawning, every preset/import/control extreme, migration, authenticated public multiplayer/player load and user visual acceptance remain open.

The separate Horizon26.3 port has a verified finite prepared-world bridge using the merged native generator hook, plus an isolated scalar/vector/Perlin/Simplex foundation matched to independent Java raw-bit fixtures. Infinite continents/rivers/filtered tiles and the full native density/surface/cave/decoration/client/save integration remain unfinished. Finite reconstruction is never substituted for new procedural terrain.
