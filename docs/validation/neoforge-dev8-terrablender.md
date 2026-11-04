# NeoForge dev8 with TerraBlender

**Passed:** isolated production custom generation and disk reload on Minecraft26.2 / NeoForge26.2.0.93 / Java25.0.5, with normal60,000ms watchdog. Both processes saved and exited0; exact installed production class/JAR provenance and SHA were verified.

- Production SHA-256: `5031e101f82f9076c198c97f765f8bb5c87799ff5e9ce2a98ff43d17a9ea3878`.
- TerraBlender26.2.0.0.2 NeoForge artifact SHA-256: `b862ff5bd541c9d23d54025100ceef822db7c9d6bfe4e03a25aa4294191af92b`.
- Test-only companion SHA-256: `03b2118b09120c37d42677125c1a759c927b4b4e714da400cb7cf8f92ae54daf`; [build/source evidence](neoforge-dev8-terrablender-qa-build.json).
- Seed8675309, exported legacy-default preset,6FULL chunks/54observed columns in each run.
- Actual global climate sampler has an installed uniqueness density, and its actual returned TargetPoint carries finite uniqueness `0.22231745719909668`. TerraBlender classes were observed loading in both processes.
- Generation/reload digest matches: `5fab505bc818ab96a66e2a65d644e5671bcc9029f1922af372df694d8fc00db1`.

[Generation](neoforge-dev8-terrablender-generation.json), [reload](neoforge-dev8-terrablender-reload.json), and [strict post-check summary](neoforge-dev8-terrablender-summary.json) retain the exact dependencies, class origins, watchdog settings, sample observations and limits. This pair has TerraBlender alone; a separate content-mod stack check covers Biomes O' Plenty. It does not establish every biome's reachability, arbitrary modpack compatibility, client rendering, or performance acceptance.

Reproduce with `tools/qa/build_neoforge_runtime_qa.py` against the exact dev8Neo JAR, then `tools/qa/production_neoforge_server.py` with both QA and exact TerraBlender JARs supplied through repeated `--extra-mod`, `--qa-mode generate`, and the exported Neo legacy preset directory through `--datapack`. Use the returned instance with `--reuse-instance` and `--qa-mode reload`, omitting `--datapack`, for the second run. The exact source/export instance and artifact paths are in the linked JSON records.
