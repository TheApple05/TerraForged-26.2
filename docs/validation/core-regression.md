# Core-kernel regression: initial result

**Passed on 2026-11-03 America/Los_Angeles (2026-11-04 UTC), using Temurin 25.0.5+8.** This verifies a small, unchanged mathematical foundation. It does not establish a playable port, full noise composition, or terrain parity.

## What ran

`tools/verify_core.py` compiled these real production files directly with `javac --release 25`, without Minecraft, substituted classes, or stubs:

- `noise/NoiseUtil.java`
- `util/PosUtil.java`
- `util/Seed.java`
- `util/FastRandom.java`

The files are under `common/src/main/java/raccoonman/reterraforged/world/worldgen/`. Their canonical-LF SHA-256 hashes still match upstream commit `29239f1f164d19dbfcccaca4a8277d64e784207c`.

The frozen fixture was captured by separately extracting those four files with `git show` at that exact upstream commit, compiling them in an isolated temporary directory, and running the project-owned probe. Candidate verification uses the checked-in fixture and requires neither network access nor `.reference/`.

The probe emits **2,167 keyed records**:

- 1,734 noise-kernel samples: six seeds, including signed integer limits, crossed with 17 coordinates in each axis. Coordinates cover signed zero, negative/positive fractional values, `-17/-16/-1/0/15/16/17`, and both 30-million-block world limits. Results include hashes, raw float bits for value/gradient/cell kernels, and coordinate-derived seeds.
- 121 integer position-packing combinations, checking exact signed round trips at boundaries and integer limits.
- 289 float/mixed position-packing combinations, checking raw-bit round trips including negative zero.
- Six seed-sequence records, checking split/root semantics, offset, and overflow behavior.
- Six deterministic random-sequence records, covering explicit seeds, bounded rejection sampling, floats in `[0,1)`, booleans, and reseeding/gamma paths.
- Ten math records covering interpolation, trigonometric lookup, floor/round conversion, mapping, curves, powers, and square roots.
- One explicit legacy negative-integer-floor observation.

Four separate candidate JVM processes produced exactly the same output as the pinned reference: serial twice, shuffled serial, and shuffled parallel with four common-pool workers. Floating-point kernel results are compared as raw IEEE-754 bits. Finite-result checks and independent packing/range assertions also ran.

Fixture SHA-256: `8070efecb62e736c3d22c64e8e22922ac468c0350804d3f8e0bfe615f3d35dc5`.

The optional harness self-test changed only a temporary copy of the hash kernel's seed operation. The resulting output differed from the fixture, demonstrating that this regression is detected. Production source and expected fixtures were unchanged.

Full evidence: [initial machine-readable result](core-initial.json), [reference metadata](../../tests/core/reference-metadata.json), and [raw reference output](../../tests/core/reference-output.tsv).

## Reproduce

From the repository root in PowerShell:

```powershell
python tools/verify_core.py --java-home 'C:\Users\Main\AppData\Local\Programs\Java\jdk-25.0.5+8'
```

Or set `JAVA_HOME` to a full Java 25 JDK and run `python tools/verify_core.py`. Python 3.9+ is required. The default result is written to ignored `build/qa/core/report.json`; `--report PATH` selects another result path. Each probe runs in its own JVM. The temporary directory is verified to be inside this workspace before use.

To reproduce the one-time mutation self-test and recorded result:

```powershell
python tools/verify_core.py --java-home 'C:\Users\Main\AppData\Local\Programs\Java\jdk-25.0.5+8' --self-test --report docs/validation/core-initial.json
```

Normal verification never regenerates expected output. Explicit reference capture is a separate maintenance operation:

```powershell
python tools/verify_core.py --java-home $env:JAVA_HOME --generate-reference --upstream-repo .reference/ReTerraForged
```

Capture refuses to overwrite an existing fixture unless `--replace-fixture` is also supplied. Review any fixture change against the pinned upstream source and its source/probe hashes. Do not refresh fixtures from candidate output to make a failure pass.

## Known legacy behavior and limits

Upstream `NoiseUtil.floor(-1.0f)` returns `-2`, and negative exact integer coordinates exhibit the same off-by-one formula. This result is preserved and made visible; the test does not endorse it as mathematical floor. Any fix must be a separately reviewed generation change because it can change deterministic terrain.

The reference and candidate runs both used Java 25 on this machine. This is not an old-Java-versus-new-Java comparison or a cross-platform floating-point guarantee. Future Java/platform differences require review rather than an automatic tolerance.

These checks cover kernel sampling at chunk-edge coordinates, **not actual neighboring chunk or tile seams**. They do not exercise full composed Perlin/Simplex modules, terrain/climate/rivers/erosion, Minecraft registries, mutable tile caches, client/server generation, save/reload, the distributable JAR, screenshots, or performance. Those remain separate validation gates.
