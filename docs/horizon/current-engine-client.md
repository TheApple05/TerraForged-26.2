# Current Horizon engine and client evidence

The current integration advances to the [world selector API and real creation
checks](native-world-selector.md) at commit `61680d714bad3496a0ca5cef31537166f1232c4b`.
The following source/client checks describe the earlier integration pin
`0e0f70ff1f8191479dbb5b1b173a74d38d324005` (merged in that fork by PR6).
It includes the fork's existing Photon and compatibility changes. Those changes
are inherited engine work; this port does not establish Photon pack compatibility.
The earlier E4 package and its reader evidence remain historical artifacts.
No package for this newer pin has been qualified for standard launcher install.

The new host API supports a world-owned biome mapper before decoration and
structure catalogs are built. It validates additional biome IDs and resource
presence, keeps delegate feature encounter order and isolates mapped decoration
from vanilla source-JSON caches. This is infrastructure for native classic
biomes, not evidence that their native definitions or features are installed.
The pure selection policy has a separate independent Java comparison.

## Source compatibility

The [current registry check](../validation/horizon-native-current-registry-v1.json)
compares 24,768 independently captured Java records in debug and release with
zero differences. Registry lifecycle, scope restoration, thread isolation and
owner-release checks pass. The [constructor check](../validation/horizon-native-current-world-entry-v1.json)
runs two fresh optimized processes. Both produce observation digest
`f03f0280ed76cdfddddd31b7356521087b53211abafa6cda5446b3f9c16a6347`,
also produced on the previous engine pin. This checks the documented constructor
cases, not arbitrary world equivalence.

The forwarding generator now includes 24 production methods and the engine's
new test-only `may_start_answers` method. It preserves that method's `cfg(test)`
attribute and rejects unexpected API drift. [Warnings-denied lint and test
configuration compilation](../validation/horizon-native-current-entry-lint-v1.json)
pass. The filtered compile executes zero tests; it must not be reported as
349 passing tests.

## Actual client and saved chunks

`horizon/tools/verify_native_client.py` builds and hashes the actual source-built
client, control tool and fixture reader. Each run copies the player's own
prepared 26.3 resources into fresh isolated homes. It neither downloads game
files nor writes caches into the player's resource directory. The report checks
both original and copied manifest, facts and world-generation JSON bytes.

The fixture uses Horizon's real level/settings writers and supplies a native
saved stem, spectator spawn and disabled structures. Two fresh client processes
open the same save. The second teleports to a previously absent distant chunk.
The reader opens real FULL region chunks and compares all 1,536 stored quart
biomes per tested chunk with the actual native factory. It also requires each
expected biome vector to differ from the vanilla factory's vector. Equality is
checked on the vectors, not just their diagnostic hashes.

The [v3 run](../validation/horizon-native-client-save-v3.json) passes both
clean exits, cold reopen, original and new distant chunk checks. Its two-chunk
render distance is a rendering smoke check: the images visibly clip terrain
at short chunk boundaries. They cannot qualify scenery quality or seams.
The [v2 failure](../validation/horizon-native-client-save-v2.json) preserves an
early teleport/readiness race which saved before the distant chunk existed.
[v4](../validation/horizon-native-client-save-v4.json) exposes the harness's
one-second `wait-ready` timeout. [v5](../validation/horizon-native-client-save-v5.json)
uses direct status polling and a wider six-chunk view, but times out waiting
for its fixed minimum at the distant location. The wider first image shows
forest terrain; incomplete view loading is not accepted as a performance pass.
These failed reports remain immutable. The corrected [v6 run](../validation/horizon-native-client-save-v6.json)
passes both clean exits, cold reopen and complete stored-biome-vector checks at
both locations. It freezes the verifier source and records its hash alongside
the exact executable and fixture-reader hashes. The six-chunk images show
forest and desert terrain; [direct image review](../validation/horizon-native-client-visual-v1.json)
accepts rendering smoke only. Client terrain queues are empty at the sampled
readiness points; limited-distance silhouettes are visible, and this does not
prove all server generation tasks or terrain-fidelity acceptance.

The subsequent UI checks cover ordinary creation and a healthy natural survival
spawn, while source spawn-algorithm parity remains open. Structures, all 17 native classic biome
resources and their custom features, source carver configuration, standard
launcher installation, controlled native performance and mod combinations
remain open. These native checks do not change the Java 26.2 Fabric/NeoForge
dev23 release or its separate acceptance results.
