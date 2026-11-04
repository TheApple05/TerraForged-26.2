# 0002: Explicit Fabric platform bindings

Date: 2026-11-03. Status: implemented for the Fabric target; full-source and runtime validation remain open.

## Context

The first 26.2 target uses the single-root Fabric build selected in [decision 0001](0001-build-layout.md). Both upstream `common/src/main/java` and `fabric/src/main/java` are compiled together. The historical Architectury subprojects are not active, so merely retaining the `@ExpectPlatform` annotation library would leave platform methods throwing at runtime. Annotation processing does not replace Architectury's platform transformation.

## Decision

Replace the nine used Architectury platform entrypoints in the following common classes with explicit calls to the existing Fabric implementation classes:

- `platform/ConfigUtil`: obtain the actual Fabric Loader configuration directory.
- `platform/ModLoaderUtil`: query actual Fabric Loader mod presence.
- `platform/DataGenUtil`: instantiate the Fabric registry data provider.
- `platform/RegistryUtil`: obtain writable registries, build and register custom registries, and register data-driven registry codecs.
- `world/worldgen/biome/modifier/BiomeModifiers`: bootstrap codec registration and construct the existing add/replace modifier implementations.

Keep the existing preset, biome-modifier, registry, and datagen paths. Do not replace them with no-op implementations or unconditional values. In particular, forwarding `BiomeModifiers.bootstrap/add/replace` must retain the Fabric implementations' full registration and feature behavior; it must not turn biome changes into empty callbacks.

Remove the unused no-argument `RegistryUtil.getBiomeModifierRegistry()` declaration. A source search of the imported common, Fabric, and Forge trees found only its declaration, and neither platform had an implementation. It was an unbound placeholder, not an executable feature path. World-specific dynamic registries require a registry lookup context; inventing a global return value would be misleading. If an external API later needs this operation, introduce a context-taking method deliberately.

## Target API adaptations

The real 26.2 game and Fabric API class signatures were inspected with Java 25 `javap`:

- `WritableRegistry.register(ResourceKey<T>, T, RegistrationInfo)` takes `RegistrationInfo.BUILT_IN` for built-in registrations, replacing the earlier `Lifecycle.stable()` argument.
- `FabricRegistryBuilder.create(ResourceKey<Registry<T>>)` replaces the old `createSimple` name; `buildAndRegister()` remains the actual registration operation.
- Registry data generation uses `net.minecraft.resources.Identifier` and `ResourceKey.identifier()` rather than `ResourceLocation` and `location()`.
- Fabric's `DynamicRegistries.register(key, codec)` and `getDynamicRegistries()` remain present in the resolved target API. Registry codec dumping and namespaced output directories remain intact.

These adaptations are limited to the affected platform files. They do not change terrain arithmetic or noise kernels.

## Build-layout implications

The `common` path is now historical source organization, not a loader-neutral published module: these five classes explicitly depend on the Fabric source root. Compiling `common` by itself or restoring the old Forge build will not work with this binding. A future second loader needs a deliberate module split or verified platform-dispatch mechanism and its own integration tests. Historical Forge files are retained as provenance, not represented as current support.

Removing `@ExpectPlatform` imports from these classes eliminates their need for Architectury weaving. The build dependency is managed separately by the build owner; this decision does not claim that every historical build artifact has been removed or that the rest of the mod is migrated.

## Validation and limits

The initial unmodified 26.2 compiler diagnostic was frozen before source changes: [initial report](../validation/target-build-2026-11-03-initial.json), 388 source files and 432 compiler errors. The seven Java files changed by this decision retain the modifier registrations and add/replace constructors; the Fabric biome-modifier implementation itself is unchanged.

After implementation, the four Fabric platform implementation classes (`ConfigUtilImpl`, `ModLoaderUtilImpl`, `DataGenUtilImpl`, and `RegistryUtilImpl`) compiled successfully with **javac 25.0.5**, `--release 25`, and the real resolved target classpath from `build/qa/target/compile-classpath.txt`. Compiler exit code: **0**. The provider emits a deprecated-API note. Local output and compiler notes are under ignored `build/qa/platform-binding/`; no test stubs or synthetic Minecraft classes were used. This bounded check validates the target signatures of those four implementation classes, not the full common bridge graph or biome-modifier classes. Check those in the next whole-source diagnostic.

No forwarding-only unit tests are added: duplicating each one-line delegation in a test would not establish the Fabric runtime's registry lifecycle, transformed mixins, dynamic registry loading, or datapack behavior. Meaningful acceptance requires the whole source tree to compile, target-version metadata and mixins to load, and actual packaged client/server runs to exercise registration, preset export, and biome feature changes. Those gates remain open until evidenced separately. A partial compiler improvement is not a playable port.
