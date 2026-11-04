# 26.2 compiler diagnostic

Recorded: 2026-11-03T23:18:01-08:00. Result: **compiler diagnostic failed**, exit code 1.

Command: `python tools/diagnose_port.py --report docs/validation/target-build-2026-11-03-integrated.json`

Source files: 393. Captured errors: 8; javac reported errors: 8; warnings: 70; affected files: 8.

Source changed during the run: False. A changed snapshot requires a new run after edits settle before using this as a coherent build result.

## Compiler categories

- terrain and world generation: 5.
- data generation and presets: 3.

## Evidence and limitations

The [JSON report](target-build-2026-11-03-integrated.json) includes the actual resolved toolchain/classpath, source fingerprints, error headings and relative-path diagnostics. The raw log remains at `build/qa/target/integrated.log` (SHA-256 `6963332187e4607af982e854813ed46e46a86175655b77b471fdbc98be306e8b`). Gradle can stop before javac if configuration or dependency resolution fails; the result classification distinguishes that from source errors.

All common and Fabric Java sources are included. Legacy access widening is unregistered, annotation processing is disabled and ExpectPlatform weaving is unresolved. Packaging remains guarded. This diagnostic does not prove mixin application, runtime loading, world generation or feature parity; see [decision 0001](../decisions/0001-build-layout.md).

Error totals are a compiler-pass inventory. Resolving missing types can expose additional errors, so a smaller count is not a fixed success gate. Only an exit-zero compiler result establishes compilation within this deliberately limited diagnostic scope.
