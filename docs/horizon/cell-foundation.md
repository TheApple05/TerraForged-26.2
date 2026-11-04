# Cell values, height levels and tile layout

Native Levels and Size descriptors, and an owned 24-field Cell value, now match
two independent runs of the genuine published Java classes. Debug/release match
30,974 records and 93,027 fields per profile. The corpus includes signed32
extremes, negative chunk/block shifts, Java shift masking, wrapped dimension/index
arithmetic, multiple heights/sea levels, integer and float height conversions,
and explicit nonfinite classifications where the source arithmetic produces them.

Every Cell field is observed for default construction, two nonuniform fill
patterns, copy, source mutation after copy, and reset. This includes signed zeros,
subnormals and a quiet-NaN payload copied as raw bits. Terrain/biome are native
value identifiers owned by the future kernel registry; default NONE/GRASSLAND
and the filled FLATS/TAIGA values match their original IDs/ordinals. This does
not yet implement or qualify terrain inheritance, composite registrations or
biome climate lookup tables. Owned cells replace Java pool/resource ownership;
no tile buffer, generator, save or rendering is activated here.

The first oracle attempt correctly stopped before freezing a reference because
the reflection inventory included Cell's public static thread-local resource.
The probe now excludes static fields and verifies exactly 24 instance fields.
No expected file existed until two fresh corrected Java processes agreed.

Size deliberately remains a pure arithmetic descriptor, including odd/extreme
inputs. A future tile allocator must validate dimensions/indices before using
them as allocation sizes. This corpus does not make wrapped invalid descriptors
safe allocations or qualify neighbor sampling/filter execution.

The Levels/Size routine draft was DeepSeek job
`ff1f5508844143609af9cf0c932e08a8` (estimated USD 0.0017463). Codex removed
unused imports, reviewed the Java float promotion and wrapping/shift semantics,
integrated the code and verified every observation. This estimate is separate
from Codex usage.

Use `verify_native_noise.py --family cell --report <new-report.json>`.
