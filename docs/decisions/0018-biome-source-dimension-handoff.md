# 0018: Apply classic selection to the dimensions actually used by the game

Date: 2026-11-04, America/Los_Angeles.

Patching `worldgen/world_preset/normal.json` alone does not install the new biome source on every creation path. The rejected dev14 runs exposed two separate cases.

NeoForge's registry exporter also writes `data/minecraft/dimension/overworld.json`. The actual 26.2 `WorldDimensions.bake` prefers that dimension resource over the selected world preset. Dev14's exported world preset had the new wrapper, but its dimension resource retained the old source. The server generated ordinary modern biomes and the new setting/source agreement check correctly failed. Fabric's registry provider did not export this dimension entry in the corresponding base run.

Vanilla's client datapack validation serializes and rebinds the **existing selected dimensions**. For a customizable preset it updates the selected preset holder without recreating those dimensions. Consequently, an otherwise passing editor/create-world flow still had only 55 possible biomes. That old flow is retained as UI/cache evidence; it is not classic selection acceptance.

The shared preset patch now updates an existing compatible overworld dimension as well as compatible world presets. It preserves the installed dimension type, generator settings, complete delegate source and other dimensions. If selection is disabled after an enabled preset, the old wrapper is unwrapped rather than accumulated or silently retained.

The client `WorldCreationUiState.setSettings` hook updates the actual selected overworld generator during datapack reload. It reads the newly loaded preset and biome registry, retains the generator's delegate and noise settings, and installs or removes exactly one wrapper. It runs before the new state is exposed to the screen. Flat/debug/unknown generator types and other noise-settings families are untouched.

Client world-creation registry views omit the DIMENSIONS layer. The rejected dev15 client run caught a `Missing registry: minecraft:dimension` error when an unconditional dimension patch was added. `RegistrySetBuilder` needs the original registry for every patched key, so the dimension patch is now added only when that registry exists in the supplied view. The selected-dimensions client hook handles the other case.

Acceptance must inspect the actual selected source before creation, then the integrated/dedicated generator's source and all 17 possible holders. A successful export, an enabled Boolean, or an existing old UI check alone is insufficient. Packaged acceptance belongs to the exact-artifact reports; this decision records the implementation and retained counterexamples.
