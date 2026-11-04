# Cell preset model and noise factories

The core now has typed world, terrain, climate, river and filter settings. The
adapter reads those five groups with their source optional defaults while retaining
the original structured document for other sections. Cave, structure and
miscellaneous execution/validation are outside this cell-only adapter's scope.
Decoding a continent enum does not yet implement that continent generator.

Two fresh genuine production Java JVMs agree on 85,630 records / 85,832 fields.
Native debug/release match five setting cases, 118 typed fields per case, derived
control points/range helpers and 49 factory graphs. Cases include omitted optional
fields, explicit alternatives, invalid control points, reversed/clamped ranges and
extreme height integers. Required sections, unknown continent/spawn/edge enums and
source octave limits have explicit rejection checks.

The factory layer preserves source scale conversion, interpolation, gains and
seed wrapping. Perlin families retain stored seeds; other bases use ShiftSeed
wrappers. The source map factory's endpoint shortcut is separate from the raw Map
record, including the constant-range case. Warp helper seed+1 wraps at signed32.
All 13 climate edge factories are checked; SIMPLEX_RIDGE deliberately constructs
Simplex2 for compatibility, and SIN/RAND keep their source scale behavior.

ControlPoints.make retains the source's validation and fallback to legacy default
control points. The validity flag and resulting fields are both observed. Climate
range min/max/bias helpers preserve source clamping and operation order. These
are constructor/helper checks, not evidence of continent, river, tile or chunk
generation.

Routine DTO boilerplate used DeepSeek job `21a4a01189d147fbb6750527a3a39fb7`
(estimated USD 0.0043764). Codex removed an unused guessed SpawnType enum,
verified actual source fields/serialized aliases, implemented the adapters and
algorithms, and checked the genuine Java observations. The estimate is separate
from Codex usage. No production Java source or released JAR bytes changed.

Run `verify_native_registry.py --family preset --report <new-report.json>`.
The immutable input cases are separate from observations, and expected values
are captured only from the genuine Java runtime.
