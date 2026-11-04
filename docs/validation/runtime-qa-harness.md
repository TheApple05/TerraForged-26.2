# Separate runtime QA companion

This is test infrastructure for the unfinished 26.2 port. The companion is built from `tests/runtime/` and the actual biome-decorator regression probe. It is a **separate Fabric mod JAR**, depends on `reterraforged`, and is never included in the production mod's source sets or packaged artifact.

## Build

After the actual production source graph compiles and `writePortClasspath` has resolved real 26.2 artifacts:

```powershell
python tools/qa/build_runtime_qa.py --java-home 'PATH_TO_JDK_25'
```

The default output is ignored `build/qa/runtime/terraforged-runtime-qa.jar`. By default the compiler uses `build/classes/java/main`; `--production-jar PATH` instead compiles against an explicit packaged candidate. The builder records source hashes, its compilation result, the companion SHA-256, and the compilation artifact in `build-result.json`. A successful companion build does not establish a successful server launch.

The packaged runtime must be launched by the real Fabric server harness with the exact production JAR and this additional QA JAR. No generated Minecraft classes or method stubs are used. The callback rejects development mode and requires `RTFCommon` to originate from a real JAR. It records the game/JDK/loader/mod version, Fabric mod origins, actual production class origin, and SHA-256. When `terraforged.qa.expectedModSha256` is supplied, a mismatch fails the run.

## Runtime contract

- `-Dterraforged.qa.mode=export|generate|reload` is required; no missing-mode fallback runs an export.
- `-Dterraforged.qa.output=ABSOLUTE_PATH` names the output directory.
- Optional `-Dterraforged.qa.expectedModSha256=HEX` verifies the exact production artifact.
- The real `ServerLifecycleEvents.SERVER_STARTED` callback sets `serverStarted: true`. A process merely starting is insufficient.
- `<output>/result.json` has `status: "passed"` or `"failed"`, the selected mode, node evidence, runtime provenance, and errors with stack traces. The same result is emitted as `TERRAFORGED_QA_RESULT_JSON=...`. Node progress uses `TERRAFORGED_QA_NODE_JSON=...`.
- The callback requests a clean server stop in `finally`, including after an exception. The outer harness must also verify process exit and completed shutdown/saving logs. The result marker alone is insufficient.
- `saveCompleted` becomes true only after an explicit `server.saveEverything(false, true, true)` returns successfully in generation/reload modes. Export relies on normal shutdown saving and does not claim that explicit save check.

The companion's only writes are QA evidence, the requested exported datapack, and deliberate saves of the isolated test world. It neither edits the production JAR nor interacts with a user's existing worlds.

## Export mode

Use a fresh isolated vanilla server world and an empty output pack directory. Call the production pipeline directly:

```java
Datapacks.makePreset(Presets.makeLegacyDefault(), server.registryAccess(), distinctCachePath, outputPresetPack, name).run();
```

Require real `pack.mcmeta`, valid pack metadata, and the actual custom `Preset.KEY` registry JSON containing world, terrain, and river settings. Record every exported file's SHA-256 and JSON/file counts. If generation throws, record the error and any partial file counts/metadata instead of accepting stale output. Existing nonempty output is rejected.

This exercises real registry patch construction and serialization, including the provider-owned serialization lookup. It does not establish that Minecraft can subsequently load the exported pack; generation mode tests that next.

## Generation mode

Install the exported pack **before creating a fresh server world**. Require all of the following before taking samples:

1. The custom preset registry contains `Preset.KEY`.
2. The actual random state implements `RTFRandomState`.
3. Its preset, `GeneratorContext`, and `TileCache` are nonnull.

These checks deliberately reject a successful vanilla fallback.

Load full chunks `(0,0)`, `(-1,-1)`, `(7,0)`, `(8,-1)`, `(-8,7)`, and `(-9,8)`. For the default tile setting these include positive/negative tile boundaries; record the actual tile coordinates in the evidence. Sample local X/Z positions `0`, `7`, and `15` in each chunk, giving **54 columns**. Record world surface height, actual biome ID, surface block, terrain/climate type, and raw float bits for height, gradient, temperature, moisture, erosion, weirdness, river mask, height erosion, and sediment. Require finite fields and heights within the actual dimension bounds.

Hash stable serialized observations, excluding timing and provenance. Map keys included in this digest are ordered so JVM collection iteration salts cannot cause false cross-run differences. Store `generation-baseline.json` and flush an actual server save.

## Reload mode

Restart the same isolated world with the same production artifact and pack. Preserve the existing `generation-baseline.json`; the outer harness archives/removes only the previous `result.json`. Repeat observations and require the same digest, then explicitly flush another save. A missing baseline fails the run.

The callback also runs the actual `TemplateBiomeDecoratorProbe` after registry initialization. Its three assertions verify distinct per-biome overrides and default fallback using real target holders and the production lookup helper.

## Limits

These are bounded runtime probes. They do not establish visual parity, every terrain/biome/structure, structure placement quality, all presets, extreme settings, concurrency stress, multiplayer behavior, performance, or compatibility with third-party mods. A compiled JAR or passed export alone is not a completed port. Exact run evidence belongs in separate runtime result reports after execution.
