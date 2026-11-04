# TerraForged for Minecraft Java 26.2

An independent, unofficial revival of TerraForged, focused on preserving its terrain algorithms, visual character, and customization.

**Status: the experimental Fabric 26.2 package generates custom terrain, saves, and reloads in an isolated production server. Fabric client validation and NeoForge implementation are in progress. No release is published yet.** Both Fabric and NeoForge are requested targets for modpack use.

The user confirmed **Minecraft Java Edition 26.2** on November 3, 2026. This is a new repository, not a GitHub fork. It is private during initial development; repository visibility is separate from software licensing.

## Original work

- [TerraForged](https://github.com/TerraForged/TerraForged), the original mod and terrain engine.
- [ReTerraForged](https://github.com/racoonman2/ReTerraForged), the subsequent modernization effort.

All imported code retains its license and attribution. New history in this repository does not imply original authorship of upstream code. Exact commits and imported paths are recorded in [source provenance](docs/SOURCE-PROVENANCE.json). The root license is MIT, with an LGPL-2.1-only language-provider component described in [third-party notices](THIRD_PARTY_NOTICES.md).

The implementation starts from ReTerraForged **1.20.1 / 0.0.6**, commit `29239f1f164d19dbfcccaca4a8277d64e784207c`. Original TerraForged **1.16.5 / 0.2.16**, commit `3ada751ac25cd3eecee5d198870a3abb89734f52`, is the classic feature/visual reference. Newer branches contain experimental or omitted generation paths; [the audit](docs/research/upstream-audit.md) explains the selection.

## Project record

- [Porting plan and completion criteria](docs/PORTING.md)
- [Development log](docs/DEVLOG.md)
- [Regression and acceptance protocol](docs/VALIDATION.md)
- [Packaged runtime results and current limitations](docs/validation/runtime-summary.md)
- [Classic feature inventory](docs/FEATURE-PARITY.md)
- [Upstream audit](docs/research/upstream-audit.md)
- [Target toolchain research](docs/research/target-toolchain.md)
- [GitHub work tracker](https://github.com/TheApple05/TerraForged-26.2/issues)

## First verified foundation

The unchanged noise/math utilities pass **2,167 frozen upstream-derived records across four isolated Java 25 JVM runs**, including shuffled and parallel sampling. A mutation self-test confirms that a changed hash kernel is rejected. This covers four mathematical utility classes, not full terrain, chunk seams, or gameplay. See [the exact scope and commands](docs/validation/core-regression.md).

With Python 3.9+ and a full Java 25 JDK selected in `JAVA_HOME`:

```shell
python tools/verify_core.py --self-test
```

CI runs the same scoped checks on Linux and Windows. A passing kernel workflow does not mean the full 26.2 mod builds or runs.

The complete source compiler pass succeeded with the pinned Java 25/Fabric toolchain. [Compiler evidence](docs/validation/target-build-2026-11-03-compile.json) and the prior failed inventories are retained. Additional checks passed for [density adapters](docs/validation/density-regression.md), exact [legacy vegetation RNG parity](docs/validation/legacy-patch-parity-2026-11-03.json), and [dimension data](docs/validation/dimension-data-2026-11-03.json).

Packaged validation found and corrected a tile-cache deadlock and a slider event bug. The corrected Fabric package passes two fresh custom-world generation runs and a save/reload run with the normal watchdog. A sampled tree canopy differed between fresh runs at a chunk edge; broader checks are investigating decoration completion before claiming whole-world determinism. [Runtime evidence](docs/validation/runtime-summary.md) records the artifact hashes, failures, and exact scope.

On Windows, `tools/gradle.ps1` selects Java 25 for this project without changing global Java settings:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File tools/gradle.ps1 compileJava --console=plain
python tools/diagnose_port.py --report build/qa/target/local.json
```

Experimental packaging requires `jar -PexperimentalPackage` for isolated runtime checks. It is not a release designation. See [the build layout and validation boundaries](docs/decisions/0001-build-layout.md).

Research records distinguish observed behavior, proposed implementation, and untested assumptions. Never install an experimental build into an existing world you care about; validation uses isolated, disposable worlds.
