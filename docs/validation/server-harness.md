# Packaged dedicated-server verification

`tools/qa/production_server_smoke.py` installs an actual production Fabric server with the official installer, not a Loom development run. It pins Minecraft 26.2, Fabric Loader 0.20.1, Fabric API 0.161.0+26.2, installer 1.1.2 and Java 25. Published installer/API SHA-256 values and Mojang's server SHA-1 are verified. The copied mod's exact hash, metadata and loaded class origins are recorded.

```powershell
python tools/qa/production_server_smoke.py --java-home PATH_TO_JDK_25 --prepare-only --report docs/validation/server-preparation.json
python tools/qa/production_server_smoke.py --java-home PATH_TO_JDK_25 --jar build/libs/CANDIDATE.jar --report docs/validation/server-boot.json
```

Each test uses a disposable instance beneath ignored `build/qa/server/instances`, binds only `127.0.0.1` on a selected ephemeral port, uses an offline QA configuration and creates its own EULA acknowledgement. Existing worlds and launcher profiles are not touched. The process runs with `fabric.development=false`; inherited Java-option environment variables are removed only for that child.

The vanilla boot gate requires observed game/loader versions, the ready marker, an explicit save acknowledgement, packaged class origin, unchanged artifact hash, clean stop and JVM exit 0. It proves packaged loading and vanilla-world startup, not custom terrain generation.

## Companion modes

Supply the separately built QA mod using `--extra-mod build/qa/runtime/terraforged-runtime-qa.jar`, then select `--qa-mode export`, `generate` or `reload`. The harness supplies mode, output directory and expected production-artifact SHA-256 properties. It waits for a fresh `qa-output/result.json`, preserving the complete companion evidence in its report. The companion may save/stop itself; that path requires its actual callback/save markers, shutdown log and process exit. Prior results are archived before a reload run.

Export writes a preset pack. `--datapack PATH_TO_EXPORTED_PACK` copies it into a new world's datapack directory before first load. `--reuse-instance build/qa/server/instances/ID` restarts only an instance carrying this harness's marker, retaining the disposable world's state for reload comparisons. Do not pass `--datapack` again when reusing an existing copied pack.

The normal 60,000 ms server watchdog is retained by default. Pass `--max-tick-time 0` **only for a bounded diagnostic follow-up** when the watchdog could conceal useful evidence. The harness still imposes a maximum 300-second deadline (configurable downward), requests a graceful stop, and terminates only its own spawned process if cleanup times out. A stalled generation/reload run captures `jcmd OWNED_PID Thread.print` 20 seconds after the ready marker. Dump path, elapsed timing and exit code are recorded; no unrelated JVM is targeted.

## Initial evidence

- [Preparation](server-2026-11-03-preparation.json): verified dependencies, no runtime claim.
- [Initial production boot](server-2026-11-03-initial.json): exact candidate loaded, vanilla world ready/saved/stopped, exit 0.
- [Real preset export](server-2026-11-03-export.json): actual data-generation pipeline produced 1,934 files and the companion's three biome-decorator helper assertions passed, followed by clean shutdown.
- [Initial custom-generation attempt](server-2026-11-03-generation-initial.json): custom preset, random-state extension, generator context and tile cache were present; the watchdog then terminated a synchronous chunk wait after 60 seconds. This is a failed generation test, not a passing terrain result.
- [Watchdog-disabled follow-up](server-2026-11-03-generation-watchdog-disabled.json): the same packaged candidate and companion completed six full chunks and 54 column/cell samples, then flushed and stopped with exit 0. Its QA phase completed before the 20-second stalled-run dump threshold.
- [Disk reload](server-2026-11-03-reload-initial.json): the same disposable world reloaded six sampled chunks and reproduced digest `5fab505bc818ab96a66e2a65d644e5671bcc9029f1922af372df694d8fc00db1`, then flushed/stopped with exit 0.

Those passing follow-ups do not resolve the original intermittent failure. The original watchdog crash retained all thread stacks; review found an asynchronous cache wait holding a non-reentrant lock while nested chunk work tried to acquire it. A corrected production candidate and normal-watchdog regression runs are required before making a reliability claim. The dump trigger remains available if a subsequent run stalls. Visual parity, client behavior, all presets and third-party combinations remain separate gates.
