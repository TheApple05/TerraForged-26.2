# 0004: Resource identifiers and bootstrap API names

Date: 2026-11-03. Status: implemented; whole-source compilation remains in progress.

The first full-source diagnostic reported 96 missing `ResourceLocation` references and 68 missing `BootstapContext` references, among other incompatibilities. These counts are initial compiler diagnostics, not independently broken features or a measure of remaining effort.

The actual downloaded 26.2 `minecraft-merged.jar` was inspected with JDK 25 `javap`:

- `net.minecraft.resources.Identifier` replaces the old resource identifier class. Its constructors are not public; use `parse(String)` or `fromNamespaceAndPath(String, String)`.
- `ResourceKey.identifier()` replaces the old `location()` accessor; the corresponding tag accessor also uses `identifier()`.
- `net.minecraft.data.worldgen.BootstrapContext` is the current spelling of the registry bootstrap interface.
- `net.minecraft.util.Util` is the current utility class package.

Applied these type/accessor/package changes to the active common and Fabric sources. The two constructor patterns in `RTFCommon.location` retain their existing namespace behavior. No noise arithmetic, landform settings, biome distributions, or river algorithms changed. Historical Forge source remains outside the configured target build and is not labelled migrated.

The Fabric manifest now names this independent continuation, credits upstream contributors, points source/issues to this repository, states the exact Minecraft 26.2 and Java 25 requirements, and describes both observed source-license categories. It also points at the icon actually present in the imported resources. These metadata declarations are intent, not runtime compatibility evidence.

The first diagnostic was intentionally captured before these changes. Subsequent diagnostics must include all active source files and show their source fingerprint. Fixing unresolved imports can expose deeper type/signature failures, so error counts alone are not a progress percentage.
