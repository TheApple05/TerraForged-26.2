# Development log

Dates use America/Los_Angeles. This log records outcomes and evidence, not an assertion that the entire port works.

## 2026-11-03 — project initiation

- User authorized implementation, an independent GitHub repository, and ongoing documentation.
- Workspace was empty. Created a fresh Git history and private repository `TheApple05/TerraForged-26.2`; GitHub's creation response reports `fork: false`.
- Verified Mojang's version manifest lists `26.2` as a release and does not list `1.26.2`. User explicitly confirmed the 26.2 target.
- Cloned both upstream repositories under ignored `.reference/` for branch and history inspection.
- Started independent upstream/feature and target/toolchain audits.
- Original default branch README warns that it is stripped back. Early inspection found experimental terrain overrides in ReTerraForged's 1.20.2 branch; choosing a baseline requires source inspection rather than taking the newest branch automatically.
- Used the authorized DeepSeek Delegator for a bounded regression-checklist draft. Codex reviewed and adapted that draft into the validation protocol. Worker output is not evidence of passing tests.

### Baseline selection and import

- Selected ReTerraForged 1.20.1 / 0.0.6 at `29239f1f164d19dbfcccaca4a8277d64e784207c` after inspecting all branches. Imported the complete tree except upstream README and ignore rules in commit `b593342`.
- Original TerraForged 0.2.x at `3ada751ac25cd3eecee5d198870a3abb89734f52` declares 1.16.5 / 0.2.16 and serves as the classic reference. Its Engine dependency must be pinned when constructing the original runtime reference.
- Rejected the 1.20.2 plateau override and mostly disabled preset patching as the preservation baseline. The newer 1.21.10 `datagen-rework` omits important hydrology/configuration paths and changes licensing; no code from that branch was imported.
- Preserved original/revival MIT notices and the file-level LGPL-2.1-only LanguageProvider notice, with component license text and source provenance.
- Created GitHub issues [#1](https://github.com/TheApple05/TerraForged-26.2/issues/1) through [#4](https://github.com/TheApple05/TerraForged-26.2/issues/4) for reference parity, 26.2 generation migration, customization, and release validation.
- Chose Fabric as the first integration target using Java 25. Forge source remains in history/tree as a reference and is not advertised as supported on 26.2.

Next: establish reproducible algorithm checks and exercise the full source against the actual target compiler.
