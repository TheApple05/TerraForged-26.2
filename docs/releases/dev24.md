# TerraForged Revival dev24 - Minecraft 26.2

Fresh Fabric and NeoForge builds from the current repository. This is an experimental prerelease.

## Install

- Minecraft Java **26.2**, **Java 25**.
- **Fabric:** Fabric Loader **0.20.1**, Fabric API **0.161.0+26.2**, and the asset ending in `fabric-mc26.2.jar`.
- **NeoForge:** NeoForge **26.2.0.93** and the asset ending in `neoforge-mc26.2.jar`.

Install only the JAR matching your loader in the instance's `mods` folder. For custom terrain, open Create New World > World > Customize, select/edit a preset, then create a new world. The included installation guide covers preset export and server setup.

## Changes and validation

The Minecraft mod source is unchanged from dev23; dev24 provides a fresh, versioned two-loader build. Subsequent repository work concerns the separate Horizon native prototype, which is not included in these Minecraft JARs.

Both loader builds, core/cache regression checks, packaged cache lifecycle checks, classic biome policy and pillar checks, and production dedicated-server boot/save/stop checks passed. The attached JSON files record this run. Uploaded files are downloaded and verified against `SHA256SUMS` before publication. `SOURCE-COMMIT.txt` identifies the source revision.

Dev23's broader client, shader, modpack, custom-world and performance evidence remains historical evidence for those exact dev23 artifacts. Those full suites are not repeated for dev24. This release does not claim new client visual acceptance, complete classic parity, cross-loader migration support, or a completed native Horizon port.

## Build this version

Use Java 25 and pass `-Pmod_version=0.1.0-dev.24+26.2` to the documented Fabric and NeoForge Gradle commands. The version override is recorded in `BUILD.json`.
