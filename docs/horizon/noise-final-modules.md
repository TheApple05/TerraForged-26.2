# Line, terrace, domain and erosion arithmetic

The remaining five registered numerical types now match their independently
captured Java observations: Line, Terrace, AdvancedTerrace, Warp and Erosion.
This completes finite numerical fixtures for all 41 registered noise types,
plus all five domain types and both registered curve-function types. It does
not complete visitors, registry decoding or the native terrain pipeline.

Two fresh production Fabric JVMs agree on 151,228 records. Native debug/release
match all 198,780 fields: 78 constructor graphs and their reported bounds, nine
domain compositions with separate offset/transformed coordinate observations,
line distance/intersection helpers, and three explicit NaN classifications.
The NaN cases compare classification rather than promising a portable payload:
both line endpoints and zero-octave erosion inherit that source behavior.

Cases cover 2/4/8/16 terrace steps, four blend ranges, dynamic ramp/cliff/height,
nonzero source bounds, advanced-terrace minimum/mask early exits and 0/1/3
octaves, direct/directional/additive/compound domains, and every erosion blend
mode with three stored seeds and 1/3 octaves. Erosion retains the original seed0
neighbor sampling and -1 scratch sentinel; its 25-float scratch belongs to each
native invocation. No thread-local source ownership or external assets are added.

Native guards reject terrace counts outside 2..4096, advanced counts outside
1..4096 and octave counts outside 0..32. Combined domain/noise graph nesting
shares the 256-depth and one-million-expanded-node limits, preventing domains
from bypassing noise construction limits. The probe observes relevant rejects.
These bounds deliberately reject some raw Java inputs; arbitrary degenerate,
nonfinite and out-of-domain behavior remains unqualified.

The Erosion boilerplate draft was DeepSeek job
`1e5ff967158c42d09145ad106d570ca9` (estimated USD 0.0028443). Codex added missing
Clone/Debug derivation, reviewed all operation order and seed use, integrated it,
and verified the output against the real Java captures. This estimate is separate
from Codex usage. Every native expected value still comes from genuine Java.

Run `verify_native_noise.py --family final --report <new-report.json>`. CI runs
all prior numerical families too. The production Java dev23 release and finite
prepared Horizon package remain unchanged; this crate is still isolated.
