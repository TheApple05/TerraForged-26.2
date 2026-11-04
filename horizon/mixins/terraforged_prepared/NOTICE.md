# Component notices

The native adapter and developer exporter are original MIT code, Copyright
(c) 2026 TerraForged Revival contributors. See LICENSE in this bundle.

The adapter targets Horizon's MIT API at revision
9dd5c4093977c718c5c0349032fd3a5434e86afc. Horizon is Copyright (c) 2026 Ludovic
Marie and the Horizon contributors; its notice is in LICENSE-HORIZON.

The pinned build-time horizon-mixins tool and generator are MIT, Copyright
(c) 2026 levzzz, revision e8cd0aa38cffef49cb2e088ef716baa107148c02. Its notice
is in LICENSE-MIXINS. The bundle contains helper source and a mixin manifest;
the tool produces source glue when building. Its executable is not included.

The combined content package also preserves the TerraForged and ReTerraForged
MIT notices for the classic biome definitions. The biome JSON is exported from
the separately tested Java mod; it does not port the Java generator kernels.

This is a separate component from the Fabric/NeoForge Java JARs. The native
package contains no Forge LanguageProvider implementation or other Java class
files. Its shared serde/serde_json/sha2 APIs come from the pinned Horizon host;
their implementations and compiled libraries are not vendored in this bundle.
Minecraft binaries, player-prepared resources and generated world snapshots
are not included.
