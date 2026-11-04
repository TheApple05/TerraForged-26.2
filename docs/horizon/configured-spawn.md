# Configured native spawn center

The Java preset supplies an empty climate spawn-target list. Its server mixin
returns `SpawnType.getSearchCenter` directly in that case. The native pipeline
now follows that behavior: CONTINENT_CENTER uses the owned heightmap's nearest
continent center to (0,0), and WORLD_ORIGIN returns (0,0). ISLANDS retains the
Java source's (0,0) fallback; its unfinished island-search TODO is not solved here.
Missing or unsupported modes are errors. The mechanical pipeline forwarder
keeps this explicit spawn override when regenerated.

The [frozen Java reference](../../horizon/tests/fixtures/native-spawn-v1/metadata.json)
was captured in two fresh, actual Minecraft 26.2 / Fabric 0.20.1 dedicated
servers from the exact dev.23 production JAR. SpawnType, GeneratorContext and
Preset class origins were checked against that installed artifact. All three
modes and eight long seeds, including 32-bit wrapping cases, agree between
processes. Native [debug and release](../validation/horizon-native-configured-spawn-v1.json)
match all 24 observations; invalid modes are rejected. Formatting, targeted
warnings-denied Clippy and the six existing resource-transform tests pass.

A [separate actual-factory test](../validation/horizon-native-configured-spawn-factory-v1.json)
runs the staged production generator twice in isolated fresh processes. Both
return the independent Java center (2298,-6656) for seed 8675309. The previous
inherited search returned (120,51), with about 97 seconds inside that call in
the diagnostic run. The new factory returns its already-computed center.
Neither those timings nor concurrent client runs are controlled performance
evidence. The immutable [failed reference attempts](../validation/horizon-native-spawn-capture-failures-v1.json)
record missing custom noise resources, then a wrong registry directory in the
producer. The corrected producer loads the original preset's custom noise
resources in their proper registry; neither failed attempt supplied expectations.

The exact archive under test has SHA256
`609fe326edd8a1fc5c59383f2c1bf8a1660209df5b79103cef5c1946a980a8e4`.
The [fresh UI v5 test](../validation/horizon-native-preset-ui-v5.json) creates the
world through the translated selector with structures enabled, reaches a
healthy grounded survival spawn without a QA-supplied position, then saves and
reopens in a second process. The host's final spawn/player positions can differ
from the search center because it places a safe spawn and applies player spawn
spread. Full source player-placement/height parity is not claimed.

All 1,536 stored quart biomes in each tested origin and new distant FULL chunk
equal the native factory and differ from the vanilla factory. Both exits are
clean. The final views have 93 and 85 client chunks respectively, with 72 and
35 drawn terrain sections and empty sampled client terrain queues.
[Direct image review](../validation/horizon-native-preset-visual-v1.json) accepts
the selector, snowy survival spawn, origin forest and distant desert as rendering
smoke. The six-chunk horizon is visibly limited; this does not qualify scenery,
seams or completion of every server task.

The earlier UI v2/v4 data checks passed, but sparse origin/distant images caught
the first nine client chunks and are rejected as visual-quality evidence.
v3 applied a relative camera turn as an absolute angle and timed out with no
drawn terrain. Those reports remain frozen. v5 calculates the required relative
turn, waits for initial readiness, requires at least 81 loaded client chunks and
settles frames before capture. Original and copied prepared resource hashes
remain unchanged; no game files are downloaded.

Native installation/execution of all 17 classic variants and their pillar/marsh
features, source carver configuration, structure coverage, standard launcher
installation, native mod combinations and controlled native performance remain
open. The Java Fabric/NeoForge release qualifications are separate.
