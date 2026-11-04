# Minecraft Java 26.2 toolchain investigation

Checked on 2026-11-03. The user confirmed **Minecraft Java 26.2** in chat; `1.26.2` was not an actual matching release in the official manifest. This report is research evidence, not a successful build or compatibility claim.

## Verified target and Java

The [official version manifest](https://piston-meta.mojang.com/mc/game/version_manifest_v2.json) resolves release `26.2` to [this version descriptor](https://piston-meta.mojang.com/v1/packages/09165f570b16394790c4e37c93b0918729615d17/26.2.json). Its `javaVersion` is component `java-runtime-epsilon`, major version **25**. Its downloads contain client/server artifacts and do not contain the old client/server mapping artifacts.

Local full JDK verified by running both `bin/java.exe -version` and `bin/javac.exe -version`:

```text
C:\Users\Main\AppData\Local\Programs\Java\jdk-25.0.5+8
Temurin OpenJDK 25.0.5+8 LTS, javac 25.0.5
```

A separate older JDK also exists at `C:\Users\Main\Documents\minecraft slop mods\.tools\jdk-25.0.4.1+1`. Use the installed 25.0.5 JDK explicitly for this project; the shell's default Java points at Java 8. Do not modify global Java settings to build this mod.

## Fabric: recommended first implementation target

Recommendation: start with **Fabric**, retaining a clear boundary between terrain algorithms and loader integration. ReTerraForged already has a Fabric entrypoint, and this approach avoids migrating Forge-specific registration/events while also replacing its outdated Minecraft hooks. This is an implementation recommendation, not evidence that Fabric terrain compatibility is better than NeoForge.

Proposed reproducible initial pins:

- Minecraft: `26.2`.
- Java compilation/runtime: `25`.
- Gradle: `9.5.1`.
- Plugin: `net.fabricmc.fabric-loom`, version `1.17.21`.
- Fabric Loader: `0.20.1`.
- Fabric API: `0.161.0+26.2`.

The [official Fabric 26.2 release article](https://fabricmc.net/2026/06/15/262.html) recommends Loom 1.17 and Gradle 9.5.1. Loader `0.19.3` in that article was its release-time recommendation; the live [26.2 loader metadata](https://meta.fabricmc.net/v2/versions/loader/26.2) now marks `0.20.1` stable. The [Loom Maven metadata](https://maven.fabricmc.net/net/fabricmc/fabric-loom/maven-metadata.xml) contains stable `1.17.21`, and [Fabric API Maven metadata](https://maven.fabricmc.net/net/fabricmc/fabric-api/fabric-api/maven-metadata.xml) contains `0.161.0+26.2`.

The current [official example build](https://github.com/FabricMC/fabric-example-mod/blob/26.2/build.gradle) uses ordinary `implementation` dependencies for Loader/API, no mappings dependency, and Java 25. Its [moving properties](https://github.com/FabricMC/fabric-example-mod/blob/26.2/gradle.properties) currently select Loom `1.19-SNAPSHOT`; its wrapper has advanced to Gradle `9.8.2`. Do not copy moving snapshot pins automatically. The stable proposal above still requires an actual build to prove resolution and compatibility.

[Fabric's Loom documentation](https://docs.fabricmc.net/develop/loom/) distinguishes the non-obfuscated plugin `net.fabricmc.fabric-loom` for 26.1+ from `net.fabricmc.fabric-loom-remap` for older versions. For non-remapping builds, included dependencies are packaged by `jar`. The old Architectury/remap/shadow configuration cannot simply be retained unchanged. Build the modern loader path directly first, then add another loader using reviewed shared-source boundaries if needed.

## NeoForge: supported alternative

The [official 26.2 ModDevGradle MDK](https://github.com/NeoForgeMDKs/MDK-26.2-ModDevGradle) currently provides:

- NeoForge `26.2.0.93` in `gradle.properties`.
- Plugin `net.neoforged.moddev` version `2.0.148` in `build.gradle`.
- Gradle `9.2.1` in `gradle/wrapper/gradle-wrapper.properties`.
- Java toolchain 25, plus client, dedicated-server and GameTest run configurations.

The version also appears in [NeoForge's official Maven metadata](https://maven.neoforged.net/releases/net/neoforged/neoforge/maven-metadata.xml). This is a viable later loader target or an alternative first target if concrete integration needs justify it. It would require migrating the old `net.minecraftforge` APIs rather than only changing version numbers.

[NeoForge's 26.1 migration announcement](https://neoforged.net/news/26.1release/) confirms removal of Minecraft obfuscation and the Java 25 transition. Official parameter names are available, so a legacy Mojang/Parchment remapping pipeline is unnecessary for the new target. It also documents changed `ChunkPos` construction/packing APIs, relevant to the old terrain cache and coordinate utilities.

## Forge availability

[Forge's official 26.2 downloads](https://files.minecraftforge.net/net/minecraftforge/forge/index_26.2.html) list latest `26.2-65.1.6` and recommended `26.2-65.1.5` as checked. Forge therefore exists for the requested game release. Its MDK/toolchain has not been built here, and this investigation does not recommend promising a Forge artifact before testing that path. Do not confuse continued Forge availability with compatibility of ReTerraForged's old Forge 48.x code.

## Concrete migration blockers in the selected modern source

Source examined: ReTerraForged `1.20.2` at `98fd3a28f2ebd638319e797a9138a3418484150f`. These are implementation work items, not fixes already completed.

1. **Chunk lifecycle hooks.** `common/src/main/java/raccoonman/reterraforged/mixin/MixinChunkStatus.java` targets `method_39464` and `method_51375` with `remap=false`, and takes `Either<ChunkAccess, ChunkHolder.ChunkLoadingFailure>`. Its hooks queue/drop terrain tiles and toggle fast lookups. Rebinding to current chunk-generation tasks must preserve ordering and cache lifetime; deleting the mixin would hide a functional/performance regression.
2. **Noise and surface internals.** `MixinNoiseChunk`, `MixinNoiseBasedChunkGenerator`, `MixinSurfaceSystem`, `MixinRandomState` and the access widener depend on exact constructors, private fields and descriptors. Compare against generated **26.2** source before changing these injections. The generation-height optimization, aquifer fluid selection, climate lookup and density wrapper must all remain active.
3. **Density traversal and serialization.** The upstream density functions implement old traversal and `KeyDispatchDataCodec` patterns. Registry dispatch, preset codecs and dynamic registry loading need explicit migrations and round-trip tests.
4. **Preset UI and preview rendering.** The upstream client code uses `GuiGraphics`, `Minecraft#setScreen`, `NativeImage` and old drawing APIs. The terrain customization UI is part of parity; a server-only implementation does not satisfy that feature.
5. **Dependencies and optional mixins.** The existing project pins Loader `0.14.22`, Fabric API `0.89.2+1.20.2`, Forge `48.0.4`, TerraBlender `3.2.0.14`, Architectury Loom `1.4-SNAPSHOT`, Java 17 and Shadow `7.1.2`. Treat these as historical evidence. Optional TerraBlender/World Preview mixins need their own supported dependency versions and runtime tests; do not enable obsolete descriptors globally.

The [NeoForged 26.2 vanilla migration primer](https://github.com/NeoForged/.github/blob/main/primers/26.2/index.md), also linked by Fabric's release article, documents changes directly relevant here: `DensityFunction#mapChildren`, default `mapAll`, removal of `DIRECT_CODEC`/`HOLDER_HELPER_CODEC`, `SurfaceRules.ConditionSource#codec` returning `MapCodec`, holder-aware biome conditions, split 2D/3D noise conditions, and changed surface context/update signatures. The old weird-scaled sampler has a non-equivalent replacement, so preserve the algorithm deliberately. It also documents current `ChunkStatusTasks` methods. These are migration leads; verify each exact target descriptor against the downloaded game before implementing.

The [1.20.5 primer](https://github.com/NeoForged/.github/blob/main/primers/1.20.5/index.md) records the earlier `ChunkStatus` package move and changed chunk result handling; the [1.21 primer](https://github.com/NeoForged/.github/blob/main/primers/1.21/index.md) records generation methods dropping their explicit `Executor`. These intervening changes explain why the 1.20.2 mixin signatures cannot survive a version-property bump.

## First verification gates

- Resolve pinned dependencies and compile the preserved terrain core with Java 25; record every source adaptation.
- Establish deterministic core samples at fixed seeds/coordinates before altering Minecraft integration.
- Generate 26.2 game sources, migrate hooks by actual signature and audit required mixin applications.
- Boot client and dedicated server using the packaged JAR with development mode disabled.
- Generate/save/reload actual custom-preset chunks, including negative coordinates and tile boundaries; verify terrain, biome, surface, carver and structure behavior separately.
- Record reproducible visual comparisons and generation timings. A resolved toolchain, successful compile or title screen is not feature parity.
