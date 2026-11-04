# Initial 26.2 compilation diagnostic

Date: 2026-11-03. Result: **source compilation failed**, as expected for the unadapted 1.20.1 baseline. Dependency resolution succeeded. This is the first measured migration inventory, not an installable port.

## Reproduce

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File tools/gradle.ps1 portToolchain compileJava --console=plain
```

The invocation used Gradle 9.5.1, Fabric Loom 1.17.21 and Temurin Java/Javac 25.0.5+8. Minecraft 26.2, Fabric Loader 0.20.1, Fabric API 0.161.0+26.2 and all compile-only dependencies resolved. The actual dependency inventory is included in the [machine-readable report](target-build-2026-11-03-initial.json).

Javac examined the complete configured source roots: **388 Java files** under common and Fabric, without source exclusions. It reported **432 errors and 70 warnings**, with errors in **103 distinct files**. The run ended with exit code 1 and `BUILD FAILED in 42s`.

The report captures each compiler error once, using the original javac output before Gradle repeats it in the failure summary. It includes relative file paths, source lines, diagnostic details, source fingerprint, baseline commit and local raw-log hash. Source edits were held until this inventory was frozen. This report continues to describe that initial snapshot even as subsequent migration commits change the sources.

## Observed distribution

- Data generation and presets: 204 errors.
- Terrain and world generation outside density functions: 111.
- Client UI and preview: 61.
- Fabric integration: 29.
- Density functions: 12.
- Common Minecraft/optional integration mixins: 10.
- Registries, configuration and common bootstrap: 4.
- Common platform dispatch: 1.

There are 289 `cannot find symbol` diagnostics. Recurring concrete issues include renamed resource identifiers and utility classes, moved chunk classes, the new GUI extraction API, changed registry/data-generation signatures, `MapCodec` dispatch requirements, changed NBT optional values, private coordinate fields, and removed block-state overloads. Missing or inaccessible type errors can conceal further method errors; the count is an inventory for this compiler pass, not an estimate of independent defects or remaining effort.

The densest individual files were preset template paths (32 errors), preset configured features (29), common configured features (25), and the preset editor (21). Error totals alone do not establish feature priority: terrain fidelity and correct lifecycle integration remain acceptance gates.

## Deliberate diagnostic limitations

As recorded in [decision 0001](../decisions/0001-build-layout.md), legacy access widening is unregistered, Mixin annotation processing is disabled and `@ExpectPlatform` weaving is unresolved. The complete Java sources are present. Some access diagnostics therefore represent the absent migration of access rules, rather than removed game functionality. No annotation-processing success, required-mixin application, runtime boot, world generation or visual parity is claimed.

The JAR task has an explicit packaging guard. Historical Fabric metadata is still unported. No mod artifact from this configuration should be installed as a playable port.

## Additional setup verification

- `writePortClasspath` succeeded and writes the actual full compile classpath to ignored `build/qa/target/compile-classpath.txt` for isolated production-adapter tests.
- The wrapper was regenerated using Gradle 9.5.1; a subsequent clean `tools/gradle.ps1 --version` returned exit 0 with the expected Gradle and JDK versions.
- The distribution SHA-256 is pinned to Gradle's published `bafc141b619ad6350fd975fc903156dd5c151998cc8b058e8c1044ab5f7b031f`. Wrapper JAR verification is recorded separately in the machine-readable report.

Next: adapt shared resource/codec APIs and explicit Fabric platform dispatch, then repeat a full-source compiler pass. Restore the appropriate class access and validate required mixins before attempting packaged runtime tests.
