# Production NeoForge runtime checks

`tools/qa/production_neoforge_server.py` installs the official NeoForge 26.2.0.93 server in `build/qa/neoforge-server`. It verifies the published installer SHA-256 and Mojang's official 26.2 server SHA-1, then copies that installation into a fresh disposable instance. Servers bind only to `127.0.0.1`, use an ephemeral port and offline QA identity, and never open an existing world. The ordinary 60-second watchdog remains enabled. The harness records the exact installed mod SHA-256, observed class origin, game version, startup, save, shutdown and process exit.

`tools/qa/build_neoforge_runtime_qa.py` compiles a separate test companion against the packaged production JAR and real NeoForge classpath. It adapts the shared Fabric runtime probe's event entrypoint and provenance to NeoForge. The preset export, generator-active gate, six FULL chunk requests, 54 column samples, finite terrain fields, digest and save/reload checks are shared. Java's module system rejects split packages, so the biome decorator probe is relocated to the test module and invokes the same package-private production helper reflectively. Source/adapter hashes and these changes are recorded in the companion build report. QA classes are never included in the production mod.

The first companion failed module resolution before game startup because of a split package; [`neoforge-dev5-export.json`](neoforge-dev5-export.json) retains that failure. After correcting only the companion, the real exporter exposed a production error: no cloner for `neoforge:structure_modifier`. [`neoforge-dev5-export-corrected-companion.json`](neoforge-dev5-export-corrected-companion.json) retains the exact stack. The platform registry hook now supplies all native/mod datapack registry codecs instead of just the common biome modifier codec. This preserves other mods' native registry types.

Dev6 export then passed with 1,938 files: [`neoforge-dev6-export.json`](neoforge-dev6-export.json). Actual custom generation passed with the preset, `RTFRandomState`, generator context and tile cache present, all six FULL chunks, flushed save and clean exit 0: [`neoforge-dev6-generation.json`](neoforge-dev6-generation.json). Its 54-sample digest is `5fab505bc818ab96a66e2a65d644e5671bcc9029f1922af372df694d8fc00db1`, the same sampled digest observed on Fabric. This is bounded sample agreement, not an assertion that every generated chunk or third-party mod combination is identical.

`tools/qa/production_neoforge_client.py` uses a separate private launcher directory under `build/qa/neoforge-client`. The pinned official installer creates its production profile and patched client. The helper reuses verified vanilla asset/library caches, verifies declared library checksums, extracts host native DLLs, copies only closed QA worlds into a fresh saves directory, and launches an offline QA player. It never writes to existing launcher profiles or account files. Staging and process success do not establish UI correctness; actual companion stages and game-rendered screenshots must be reviewed separately.

Example reproduction, using a full Java 25 JDK:

```powershell
python tools/qa/package_candidates.py --java-home '<Java 25 JDK>' --version 0.1.0-dev.7+26.2 --report docs/validation/dual-loader-dev7-build.json
python tools/qa/build_neoforge_runtime_qa.py --java-home '<Java 25 JDK>' --production-jar neoforge/build/libs/terraforged-revival-0.1.0-dev.7+26.2-neoforge.jar
python tools/qa/production_neoforge_server.py --java-home '<Java 25 JDK>' --jar neoforge/build/libs/terraforged-revival-0.1.0-dev.7+26.2-neoforge.jar --extra-mod build/qa/neoforge-runtime/terraforged-neoforge-runtime-qa.jar --qa-mode export --report docs/validation/neoforge-export-local.json
```

For generation, pass the actual exported `presetpack` directory using `--datapack`, and select `--qa-mode generate`. For reload, use `--reuse-instance` with that harness-created generation instance and `--qa-mode reload`. The reuse path must stay inside the marked isolated instances directory.

## Dev8 candidate

The exact NeoForge dev8 JAR is `5031e101f82f9076c198c97f765f8bb5c87799ff5e9ce2a98ff43d17a9ea3878`. [Custom generation](neoforge-dev8-generation.json) and [reload](neoforge-dev8-reload.json) both passed, including active custom context, six FULL chunks, 54 samples, flushed save, clean exit 0 and matching `5fab505b...` digest. The reload used the final helper's exception cleanup and verified that save-completion messages followed shutdown.

The [production client flow](neoforge-client-dev8-fullflow.json) passed with the same packaged JAR, production provenance, clean exit 0, no critical runtime errors and all four actual Font/inner-width text-fit assertions. It activated the real Done and Create New World callbacks with seed 123456789, generated/rendered custom terrain, saved/disconnected, then loaded/rendered a copied vanilla world with seed 8675309 in the same process. The custom world had an active generator context, 22 native biome modifiers and 2,702 feature-sorter references checked; the vanilla world had no custom context, 0 native biome modifiers and 2,587 references checked. These checks were performed against actual game holders.

All seven actual Screenshot API PNGs were visually inspected and copied unchanged into [`images/neoforge-client-dev8`](images/neoforge-client-dev8/). The preset list labels fit, the translated editor/preview is visible, and both custom and subsequent vanilla terrain render from validated surface positions. This does not establish complete classic TerraForged visual parity or general modpack compatibility.
