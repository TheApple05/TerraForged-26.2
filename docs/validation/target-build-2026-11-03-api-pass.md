# 26.2 compiler diagnostic

Recorded: 2026-11-03T23:04:44-08:00. Result: **compiler diagnostic failed**, exit code 1.

Command: `python tools/diagnose_port.py --report docs/validation/target-build-2026-11-03-api-pass.json`

Source files: 388. Captured errors: 251; javac reported errors: 251; warnings: 69; affected files: 67.

Source changed during the run: False. A changed snapshot requires a new run after edits settle before using this as a coherent build result.

## Compiler categories

- terrain and world generation: 87.
- data generation and presets: 78.
- client UI and preview: 55.
- fabric integration: 20.
- density functions: 7.
- Minecraft and optional integration mixins: 4.

## Evidence and limitations

The [JSON report](target-build-2026-11-03-api-pass.json) includes the actual resolved toolchain/classpath, source fingerprints, error headings and relative-path diagnostics. The raw log remains at `build/qa/target/api-pass.log` (SHA-256 `f39b76a8a56c966f1b38e5b468cbb276808d5b0819475e4156f4f219932204b4`). Gradle can stop before javac if configuration or dependency resolution fails; the result classification distinguishes that from source errors.

All common and Fabric Java sources are included. Legacy access widening is unregistered, annotation processing is disabled and ExpectPlatform weaving is unresolved. Packaging remains guarded. This diagnostic does not prove mixin application, runtime loading, world generation or feature parity; see [decision 0001](../decisions/0001-build-layout.md).

Error totals are a compiler-pass inventory. Resolving missing types can expose additional errors, so a smaller count is not a fixed success gate. Only an exit-zero compiler result establishes compilation within this deliberately limited diagnostic scope.
