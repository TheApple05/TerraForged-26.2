# Production density adapter validation

**Result: PASS, 235 assertions, process exit 0**, on 2026-11-03 America/Los_Angeles (2026-11-04 UTC), with Temurin Java 25.0.5+8 and Minecraft 26.2.

The test compiles the real production `ClampToNearestUnit`, `LinearSplineFunction`, and `NoiseUtil` classes against the Gradle-resolved game dependency classpath. It then calls the real `SharedConstants.tryDetectVersion()` and `Bootstrap.bootStrap()`, checks that the detected game version is exactly `26.2`, and runs without Fabric launch or replacement Minecraft classes. The actual game artifact name and SHA-256, production/test source hashes, and dependency artifact names are recorded in [density-initial.json](density-initial.json).

## Checks that ran

- Clamp results at negative, zero, positive, and gradient endpoint values match explicit inherited arithmetic expectations. Its reported min/max values are unchanged.
- Spline interpolation at interior points, exact knots, outside both ends, and a single-point spline matches explicit expected results. Bounds retain the inherited float-rounding behavior.
- Both adapters' `fillArray` results match scalar `compute` as raw double bits at 17 contexts, including an empty-array call.
- `mapChildren` maps only immediate children. The game's default `mapAll` visits one spline, four nested clamp nodes, and three distinct vanilla constant leaves exactly once each. Replacing those leaves reaches nested spline point values, changes the expected result, and recalculates bounds.
- Each actual record codec round-trips JSON with vanilla constant and gradient children; encoded data, scalar values, arrays, and bounds remain stable.
- An explicit non-default spline configuration parses, persists, and reloads with its point locations and values unchanged.
- Empty point lists and unsupported untyped point arrays are rejected.

The machine-readable report includes real encoded examples. The initial inherited point-codec failure is preserved separately in [density-legacy-codec.json](density-legacy-codec.json). The named point schema and traversal rationale are documented in [decision 0003](../decisions/0003-density-traversal.md).

## Reproduce

Resolve the target dependency classpath first if necessary. Do not run concurrent Gradle builds in the same workspace:

```powershell
$env:JAVA_HOME = 'C:\Users\Main\AppData\Local\Programs\Java\jdk-25.0.5+8'
.\gradlew.bat writePortClasspath
python tools/verify_density.py --java-home $env:JAVA_HOME
```

The wrapper refuses absent classpath artifacts. It compiles only the stated production slice into a temporary directory under ignored `build/qa/density/`, runs the probe against those freshly compiled classes and the actual dependency jars, and writes `build/qa/density/report.json`. `--report docs/validation/density-initial.json` selects the committed evidence path. Full local output is retained in ignored `build/qa/density/latest-output.log`.

## Scope

This establishes working adapters against actual 26.2 APIs. It does not verify custom density-type registry dispatch, loader launch, the rest of the source tree, chunk generation, packaged-artifact installation, visual parity, or performance. Those claims remain unestablished.
