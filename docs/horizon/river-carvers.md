# River, lake and wetland carving

The owned native core now implements the original RiverCarver, Lake, Wetland and
Boundsf arithmetic. Two fresh genuine packaged Java game JVMs agree on 169,629
records / 3,919,986 fields. Both Rust debug and release match every field: finite
float bits, integer and boolean values, plus explicit NaN classifications.

The predefined corpus includes 48,600 river, 34,992 lake and 78,732 wetland cell
mutations. Every mutation observes all 24 Cell fields, including incoming dirty
fields that must remain untouched. It exercises three height/sea-level pairs,
degenerate and negative geometry, normal and overflowing integer configurations,
four valley/fade/curve settings, nine line parameters including NaN, six input
terrain/height/mask cases, lake geometry predicates, and wetland noise sampling.
There are 810 fork configurations, 384 valley-curve random checkpoints, 18 carver
construction checkpoints and 24 independent Math.round(double) boundaries.

Original behavior is retained, including the empty connecting block, sequential
lake mask multiplication/minimum, river terrain override rules, wetland pooling
and mound order, and Boundsf.Builder's positive Float.MIN_VALUE initial maxima.
Fork widths use the original double sqrt/multiply/round followed by narrowing.
NaN payloads are classified; finite raw-bit equality is the stronger claim.

Expected values came only from the production Java JAR; they were never generated
from Rust. Producer reports are server-horizon-river-carvers-oracle-v1a/b.json.
The freezer checks genuine loaded class origins, process identity, predefined
input bytes and both captures before writing the immutable compressed fixture.
Run `verify_native_registry.py --family river-carvers --report <new-report.json>`.
The Windows/Linux CI matrix includes this comparison.

The Lake translation began with bounded DeepSeek job
`34706819915245ad9de622210ef09b3e` (estimated USD 0.0017493, separate from Codex).
Codex corrected the missing import and shadowed numeric/object variable, reviewed
the complete source, implemented the other carvers and performed all independent
capture and verification. No released Java JAR changed.

River network construction, network traversal, caching and complete Heightmap
apply remain required. This qualification does not activate a native Horizon
generator or establish filtered tiles, native chunks, clients or save support.
