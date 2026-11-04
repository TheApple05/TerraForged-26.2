# Scoped Horizon resource decoding

The three density bindings qualified in host-density-graph.md now also work
through Horizon's actual JSON `DensityRegistry`. A cancellable HEAD hook handles
`reterraforged:cell`, `reterraforged:noise` and
`reterraforged:clamp_to_nearest_unit` while an explicit registry state is active.
Vanilla resources continue through the original decoder.

The state owns its kernel and bounded resolved noise registry. Its world seed is
narrowed to i32 for source noise functions, matching `MixinRandomState`'s marker
visitor. Separately compiled clamp children keep their own cache namespace.
The native sampler cache retains its normal eight-tile default; the fixed oracle
working set explicitly requests the supported capacity of 32.

Registry activation is synchronous and thread-local. A private guard restores
the previous context on normal return, decoder failure and panic. Nesting is
bounded to eight, and a new thread inherits no active state. After decoding,
compiled samplers hold their own Arc owners and require no active thread-local
context. Dropping their graphs releases those owners.

`horizon-native-host-registry-v1.json` records exact agreement with the existing
independent Java capture for all 24,768 scalar/bulk values in both debug and
release. Those roots are constructed as JSON resources, resolved by the real
Horizon registry, and then compiled. The addition resource uses the 26.3
`left`/`right` schema; this is an explicit small adaptation, not a general
26.2 resource converter.

The same executable checks nested restoration, the eight-scope limit, thread
isolation, panic cleanup, recovery after invalid-field errors, rejection of
custom resources outside a scope, unchanged vanilla constant decoding and
release of the final kernel owner. Controlled panic messages in its stderr are
expected observations from `catch_unwind`, followed by the success marker.
The assembled `horizon-world` library passes Clippy with warnings denied.

Use `--registry` with both `stage_native_host.py` and `verify_host_density.py` to
repeat this qualification. CI exercises graph-only and registry modes separately.
An initial source check needed an explicit `Result<(), String>` type in the
thread-local initializer; the tested stage includes that fix.

This does not activate a world. Only named custom noise holders are supported
by this decoder; inline holders, custom linear splines and conditional-array
cache integration remain explicit unsupported inputs. The complete router,
world selection, biomes, surfaces, caves/features, chunks, client and save/reload
still need integration and acceptance.
