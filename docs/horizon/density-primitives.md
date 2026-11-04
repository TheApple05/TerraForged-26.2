# Custom density primitives before host binding

The native core now implements all ten CellSampler.Field conversions, including
the original continent remapping, ClampToNearestUnit's double/int/float arithmetic,
LinearSplineFunction's selection/interpolation/callback order and reported bounds,
ConditionalArrayCache's float storage/coordinate rules, and 2D NoiseFunction
sampling. These are numeric primitives; Horizon graph/router binding is separate.

Two fresh genuine packaged Java game JVMs agree on 45,328 records / 430,254 fields.
Native debug/release match all finite raw bits, integral/callback observations and
explicit NaN classifications. The corpus includes 42,120 ten-field records across
three settings contexts and all18 built-in terrains, 13 height and15 edge inputs,
four beach offsets, degenerate/fallback control points, signed zero, infinities and
NaNs. It also covers 280 clamp cases, 336 spline queries, 819 conditional cache
initialization callbacks, 1,728 cache queries, 12 forwarded bounds and 30 samples
of the two actual resolved ground/biome-edge noise bindings.

Source semantics are retained. The clamp truncates double×resolution to int,
adds one with int wrapping, converts to float, divides as float and widens to
double; its name does not describe conventional nearest rounding. Resolution0
and signed extremes are observed. Spline bounds narrow child double extrema
through float. Duplicate locations and one-point tables are covered, as are
endpoint selection and callback order. Source NaN spline input throws before
evaluating child points; native code rejects it explicitly. Conditional caches
initialize at Y0 in X-then-Z order, store float values, ignore queried Y for cached
positions, wrap integer coordinate arithmetic, and preserve double fallback
results outside the cached area. Width0 is supported. Failed native initialization
returns no partially constructed cache.

Native spline tables must be finite, nonempty, nondecreasing and at most256
points. Conditional widths are 0..256. Unsupported tables/widths and unknown field
names/owned terrain IDs reject explicitly. These are bounded supported inputs;
the Java codec's entire invalid/unsorted-domain behavior is not claimed equivalent.
The independent Java harness uses controlled leaf inputs and callback traces,
while the actual original mod classes perform the operations under comparison.
Original class origins include every anonymous CellSampler.Field implementation.
The freezer validates separate processes, predefined input bytes, class origins,
raw hashes and corpus counts. Expected values never come from native output.

Legacy reported min/max values are metadata, not proven optimization bounds.
For example, CellSampler advertises 0..1 although continent values can be negative.
A future Horizon graph binding must use conservative valid intervals and preserve
the supported float/double boundaries; blindly trusting those source bounds could
change the graph's evaluation. Marker visitors, accurate/approximate caller modes,
host density sampling errors, router conversion, biome/surface integration and
actual chunks remain unfinished.

Run `make_density_primitive_inputs.py` to check/reproduce predefined inputs,
`freeze_density_primitive_reference.py` only with two genuine Java captures, then
`verify_native_registry.py --family density-primitives --report <new-report.json>`.
The input builder checks existing bytes; freezers/verifiers refuse replacement.
Windows/Linux CI includes this family. Released Fabric/NeoForge JARs are unchanged.
