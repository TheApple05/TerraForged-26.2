# Native preset selection and packaging

The prototype can now select the procedural native pipeline through Horizon's
real world constructor. This work targets Horizon 26.3; the published Fabric
and NeoForge dev23 artifacts target Java 26.2 and are unchanged.

The opt-in preset is `terraforged:default`. Its overworld stem records noise
settings `terraforged:overworld`. The constructor preserves Horizon's precedence:
a dimension override wins over a saved stem, which wins over a newly selected
preset. Opening an existing vanilla world with a native preset argument therefore
keeps its vanilla generator. Non-overworld constructors retain engine behavior.
The create-world UI currently offers only built-in engine presets; exposing the
custom preset in that UI is separate work. The constructor probe passes the
preset identifier directly and must not be described as a UI playtest.

[Constructor evidence](../validation/horizon-native-world-entry-v1.json) records
two fresh optimized processes with identical outputs. Both run the actual
constructor, compare origin/distant native terrain against the explicit factory,
recognize a native saved-stem value and check conflicting vanilla/native
selection. Vanilla origin output agrees before/after; Nether/End constructor
level and sea properties agree with their baseline. This does not establish
full Nether/End chunk parity or a disk save-file round trip.

The exact constructor executable SHA-256 is
`d129a9e11722669fd0ecd03b288114de51fd36f568016f30c90385e99aec85a7`.
After packaging the numerical tables at their allowed crate-level data paths,
[full-source validation](../validation/horizon-native-world-entry-v2.json)
reruns the same constructor checks in two fresh optimized `fast` processes.
Both produce the exact same observation digest as the earlier release build;
the new executable is
`e4c6e0c810bf2ce533acdbd97233982c4693ee7bf349bff1d1e94effeb0ee55f`.
The entry library also passes Clippy with warnings denied; see
[lint evidence](../validation/horizon-native-entry-lint-v1.json).

`horizon/tools/package_native.py` packages the source-only mixin and its own
preset/language files. It includes no Minecraft resources, Java binaries or
terrain snapshots. All manifest versions derive from the mixin manifest.
The first immutable local package had mismatched versions and an unsupported
root README path; it is retained for rejection testing. The corrected package
now passes the pinned engine's native bundle, content manifest and content
readers via `verify_native_package.py`. A second rejected archive exposed the
reader's requirement that numerical data live in `kernel/data`, rather than
`kernel/src/data`. The staged include paths now follow that contract without
altering table bytes. The [failed reader run](../validation/horizon-native-package-readers-v1.json)
is retained alongside the [passing reader/negative-case run](../validation/horizon-native-package-readers-v3.json).
The latter rejects the original root README and an independently mismatched
manifest version, and confirms all executable source bytes match the stage.
The accepted archive SHA-256 is
`856aa6fc2a9dbf4991e0665bb80e39fe9babce4ee32dd51443975291eaaa4a23`.

This bundle requires commit
`e4d7699a8d4ff5b18a966601b9c3e8346f4571e5` from `TheApple05/horizon`, plus
horizon-mixins `e8cd0aa38cffef49cb2e088ef716baa107148c02`. This is a source
prototype for that engine, not a standard-launcher or player-qualified release.
Full-source integration uses `stage_native_host.py --entry --full` in a fresh
ignored developer workspace. It exports pinned source only and requires no
Minecraft download; game checks use the player's existing prepared resources.

Native classic biome/feature restoration, source carver settings, spawn and
structures, actual saves, rendered client/launcher behavior, controlled native
performance and mod combinations remain acceptance work. Stored-stem selection
alone does not qualify those paths.


A later engine pin and actual client/save checks are documented in
[current engine and client evidence](current-engine-client.md). The E4 archive
above remains the exact historical reader-tested package; it is not silently
replaced by a package for the newer engine.
