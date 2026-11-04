# TerraForged for Minecraft Java 26.2

An independent, unofficial revival of TerraForged, focused on preserving its terrain algorithms, visual character, and customization.

**Status: port in progress. There is no playable 26.2 release yet.** Successful compilation or a title-screen launch will not constitute feature parity.

The user confirmed **Minecraft Java Edition 26.2** on November 3, 2026. This is a new repository, not a GitHub fork. It is private during initial development; repository visibility is separate from software licensing.

## Original work

- [TerraForged](https://github.com/TerraForged/TerraForged), the original mod and terrain engine.
- [ReTerraForged](https://github.com/racoonman2/ReTerraForged), the subsequent modernization effort.

All imported code must retain its license and attribution. New history in this repository does not imply original authorship of upstream code. Exact commits and imported paths are recorded in [source provenance](docs/SOURCE-PROVENANCE.json).

## Project record

- [Porting plan and completion criteria](docs/PORTING.md)
- [Development log](docs/DEVLOG.md)
- [Regression and acceptance protocol](docs/VALIDATION.md)
- [Upstream audit](docs/research/upstream-audit.md)
- [Target toolchain research](docs/research/target-toolchain.md)

Research records distinguish observed behavior, proposed implementation, and untested assumptions. Never install an experimental build into an existing world you care about; validation uses isolated, disposable worlds.
