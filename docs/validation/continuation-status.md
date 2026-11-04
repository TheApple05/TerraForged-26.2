# Current validation checkpoint

Updated 2026-11-04. Java modpack evaluation milestone is complete; procedural Horizon generation remains active work.

## Published Java candidate

**0.1.0-dev.23+26.2** is published in one [Fabric + NeoForge prerelease](https://github.com/TheApple05/TerraForged-26.2/releases/tag/v0.1.0-dev.23%2B26.2). The repository remains private and independent, fork=false. Release target is `a5381773145c72f13efe0be918ee302a39967685`; all three relevant workflows passed, including Windows/Linux native fixtures.

- Fabric SHA-256: `1c9e5e7e8d286c145a647f466b2f908cda721d10672c40949864f1015570e886`.
- NeoForge SHA-256: `61ed42e28fbebbb1166b2a757e82ddc247038b70b19cdbdfc3a70a6621e80fce`.
- Release record: `release-dev23.json`, seven uploaded assets downloaded again and SHA-256 verified.
- Final local assets: `build/distributions/dev23-release-final/`; both loader/game names are explicit.
- Full JDK: `C:/Users/Main/AppData/Local/Programs/Java/jdk-25.0.5+8`.

Complete acceptance `dev23-acceptance.json` binds 104 evidence files and 30 native images. All 17 natural variants pass FULL/stored/source/cold-reload checks on both loaders, with actual positive Bryce and capped stone pillars. Both base and active-shader clients, all three compatibility cases per loader, actual combined-stack loopback joins, Linux generation/reload and all 36 native structure-column comparisons pass. Exact final cache lifecycle passes 3,511 assertions per loader; scoped/nested real-world cell probes pass 801 each with all 24 fields retained and 384 lookup calls reduced to four.

Both controlled benchmark matrices pass all 24 fresh generation trials and eight cold reloads, with no compiler interference/budget violations. Base custom new-region medians are approximately 5.7-6.0 seconds per 64 requested FULL chunks; optimized medians are approximately 4.2-4.4 seconds. Custom whole-process peaks are approximately 1.30-1.34 GiB. Matched vanilla is faster. See `dev23-performance.md` and the raw matrices. Do not turn these local requested-chunk measurements into FPS, player capacity or universal minimum requirements.

The published artifact stays experimental for new-world modpack evaluation. Full classic Engine/geology/visual parity, island preference, every preset/import/control extreme, migration, public authentication/load, long exploration, arbitrary packs, finished far LODs and user aesthetic acceptance remain outside the accepted scope.

## Retained corrections and inputs

Decisions 0020-0023 cover reentrant cache publication, actual grass footing tags, scoped accurate biome queries/coarse stronghold-ring search, and NeoForge original-biome export. Dev20 failed cold warm-beach agreement; dev21's global filtering caused excessive remote tile work; dev22 fixed that scope but exposed double-applied native modifiers in the combined NeoForge export path. Dev23 passes the corrected cases. Preserve every rejected report/JAR.

Qualified natural inputs remain `classic-biome-dev20-qualified-targets.json`, its water targets and `classic-biome-dev19-pillar-locations.json`. The corpus omits the expensive approximate sparse survey while retaining actual saved/reloaded holders and pillar columns.

Initial native Windows CI failed because Git changed the original frozen TSV to CRLF. The -text fixture rule preserves its unchanged LF Git blob, and bash now fails the multi-command check immediately. The retained failure is `ci-native-noise-windows-fixture-failure.json`; the next Windows/Linux run passes.

## Horizon state and active worktree

The exact23 finite bridge, dense native probe and deterministic prototype package pass. `horizon-current-dev23.json` compares 884,736 blocks, 13,824 quart-biome samples and 2,304 heightmap/LOD columns. The merged minimal Horizon hook is `21679eb68f770d3ac1c2cc1c51a535eea37787ab`, with the tested tree of `9dd5c4093977c718c5c0349032fd3a5434e86afc`. Only the player's existing prepared26.3 resources were used. The prototype is excluded from the Java release.

The isolated native-core scalar/vector/Perlin/Simplex foundation matches 34,976 independent Java records (56,080 values) in debug/release. Seed, PosUtil and FastRandom match another 422 frozen records. Native CI passes both Windows and Linux. Provenance and DeepSeek routine-draft review/cost estimates are in `docs/horizon/`. Planning/integration/final checks remain with Codex.

Two independent Java JVMs also agree on the full kernel oracle: six long seed cases, all 24 Cell fields, terrain/rivers/climate modes and complete production-filtered tiles with their actual borders. Each capture set has 310,104 cells; repeat contexts produce 620,208 observations. The high32bit pair confirms Java's actual narrowing. Frozen files are under `horizon/tests/fixtures/kernel-v1`; the archive verifier checks raw schema/finite values/counts/completeness. This is an independent reference, not a native full-kernel pass.

Further native work is isolated at **C:/Users/Main/Documents/Codex/TerraForged-native**, branch **codex/horizon-noise-graphs**, based on release-source a538177. It currently has uncommitted Perlin2/Simplex2 implementations and a shared interpolation visibility adjustment; these additional families are not validated yet. Continue there with independent Java fixtures, typed noise composition, then the complete continent/river/filtered-tile kernel. Keep this main checkout's published Java artifacts unchanged.

Native infinite terrain, density/surface/cave/decoration integration and native rendered client/save acceptance are unfinished. Do not substitute a height-only terrain fill or the finite snapshot bridge. The dependency inventory and next milestone remain in `docs/horizon/`.

The earlier parallel Codex workers stopped at a service usage limit. No retry or model substitution bypassed that limit. Root continued locally and used the separately authorized bounded DeepSeek routine tasks within their configured budget/session.
