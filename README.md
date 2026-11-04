# TerraForged for Minecraft Java 26.2

An independent, unofficial revival of TerraForged, focused on preserving its terrain algorithms, visual character, and customization.

**Status: port in progress. There is no playable 26.2 release yet.** Successful compilation or a title-screen launch will not constitute feature parity.

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

Research records distinguish observed behavior, proposed implementation, and untested assumptions. Never install an experimental build into an existing world you care about; validation uses isolated, disposable worlds.
