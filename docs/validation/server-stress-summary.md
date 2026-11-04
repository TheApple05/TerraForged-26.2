# Packaged Fabric boundary and persistence checks

These checks exercise **dev.2**, not later candidates. All six server processes exited with code 0, saved successfully, kept the normal 60,000 ms watchdog, and used the exact packaged production artifact. No runs remain active or pending.

## Artifact and workload

- Production JAR: `build/libs/terraforged-revival-0.1.0-dev.2+26.2.jar`.
- Production SHA-256: `94471bb542a308982bfb29c7340ed14dbf3494922cfd9f935c5f32b0485e80bd`.
- Minecraft 26.2, Fabric Loader 0.20.1, Fabric API 0.161.0+26.2, Java 25.0.5.
- Seed `8675309`; actual exported legacy-default preset for custom runs.
- 64 sampled chunks: four 4x4 regions beginning at chunk coordinates `(-66,62)`, `(-10,-10)`, `(-2,-2)`, and `(6,6)`. These cover negative and positive tile boundaries, origin, ocean, plains and birch forest.
- Each run finishes **144 FULL chunks**, including a one-chunk halo around every sampled chunk, **before** sampling any column. Requests run in ascending X/Z order or the exact reverse. Requests use Minecraft's ordinary synchronous FULL API; internal generation uses Minecraft/mod worker scheduling. This is not a claim of 144 simultaneous requests.
- Nine columns per sampled chunk, local X/Z `{0,7,15}`: 576 total. Records contain block/biome/surface observations and raw bits for nine terrain fields. Custom runs require the actual custom preset, RTFRandomState, GeneratorContext and TileCache.
- Historical fresh-run block hashes cover Y **-64 through 318 inclusive**. During review, the QA loop's exclusive maximum was corrected because 26.2 `getMaxY()` is inclusive. Reload diagnostic RLE dumps independently cover **all 384 levels, -64 through 319 inclusive**. The final source hashes this inclusive range and records the bounds. Earlier reports remain unchanged.

## Results and limits

Two fresh custom worlds and their disk reloads passed the artifact-origin, custom-generator, finite-field, save/flush, clean-exit and watchdog gates. Forward generation/sampling took 14,166 ms; reverse took 16,445 ms. Reload sampling took 5,223/5,536 ms. These are run observations, not a controlled performance benchmark.

The raw terrain fields, terrain classifications, cell climate classifications and observed biome IDs match in **576/576 columns across opposite request orders**. Their canonical comparison digest is `57d58b5e25debd703e520fcf111d2b1419a6fa9977aeaeccde27710c16d803e5`. Each world's entire recorded digest is stable after disk reload.

**Full post-feature block determinism is not established.** The two custom worlds differ in 212 sampled columns: 836 block states across Y -58..105, with 60 WORLD_SURFACE heights, 55 OCEAN_FLOOR_WG heights and 49 top-block IDs differing. Examples include birch leaves/logs, coral, dripstone, and intersecting ore/stone variants. The OCEAN_FLOOR_WG examples show feature tops rather than a changed sampled base: at `(-1009,1039)` both worlds have sand at Y31, while one has coral/fans up to Y36 and the other has water. At `(-1017,1039)` both have sand at Y36, then coral versus seagrass at Y37. OCEAN_FLOOR_WG is not an immutable pre-decoration terrain reference.

An explicit **vanilla-generator control** uses the same seed, regions, halo and request orders, requires the custom preset/context to be absent, and emits no custom-generator acceptance node. Its reports correctly set `customTerrainValidated=false`. It also shows order effects: 324 sampled columns / 1,208 states, including leaves, stone variants, ores and dripstone. TerraForged remains loaded in this control; this is **not a pure unmodded vanilla control**. The evidence supports a decoration-order explanation, but does not prove every custom difference has the same cause. No generator redesign was made to erase vanilla-style feature interactions.

These checks cover one seed and preset, sampled columns rather than every generated block, and no third-party mod combinations, visual parity, integrated-client parity or release-level performance acceptance.

## Evidence

- [Combined assertions and field digest](server-stress-2026-11-04-comparison.json).
- [Custom forward](server-stress-2026-11-03-forward.json) and [reverse](server-stress-2026-11-03-reverse.json).
- [Forward disk reload](server-stress-2026-11-03-forward-reload.json) and [reverse disk reload](server-stress-2026-11-03-reverse-reload.json).
- [Exact custom block differences and all 55 OCEAN_FLOOR_WG examples](server-stress-2026-11-03-block-differences.json).
- [Vanilla-generator forward](server-stress-2026-11-04-vanilla-forward.json), [reverse](server-stress-2026-11-04-vanilla-reverse.json), and [differences](server-stress-2026-11-04-vanilla-differences.json).

Separate companion SHA-256 values: initial `4af9127b15924e368638874cdc4dc522225b7a6218461585968fbd983ae683da`; reload dump `71f77a07b62f12281cf374524ef2fbd12478d42911a4ca82a251a75c884923a2`; inclusive/control `90eb3e7d3f3a9f545237c29f0fe671aa11c310845fdae776e1c2108f0462cc79`. Installed copies stay isolated in their respective instances. QA classes are never included in the production JAR.

## Reproduction

Source: `tests/runtime/terraforged/qa/RuntimeQaMod.java`; builder: `tools/qa/build_runtime_qa.py`; unchanged launcher: `tools/qa/production_server_smoke.py`. The final companion preserves the original six-chunk default workload. Optional properties select the boundary workload, reverse request order and RLE diagnostic dumps.

```powershell
$qaJava = 'C:/Users/Main/AppData/Local/Programs/Java/jdk-25.0.5+8'
python tools/qa/build_runtime_qa.py --java-home $qaJava --production-jar 'build/libs/terraforged-revival-0.1.0-dev.2+26.2.jar' --output 'build/qa/runtime-stress-inclusive/terraforged-runtime-qa.jar'
python tools/qa/production_server_smoke.py --java-home $qaJava --jar 'build/libs/terraforged-revival-0.1.0-dev.2+26.2.jar' --extra-mod 'build/qa/runtime-stress-inclusive/terraforged-runtime-qa.jar' --qa-mode generate --datapack 'build/qa/server/instances/20261104T072840Z-9f3db6/qa-output/presetpack' --jvm-arg=-Dterraforged.qa.workload=boundary64 --jvm-arg=-Dterraforged.qa.dumpColumns=true --timeout 180 --max-tick-time 60000 --report docs/validation/server-stress-new-forward.json
```

For a second fresh world add `--jvm-arg=-Dterraforged.qa.reverseRequests=true` and use a distinct report path. To reload, use `--qa-mode reload --reuse-instance <instance-returned-by-generate>`, omit `--datapack`, and retain workload/order properties. The final inclusive source uses `generation-boundary64-inclusive-baseline.json`, so reload its own newly generated baseline rather than the older exclusive-bound baseline.

For the explicit vanilla-generator control omit `--datapack` and add `--jvm-arg=-Dterraforged.qa.vanillaControl=true`. A custom preset/context in that control fails loudly; absent custom terrain in the normal custom workload also fails loudly. The launcher marks only custom-generator-gated runs as custom terrain evidence.
