# Cellular, legacy climate, spline and cache observations

Worley, WorleyEdge, LegacyTemperature, LegacyMoisture, LinearSpline and the
deprecated Cache2d wrapper now match 158 genuine Java constructor graphs plus
18 explicit cache-sequence observations. The two fresh production game JVMs
agree on all 274,148 records. Native debug/release match all 274,464 fields.

The corpus covers every distance/cell/edge function, five jitter values including
zero and negative jitter, signed seeds, world-border coordinates after frequency
scaling, eight legacy power counts, both frequency signs, spline endpoints,
singleton and duplicate knots, and dynamic spline values. The undefined zero
jitter / Distance2Div coincidence is excluded from this finite raw-bit corpus.
Source DISTANCE deliberately maps to zero; NOISE_LOOKUP returns its lookup value
without remapping, even when that value exceeds the reported 0..1 interval.

The deprecated cache retains its exact legacy key and sentinel behavior: raw
coordinate bits, no seed in the key, and initial Long.MIN_VALUE position with a
zero value. Six-call sequences on the main thread and two fresh threads check
seed changes at equal coordinates, initial signed-zero sentinel, and independent
thread state. Native mutable entries belong to the graph and are released when
its last owner drops; they do not live in a process-wide strong-owner cache.
The small mutex-protected per-thread map has not yet been profiled in the native
terrain pipeline. These semantics must not be mistaken for a general seed-aware
cache suitable for unrelated worlds.

Spline constructors explicitly reject empty, nonfinite or descending knots;
equal knots remain supported. Legacy powers above 1024 return a resource-limit
error. The probe checks those guards. Arbitrary invalid/nonfinite compute inputs
and every parameter domain remain unqualified. Visitors, registry decoding and
the cell/river/filtered-tile kernel are still unfinished; no native game generation
is activated by this numerical work.

Worley geometry boilerplate used the bounded DeepSeek draft job
`d13f20d4624a403a9f60355833d41d79` (estimated USD 0.0027708). Codex reviewed the
source operation order, wrapping neighbor coordinates, strict nearest ties and
explicit missing-cell result, integrated graph behavior, and verified it against
the genuine Java captures. The estimate is separate from Codex usage.
