# 0007 — Preserve the preset editor using the 26.2 extraction UI

Date: 2026-11-03 (America/Los_Angeles). Status: source migration implemented; client behavior awaits an actual launch.

## Scope

Adapt the existing preset list, editor pages, controls, import/export path, and live map preview to the real Minecraft 26.2 client APIs. The port retains all existing pages and settings controls. A compiling source tree is not evidence that the editor is usable or visually correct.

The APIs were inspected from the actual resolved game classes with `javap` and from Loom's generated 26.2 sources. No game-class stubs, empty replacement screens, or removed controls are used.

## Rendering and controls

- Replace `GuiGraphics` rendering with `GuiGraphicsExtractor` extraction. Screens implement `extractRenderState`, list entries implement `extractContent`, and custom buttons implement `extractContents` behind the game's final widget-extraction methods.
- Use the target's `text` and `centeredText` methods with explicit opaque white text, and `Matrix3x2fStack` for 2D legend translation and scaling.
- Keep value buttons styled through `Button.Plain`, the same vanilla implementation used by the modern `Button` builder. Labels and the preview keep their custom contents.
- Initialize modern cycle buttons through their initial-value argument; preserve available values, callbacks, labels, tooltips, toggles, and navigation.
- Adapt the selection-list constructor to a visible height and top coordinate. Use the target entry's content rectangle, public widget height setters, and variable entry height for the square preview so scrolling accounts for its full image.
- Preserve selected preset outlines using `entriesCanBeSelected`, and the two-column alignment using the current scrollbar positioning method.
- Use public `Screen.getFont()` and the client singleton instead of assuming protected screen fields have been widened. Screen transitions now use `Minecraft.gui.setScreen`.

## Live preview texture

The preview still generates the real zoomed terrain tile from the edited preset, seed, zoom, and selected render mode. Existing palette calculations are unchanged.

- Give each preview texture a unique mod-namespaced identifier and a descriptive `DynamicTexture` label, then register it explicitly with the texture manager.
- Write the existing packed ABGR palette through `NativeImage.setPixelABGR`; changing to an ARGB setter would exchange color channels.
- Upload regenerated pixels through the actual dynamic-texture API.
- Extract a normalized-UV blit of the entire texture through the target's built-in `GUI_TEXTURED` path. Remove obsolete global OpenGL blend-state calls; the GUI pipeline supplies its render state.
- Release the texture through `TextureManager.release(identifier)` on page close/rebuild, with an idempotent guard. The manager owns closing the registered texture; the preview does not close it a second time.
- Use the mouse handler's actual GUI-scaled coordinate helpers for click-to-copy. Clamp the sampled tile index at the final pixel and exclude the outside right/bottom edge, preventing the old hover code from indexing one past the tile.

Preview generation remains synchronous as in the imported baseline. This migration makes no claim of improved responsiveness or rendering performance.

## Preset export and editor integration

The datapack export/reload path remains in place. Two still-private `CreateWorldScreen` methods are reached through the narrow `CreateWorldScreenInvoker` mixin: `getDataPackSelectionSettings` and `tryApplyNewDataPacks`. Their exact target methods were verified. The client mixin list must include that interface.

`ScreenInvoker` now includes the target's narration bound, so columns remain focusable/narratable widgets. `StructureSettingsPage` retains its existing salt lookup; the verified `StructurePlacement.salt()I` access-widener entry is required until that access strategy is changed.

The existing preset-editor registration mixin's targeted two-entry `Map.of` call still exists in the actual 26.2 `PresetEditor` initializer. Its behavior is retained: preserve vanilla flat/single-biome editors and add the TerraForged editor for the normal preset. Runtime injection remains a separate launch gate.

## Data and language helpers

- Adapt error handling to modern `DataResult.Error`, toast identifiers to `SystemToastId`, toast submission to `Minecraft.gui.toastManager()`, and Fabric datagen callbacks to `FabricPackOutput`.
- Keep the language provider's LGPL-2.1-only header intact. Item-stack and enchantment descriptions are now components: their helper overloads obtain the translation key from a translatable component. A literal description produces an explicit error rather than silently inventing a translation key. Existing string-key translation declarations are unchanged.

## Evidence and outstanding verification

A direct `javac` check of the owned UI/Fabric files and their real source dependencies removed all owned missing-API errors. Its sole remaining owned diagnostic was the deliberately retained protected structure-salt call, pending the validated access widener. The transitive source closure still failed on unrelated migration work, so this is not a successful whole-mod compilation result. The coordinator records the final complete-tree diagnostic after source freeze.

Required client checks remain: open the normal-world customization screen; navigate every page; edit sliders/toggles; create/copy/delete/reload presets; export and apply a datapack; verify seed/zoom/render-mode changes regenerate the map; check color channels, scrolling, GUI scales, hover legends, coordinate copying, and resource cleanup over repeated page changes. Verify resizing and returning from reload/error screens. Capture actual screenshots and logs before asserting UI parity.
