# 0001: Direct Fabric build for the first 26.2 diagnostic

Date: 2026-11-03. Status: full source compilation passes; packaged runtime validation in progress.

The source baseline is the imported ReTerraForged 1.20.1 tree. Preserve its common and Fabric source directories, and compile all of them in a single modern Fabric Loom root project. Keep the historical `common/build.gradle`, `fabric/build.gradle` and `forge/build.gradle` as reference; none is included by the new `settings.gradle`.

This reduces the simultaneous build-system migration surface. The old Architectury plugin, shadow/remapping pipeline, Java 17 and snapshot dependencies do not describe the unobfuscated Java 25 game. The first target uses Minecraft 26.2, Loom 1.17.21, Gradle 9.5.1, Loader 0.20.1 and Fabric API 0.161.0+26.2. See [toolchain research](../research/target-toolchain.md) for primary sources. A second loader remains future work.

## Diagnostic scope

- Every Java file in `common/src/main/java` and `fabric/src/main/java` is compiled, including UI, data generation and optional integration code. No Java source exclusions, stubs or substitute entrypoint are used.
- TerraBlender's actual Fabric 26.2 artifact `26.2-26.2.0.0.2`, Architectury injectables `1.0.10` and Night Config TOML `3.6.7` are compile-only dependencies. They are not claimed as tested runtime integrations.
- The historical access widener is **not registered**. It refers to obsolete classes/descriptors that could abort Loom before compilation. Access errors remain in the diagnostic until each access rule is migrated against real 26.2 signatures.
- Annotation processing is disabled with `-proc:none` for this first compiler inventory. This is not a Mixin audit.
- `@ExpectPlatform` annotations compile against their real annotation library, but no Architectury weaving occurs. Existing platform methods still throw until migrated to explicit dispatch or a verified transformation pipeline.
- Original mod metadata and mixin resources are retained unchanged except normal version expansion. Their target/version declarations and hooks are not validated.
- The JAR task deliberately fails with an explanation, preventing a compiler-only milestone from producing a misleading installable artifact.

## Reproduction

On Windows, use `powershell -ExecutionPolicy Bypass -File tools/gradle.ps1 portToolchain compileJava --console=plain`. The helper selects a full Java 25 JDK from `-JavaHome`, `TF_JAVA_HOME`, an appropriate `JAVA_HOME`, or local Java installations. Environment changes apply only to that invocation and are restored; global Java configuration is untouched.

`portToolchain` prints machine-readable actual Gradle/JDK versions, all source roots, source count and resolved classpath. Javac is configured with a 10,000-error ceiling to expose the migration surface in one run. Compiler errors are evidence of unfinished source adaptation, not an acceptable release state. The curated results belong in `docs/validation/target-build-*.json` and `.md`; local raw logs remain ignored.

For repeatable capture, run `python tools/diagnose_port.py --report docs/validation/target-build-NAME.json`. This invokes the full diagnostic, saves its raw log under ignored `build/qa/target`, writes a JSON report and Markdown summary, and returns the compiler invocation's actual exit code. `--java-home` selects an explicit JDK and `--log` chooses a distinct raw-log path. The report fingerprints source files before and after execution so concurrent edits are visible. It distinguishes dependency/configuration failures from source compilation errors and does not treat a lower error count as success.

## Gates before packaging

Migrate source APIs, explicit platform dispatch, access widening, metadata, required mixin descriptors and all necessary resources; enable annotation processing or equivalent runtime audit; then remove the packaging guard only when a real candidate can be smoke-tested. A successful compiler invocation alone does not establish playable world generation or feature parity.

## Subsequent implementation stages

The initial limitations above describe the first inventory snapshot. The full 393-file source pass now succeeds. Platform bridges use explicit Fabric implementations, metadata declares the target, and a new `terraforged-26.2.accesswidener` contains only reviewed target members. Modern non-obfuscated Loom requires its `official` namespace; a first `named` attempt was rejected, corrected, and resolved successfully. The obsolete imported widener remains reference-only. Mixin annotation processing is still disabled; actual packaged runtime mixin application is a separate gate.

The default JAR task guard now permits `-PexperimentalPackage` for isolated production validation. Experimental JARs include upstream/component license notices. This flag does not establish release readiness. Runtime tests must use this exact packaged artifact, its declared dependencies, and a disposable installation; no generated Minecraft source is committed.
