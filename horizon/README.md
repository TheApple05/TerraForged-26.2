# TerraForged for Horizon: procedural port and prepared prototype

This is an experimental **finite terrain bridge**, not the completed native
TerraForged generator. Horizon currently targets Minecraft **26.3**; the Java
port in this repository targets **26.2**.

The developer exporter runs the genuine packaged Java mod, completes FULL
chunks and a one-chunk halo, and records all block states and quart biomes.
The native Rust pipeline reconstructs those chunks, derives heightmaps and far
terrain from the same blocks, and never invokes Java while playing. Missing
chunks, wrong seeds, altered snapshots and unknown block/biome IDs fail. There
is no vanilla terrain fallback outside the prepared area.

The exact dev23 [native probe](../docs/validation/horizon-current-dev23.json)
matches 884,736 block states, 13,824 quart-biome samples and 2,304 heightmap/LOD
columns. The minimal hook change is merged into the user's Horizon repository
as `21679eb68f770d3ac1c2cc1c51a535eea37787ab`, with the same tested source tree.
The [dev23 package](../docs/validation/horizon-package-dev23.json) retains the
finite-prototype label and contains neither snapshots nor prepared resources.

Procedural work is isolated in [native-core](native-core/README.md). Complete
cell/river/climate stages and filtered tiles now match all310,104 independent Java
kernel records in debug/release. The bounded cached column sampler matches
196,608 observed columns, with lifecycle checks and an optimized kernel benchmark.
Custom density primitives also have independent coverage. The assembled native
bundle now binds procedural cells/noise/clamps into real compiled Horizon
density graphs, matching 24,768 independent Java scalar/bulk observations in
debug and release. The custom resource decoder also passes those observations
through Horizon's JSON registry; see
[resource integration](../docs/horizon/host-density-registry.md).
See [graph integration](../docs/horizon/host-density-graph.md)
and the [current checkpoint](../docs/horizon/PROGRESS.md). World activation,
complete routers, biomes, surfaces, caves, decorations and structures remain
integration work. The separate prepared bundle does not include this kernel.

The player-facing bundle is under `mixins/terraforged_prepared/`. It uses the
actual Horizon world-generator constructor through horizon-mixins, pinned to
Horizon `9dd5c4093977c718c5c0349032fd3a5434e86afc` and mixins
`e8cd0aa38cffef49cb2e088ef716baa107148c02`. The Horizon commit spells the
constructor result explicitly so the existing cancellable HEAD contract can
accept it; the ordinary generator algorithm is unchanged. Its parent is
`da6b3c353d9bc495704f3658bdb09fade27e2679`, the inspected current
`TheApple05/horizon` main. Earlier `b33bc70c`/`7fb69700` evidence belongs to an
installed-nightly rehearsal and is retained separately.

Set `TERRAFORGED_HORIZON_PREPARED` to a prepared snapshot directory before
starting the native build. Only the overworld noise generator is replaced;
Nether/End and disabled builds retain Horizon generation. This is a developer
prototype and requires that exact source revision to be available to the
builder. A manifest alone does not establish launcher or rendered-game testing.

`python horizon/tools/package.py --biomes <exported-classic-biome-json-directory>`
creates a combined `.hmod` containing the native bundle and the 17 real biome
definitions. Install the same file on the native client/server so their biome
registries agree. The `.hnmod` without `--biomes` contains only code and requires
equivalent biome definitions from another prepared content source. Neither
package contains terrain snapshots, Minecraft binaries or player-prepared game files.

Development steps, from the repository root:

```powershell
python horizon/tools/build_exporter.py --java-home <JDK25> --jar <exact-fabric-jar>
# Run the exporter companion in an isolated production server using a matching
# preset pack. It writes qa-output/horizon-prepared and a provenance report.
python horizon/tools/stage.py --horizon <local-horizon-source> --mixins <pinned-horizon-mixins.exe>
python horizon/tools/run_probe.py --stage <staged-workspace> --resources <own-prepared-26.3-resources> --snapshot <horizon-prepared> --biomes <exported-classic-biome-json-directory> --expected-mod-sha256 <exact-production-JAR-SHA256>
```

No tool here downloads Minecraft, assets or prepared game resources. Use the
player's own local resources. Developer archives and binaries remain under the
repository's ignored `build/qa/horizon` folder.

Remaining work: procedural kernel integration into real native chunks; native
preset UI; typed entities and block entities; structure metadata/locate; scheduled
ticks; full rendered client and native save/reload acceptance. The v1 exporter
refuses chunks containing block entities instead of silently dropping them.
Copied structure blocks do not restore structure starts. No claim of general
Horizon modpack readiness is made.

## Component notices

This is a separate native component under the [MIT license](LICENSE),
Copyright (c) 2026 TerraForged Revival contributors. Its bundles retain the
[Horizon MIT notice](mixins/terraforged_prepared/LICENSE-HORIZON), the
[horizon-mixins MIT notice](mixins/terraforged_prepared/LICENSE-MIXINS), the
TerraForged/ReTerraForged MIT notices and a
[component inventory](mixins/terraforged_prepared/NOTICE.md).

The MIT texts state: "The above copyright notice and this permission notice
shall be included in all copies or substantial portions of the Software."
The native source bundle includes those notices and its helper source; the
Horizon host and build-time tool remain separately versioned dependencies.
Their original license texts are available at
[Horizon](https://github.com/TheApple05/horizon/blob/9dd5c4093977c718c5c0349032fd3a5434e86afc/LICENSE)
and [horizon-mixins](https://github.com/horizon-rust/horizon-mixins/blob/e8cd0aa38cffef49cb2e088ef716baa107148c02/LICENSE).

The Java Gradle projects do not include `horizon/` in their source sets. The
native bundles contain no Java classes, Forge LanguageProvider implementation,
Minecraft binaries or player-prepared game resources. The Java components'
separate notices remain in the repository root and their own JARs.
