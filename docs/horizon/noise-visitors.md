# Original graph traversal semantics

The native map_all API now preserves the source mapAll traversal for every
registered numerical type, bound holder adapter and five domain types. A genuine
Java corpus records 48 cases under three transformations: identity, wrapping
every visited node in a small Add, and a stateful constant/seed transformation.
Two fresh production game JVMs agree on all 249,984 records. Native debug/release
match every callback trace, resulting root type, reported bound and compute value
(250,416 fields per profile).

The exceptional cases are preserved deliberately. Abs drops its own wrapper and
applies the visitor again to the visited child. Min, PowerCurve, AdvancedTerrace
and Erosion rebuild without applying the visitor to their parent. Line rebuilds
without visiting children or its parent. LinearSpline visits its input recursively
but applies the visitor directly to each point value. DomainWarp visits raw x/z,
already mapped x/z and distance separately, retaining that exact order instead of
remapping visited raw inputs. Reconstructed caches start with new state. Bound
holder traversal preserves its wrapper while replacing the underlying value.

The raw graph API still rejects unsupported construction limits. It does not
resolve missing/unbound or cyclic registry references, decode presets, execute
the cell/river/filtered-tile pipeline or activate native chunk generation.
Arbitrary user callback programs and all possible graph/input domains are not
certified by these three finite transformations.

The bounded DeepSeek extraction job `996a9a32e0d0403484940e1aa7f8751c` hit its
output limit and produced no accepted draft. It was not retried; the reserved
USD 0.0049608 estimate was retained. Codex performed the extraction and
implementation locally, then checked the actual Java traversal. This estimate
is separate from Codex usage.

Run `verify_native_noise.py --family visitors --report <new-report.json>`.
Expected values and traces originate only from the genuine packaged Java classes;
the native verifier neither regenerates nor overwrites them.
