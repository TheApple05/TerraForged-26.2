# Classic pillar shapes and material bands

The native `PillarSampler` reproduces the Java mod's Bryce and stone shape graphs,
band offsets, nine seed-derived terracotta palettes and stone cap materials.
It is immutable after construction and can be retained with a world generator.
Construction returns graph errors; callers must propagate them.

The frozen reference contains 38,336 observations from the exact dev.23 Fabric
JAR (`1c9e5e7e…70e886`). Two fresh Minecraft 26.2 / Fabric 0.20.1 servers,
using Fabric API 0.161.0+26.2, produced identical results. Their three relevant
class origins were verified against the installed production JAR. Both native
debug and release builds match every observation, including raw float bits.

The reference covers six integer seeds (including integer extrema), negative and
distant coordinates, all palettes, material rows, palette-index float boundaries,
NaNs, infinities and signed zero. Static shape graphs and palette methods are
read reflectively from the packaged feature. Detail, band, palette-index and
stone-material expressions follow that feature's placement method. This is not
an independent execution of its whole block-placement loop.

The source's integer overflow, float coordinate rounding above 2^24, negative
Java remainder, float accumulation order and truncating casts are intentional.
Changing to Euclidean remainder or double precision would change the terrain.
Palette heights must be positive. Feature callers must use the clipped placement
height for stone caps, but the original unclipped height for Bryce bands.

The first two plain-JVM capture attempts failed: an initial relative classpath,
then Fabric registry access without active loader mixins. Neither produced an
accepted reference. The successful producer launches actual servers from an
ordinary copy of an existing local installation. It invokes no installer or
game download. Production JARs are unchanged; the capture companion is separate.

Run `horizon/tools/capture_pillar_sampler.py --help` to reproduce the reference
with existing local game files. CI needs no game files: it runs
`horizon/tools/verify_pillar_sampler.py --report <fresh-report.json>` against the
frozen observations on Windows and Linux.

This does not install native biome definitions, execute the feature in a world,
apply filtered-cell masks or biome-edge tapering, or validate structures,
scenery or game performance. Those remain integration gates.
