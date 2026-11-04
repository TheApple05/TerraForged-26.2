# 26.2 compiler diagnostic

Recorded: 2026-11-03T23:19:46-08:00. Result: **compiler diagnostic passed**, exit code 0.

Command: `python tools/diagnose_port.py --report docs/validation/target-build-2026-11-03-compile.json`

Source files: 393. Captured errors: 0; javac reported errors: None; warnings: 70; affected files: 0.

Source changed during the run: False. A changed snapshot requires a new run after edits settle before using this as a coherent build result.

## Compiler categories

- No source-error categories captured; inspect the result classification and raw log.

## Evidence and limitations

The [JSON report](target-build-2026-11-03-compile.json) includes the actual resolved toolchain/classpath, source fingerprints, error headings and relative-path diagnostics. The raw log remains at `build/qa/target/compile-complete.log` (SHA-256 `4e3e8e07c38238e7e8951cbe095dfdf34ccc55550bb05ae936e9379ff2c1f272`). Gradle can stop before javac if configuration or dependency resolution fails; the result classification distinguishes that from source errors.

All common and Fabric Java sources are included. Reviewed 26.2 access widening is registered, annotation processing is disabled and platform services use explicit Fabric bindings. Packaging remains guarded. This diagnostic does not prove mixin application, runtime loading, world generation or feature parity; see [decision 0001](../decisions/0001-build-layout.md).

Error totals are a compiler-pass inventory. Resolving missing types can expose additional errors, so a smaller count is not a fixed success gate. Only an exit-zero compiler result establishes compilation within this deliberately limited diagnostic scope.
