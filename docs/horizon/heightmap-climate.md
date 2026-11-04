# Complete terrain and climate stages

The core now constructs the original Heightmap terrain and climate components in
the original seed order. Region borders, weighted terrain, mountain-chain blend,
advanced continent, climate, oceans and beach noise are assembled together.
Climate includes warped regions, legacy moisture/temperature, category selection,
altitude adjustment, coast conversion, edge resampling and source field ordering.
Heightmap climate preprocessing preserves river/lake/wetland rules and the final
macro-biome weirdness sign flip, including signed zero.

This deliberately exposes only apply_terrain and apply_climate. There is no full
apply sampler or fake/no-op river stage. River networks, filtered tiles and native
chunk execution remain unfinished. The constructor currently accepts only
MULTI_IMPROVED, with explicit positive/derived-scale and graph-budget limits.
Unsupported continent types or zero derived warp scales return errors.

Two fresh packaged Java game JVMs agree on 173,010 records / 4,151,610 fields.
Both native profiles match exactly across five presets, six world-seed cases,
23 by 23 coordinates out to +/-30 million, fresh/dirty Cells and both climate
modes. All 172,980 Cell snapshots include every field; 77,760 synthetic snapshots
exercise terrain, altitude, water, river-mask and macro-biome boundaries. Actual
registry bindings, all 21 main constructor seed draws and the climate random
seed are observed. The original upper32bit seed-narrowing pair agrees. Native
reverse order, four parallel readers, unsupported type, scale, coordinate and
owned-terrain rejection guards pass.

The first capture pair failed a QA binding comparison: Gson compared a boxed
Float's widened double value with the parsed decimal's double value. The guard
now compares the actual serialized JSON binding after parsing, preserving its
canonical float literals. Both failed reports and corrected captures are retained.
No numerical fixture came from failed runs or native output.

As a separate check, native terrain matches all 726 terrain records / 17,424 fields
from the earlier kernel-v1 full Java corpus, in both profiles. That native binary
reads only preset, noise registry and coordinate inputs; it never reads expected
cells or prepared terrain. Other stages in that older corpus are not claimed as
matched. The archive's original nativeImplementationPassed=false flag is preserved.
The first cross-check report passed with an unused QA helper warning; the second
uses the shared default-cell helper and passes without warnings.

Run `verify_native_registry.py --family heightmap-climate` and
`verify_kernel_terrain.py` with new report paths. Inputs, genuine class origins,
independent producer reports and losslessly compressed expected data are pinned.
Java production source and published artifacts remain unchanged. Full river/tile
parity, other continent generators, arbitrary preset domains and native game
integration are still required acceptance work.
