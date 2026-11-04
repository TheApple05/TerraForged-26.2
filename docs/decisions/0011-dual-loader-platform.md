# 0011: Fabric and NeoForge platform bindings

The requested target is Minecraft Java 26.2 on **both Fabric and NeoForge** for modpacks. Both builds compile the complete common terrain, serialization, preset editor and preview sources. Historical `forge/` remains reference material; it is not included in either build.

`PlatformServices` has one `ServiceLoader` provider in each packaged artifact. Startup fails explicitly if zero or multiple providers are present. The common configuration, mod detection, registry registration, datapack provider and biome modifier facades call that provider. Common code has no Fabric or NeoForge API imports. Fabric retains its working native implementation and existing registry identities.

NeoForge is a separate Gradle build in `neoforge/`, using ModDevGradle 2.0.148 and NeoForge 26.2.0.93. The root Gradle 9.5.1 wrapper and Java 25 successfully compile and package it. Invoke the wrapper using `--project-dir=neoforge`; with the PowerShell helper, pass an explicit `-GradleArgs` array so `-p` cannot be consumed as a PowerShell parameter abbreviation. Each artifact can be versioned with `-Pmod_version=...`.

NeoForge custom built-in registry instances are created once, attached through `NewRegistryEvent`, and populated only during the matching `RegisterEvent`. Common registration requests are queued until that event. Custom datapack registry codecs are registered through `DataPackRegistryEvent.NewRegistry`. Config paths and early optional-mod detection use FML's own APIs. Early detection uses `FMLLoader`'s loading list because mixin selection precedes `ModList` construction.

NeoForge uses its native `neoforge:biome_modifier` registry and serializer registry. Our Add/Replace records implement both the common and native interfaces and keep the same data fields, filter behavior and final modification phase. They modify the native `BiomeGenerationSettingsBuilder.getFeatures(step)` lists; no historical Forge accessor mixin is needed. Registry boundary casts are confined to the NeoForge provider. Crucially, the registry patch cloner uses **NeoForge's native `BiomeModifier.DIRECT_CODEC`**. Other mods' native biome modifiers do not need to implement TerraForged's interface.

The first real NeoForge export exposed another requirement: `RegistrySetBuilder` clones every registry in the original provider, including native structure modifiers and other mods' custom registries. The provider therefore supplies the complete loader datapack registry codec list through `addPlatformRegistryCloners`. Fabric uses `DynamicRegistries.getDynamicRegistries()` and NeoForge uses `DataPackRegistriesHooks.getDataPackRegistriesWithDimensions()`. Each registry retains its actual registered codec. The failed dev5 export is retained as evidence; the correction is part of the next candidate.

Fabric API's transitive access wideners expose several `NoiseRouterData` fields and helper methods used by the common preset routers. The NeoForge compiler revealed those accesses explicitly. The NeoForge access transformer lists the observed fields and exact method descriptors, plus the already-reviewed common surface/ore/structure accesses. It does not widen entire classes' members or import historical Forge access rules.

NeoForge registers the editor with `RegisterPresetEditorsEvent`. Its chunk tile lifecycle and template reload mixins perform the same reviewed operations as Fabric: queue before biomes, release after the features future completes, and reload templates after successful resource reload. The common spawn, noise, surface and optional integration mixins are shared.

## Evidence and limits

- The dev8 Fabric and NeoForge packages compile with no source exclusions. NeoForge compiles 382 common and 8 loader-specific Java files; Fabric compiles the same common files with its own bindings (397 Java files total).
- The exact NeoForge dev5 JAR passed an isolated **production** loader boot, vanilla-world save and clean stop with exit 0. The report records the installed JAR hash and observed production class origin: [`neoforge-dev5-first-boot.json`](../validation/neoforge-dev5-first-boot.json).
- Current package hashes, metadata, class counts and unchanged source snapshots are recorded in [`dual-loader-dev8-build.json`](../validation/dual-loader-dev8-build.json). Earlier candidate reports are retained.
- The exact NeoForge dev8 package passed custom server generation, save and reload with matching sample digests: [`neoforge-dev8-generation.json`](../validation/neoforge-dev8-generation.json), [`neoforge-dev8-reload.json`](../validation/neoforge-dev8-reload.json).
- The same package passed the real client editor, font-fit checks, new custom-world creation and rendering, then a vanilla world in the same process with no TerraForged context. All seven game-rendered screenshots were inspected: [`neoforge-client-dev8-fullflow.json`](../validation/neoforge-client-dev8-fullflow.json).
- Vanilla startup is not proof of custom generation, client editor behavior, TerraBlender compatibility or arbitrary modpack compatibility. Those require separate runtime reports.
- Loader-native biome modifier paths differ intentionally. Export presets through the loader that will consume them; cross-loader exported-pack migration is not yet validated.

Primary API references: the exact NeoForge 26.2.0.93 and FML 11.0.16 published source artifacts inspected locally; [ModDevGradle](https://github.com/neoforged/ModDevGradle); [NeoForge user server documentation](https://docs.neoforged.net/user/docs/server/).
