# Native classic biome resource derivation

`horizon/native-adapter/src/classic_biome_data.rs` derives 17 definitions from
installed parent resources. It contains the project's MIT classic variant policy;
it does not embed Minecraft biome or feature JSON. It takes resolved biome and
placed-feature tag memberships, plus parent biome/configured/placed bundles.

The transformer retains parent fields and inherited carvers, copies the source
climate changes, uses savanna ambience for steppe, and inherits warm beach's tags
from beach rather than warm ocean. Tree-removing variants walk referenced feature
compositions, including flat 26.3 configured features, wrapped configurations,
inline placed holders, selectors and resolved placed-feature tags. Traversal is
bounded by depth and visit budgets, memoizes named holders and rejects cycles
and missing references. Previous TerraForged modifiers are removed before
looking up their implementations. Scrub, rock and mushroom removal follows the
explicit variant policy. Plant aliases retain their installed source and append
fixed count 1, preserving the distinct feature identities used by ordering.

The result includes eight placed-feature definitions, three configured-feature
definitions and inherited tag additions. The configured definitions **require**
native classic-pillar and swamp-surface implementations. Returning their JSON
is not evidence that those algorithms execute. This module is not yet installed
in the native pipeline, client registry or renderer.

## Verification

Six synthetic Rust tests cover retained parent properties, climate/ambience,
tag inheritance, aliases, nested tree filtering in both schemas, inline holder
resolution, stale modifier removal, missing parents, incompatible step counts,
missing feature references and cycles. The adapter library and the new probe
pass Clippy with warnings denied. An exploratory all-targets Clippy invocation
also found existing warnings in the unrelated terrain_factory_probe and
tile_filter_probe binaries; it is not recorded as an all-targets pass.
CI now runs the adapter unit tests and targeted lint on Windows and Linux.

The [installed-resource smoke check](../validation/horizon-native-classic-biome-data-v5.json)
uses five hashed resource bundles from the player's existing prepared 26.3
installation. Debug and release independently produce byte-identical output:
`945cb1a4d0529a126a14c3ca8fb175f752346665432112bbf472928cd40ff1e1`.
Both produce 17 biomes, eight placed features, three configured features and
membership additions in 44 tags. Checked inputs are unchanged. Raw Minecraft
inputs and derived resource JSON remain in ignored local QA storage; only
hashes, counts and scoped evidence are committed.

The [first preflight failure](../validation/horizon-native-classic-biome-data-v1.json)
exposed the QA resolver treating a configured-only feature tag as a placed tag.
The resolver now invalidates that unrelated tag independently and records it;
using a missing tag in a derived feature still fails. The [second failure](../validation/horizon-native-classic-biome-data-v2.json)
exposed an inline placed holder being interpreted as a selector entry. A
regression test now distinguishes that holder's configured-feature reference.
The later passing reports are retained without replacing either failure.

These are structural tests and a real-input transformation smoke check, not
independent Java definition parity. Native registry/renderer installation,
world-pack priority, actual custom-feature execution, natural biome coverage,
stored chunks and controlled native performance remain integration acceptance.
The separate [selection-policy corpus](classic-biome-policy.md) already compares
154,936 observations against the published Java implementation.
