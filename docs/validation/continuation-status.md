# Dev23 validation checkpoint

Recorded 2026-11-04. This checkpoint preserves dev23 qualification; see the [README](../../README.md) for the current Java download. Native Horizon development continues in [TerraForged-Horizon](https://github.com/TheApple05/TerraForged-Horizon).

## Published Java candidate

**0.1.0-dev.23+26.2** is published in one [Fabric + NeoForge prerelease](https://github.com/TheApple05/TerraForged-26.2/releases/tag/v0.1.0-dev.23%2B26.2). At this release, the repository was private and independent, fork=false. Release target is `a5381773145c72f13efe0be918ee302a39967685`; all three relevant workflows passed, including Windows/Linux native fixtures.

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

Initial native Windows CI failed because Git changed the original frozen TSV to CRLF. The -text fixture rule preserves its unchanged LF Git blob, and bash now fails the multi-command check immediately. The [retained native CI failure](https://github.com/TheApple05/TerraForged-Horizon/blob/main/docs/validation/ci-native-noise-windows-fixture-failure.json) moved with the native evidence; the subsequent Windows/Linux run passed.

## Native port continuation

Native Horizon 26.3 source, documentation, fixtures and evidence have moved to [TerraForged-Horizon](https://github.com/TheApple05/TerraForged-Horizon). That repository retains the `horizon/`, `docs/horizon/` and Horizon-specific `docs/validation/` paths. Follow its current checkpoint for native acceptance and remaining work. Historical Java release records retain their original paths and hashes; the [pre-split log](https://github.com/TheApple05/TerraForged-26.2/blob/c7a477a4a272ef34df9d3f55a95f185f264b40ec/docs/DEVLOG.md) records the earlier shared-repository milestones.
