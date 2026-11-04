# NeoForge dev8 validation

Target: Minecraft Java 26.2, Java 25, NeoForge 26.2.0.93. These are experimental candidates, not a completed parity or general modpack certification.

- NeoForge: `neoforge/build/libs/terraforged-revival-0.1.0-dev.8+26.2-neoforge.jar`, SHA-256 `5031e101f82f9076c198c97f765f8bb5c87799ff5e9ce2a98ff43d17a9ea3878`.
- Fabric: `build/libs/terraforged-revival-0.1.0-dev.8+26.2.jar`, SHA-256 `ec4cb94cf0dd3da55ca543bfee6eb145a61bb3310b02db7cc08d29f2f58d8585`.

[Coherent package report](dual-loader-dev8-build.json): both compile/package exits 0, no source exclusions, source/configuration snapshots unchanged during the build, one matching platform provider per JAR, 277 translations, license notices, and no packaged QA classes. NeoForge includes all 382 common plus 8 loader-specific Java files.

## Production runtime results

- **Dedicated custom generation and reload passed:** exact NeoForge dev8 artifact, seed 8675309, active preset/context/tile cache, six FULL chunks and 54 sampled columns, ordinary 60-second watchdog, flushed save and clean exit 0. [Generation](neoforge-dev8-generation.json) and [reload](neoforge-dev8-reload.json) retain digest `5fab505bc818ab96a66e2a65d644e5671bcc9029f1922af372df694d8fc00db1`. The same bounded sample digest was observed on Fabric; this does not prove complete block determinism.
- **Export passed in the real dev8 client:** actual Done callback exports the preset and reloads it into the real CreateWorld context. The separate dedicated-server export inventory was run on dev7: [1,938 exported files](neoforge-dev7-export.json). Its pack is also consumed by the dev8 server checks; a dev8 command-line export inventory is not claimed here.
- **Client flow and visual smoke passed:** [exact dev8 production client report](neoforge-client-dev8-fullflow.json), packaged origin/hash verified, exit 0, no critical runtime errors. Actual Font/inner-width checks confirm all four shortened labels fit. Real slider events update a 65,536-cell preview, restoring defaults reproduces its initial digest, and the actual Create New World callback creates seed 123456789 with an active custom generator. After saving, the same process loads vanilla seed 8675309 without a custom context. Native modifier registry counts are 22 and 0 respectively; feature-holder consistency checks cover 2,702 and 2,587 references. All seven [game-rendered screenshots](images/neoforge-client-dev8/) were inspected and retained unchanged.
- **Optional TerraBlender server checks passed:** [generation](neoforge-dev8-terrablender-generation.json) and [reload](neoforge-dev8-terrablender-reload.json) record the exact stack and finite uniqueness assertion. Content-mod stack results are separate reports.

## Platform scope

These local client and dedicated-server executions ran on **Windows**, using Temurin 25.0.5+8. GitHub's separate [packaging workflow](../../.github/workflows/package-candidates.yml) compiles/packages both loaders on Ubuntu; its result is a build gate and does not establish Linux game runtime or visual acceptance. The [utility regression workflow](../../.github/workflows/core-regression.yml) covers its scoped kernel/cache checks on Linux and Windows. Read the recorded CI run status separately; workflow presence is not proof of a successful run.

Not established: full classic feature parity, every preset/control extreme, original-version visual acceptance, complete block determinism, multiplayer clients, Linux game runtime, arbitrary modpack compatibility or measured generation-performance acceptance. [Harness/reproduction details](neoforge-runtime-harness.md) and [feature gaps](../FEATURE-PARITY.md) remain explicit.
