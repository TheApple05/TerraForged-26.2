# Current Horizon prepared-terrain validation

**Passed: a finite native terrain prototype.** This is not the completed
infinite Horizon port or a Horizon modpack-readiness claim. Native target:
Minecraft 26.3. Java terrain producer: the exact Fabric Minecraft 26.2 dev18
JAR `dd5b075b6fcdb26e31159c97401de8e493308142599e5e193cbf886a3e55eb94`.

[Combined evidence and package hashes](horizon-current-dev18-summary.json),
[verified Java export](server-horizon-prepared-dev18-export-verified.json),
[actual native probe](horizon-current-dev18.json).

## What ran

The developer-only exporter used the genuine production Java generator, sought
an actual classic biome, completed a FULL one-chunk halo, then captured nine
complete chunks across all 384 vertical blocks. The Java world saved and its
process exited 0. The corpus contains **884,736 block states**, **13,824 quart
biomes** and actual `reterraforged:savanna_scrub` values. This corpus does not
claim to contain all 17 variants.

The native bundle is pinned to **9dd5c4093977c718c5c0349032fd3a5434e86afc**,
which adds one concrete constructor result spelling and documentation on top of
the inspected current owned main,
`da6b3c353d9bc495704f3658bdb09fade27e2679`. It preserves the 381 commits since
the older installed-nightly baseline. The actual pinned horizon-mixins tool
planned and applied the constructor hook in an isolated source export; its
Cargo dependency versions were retained from this Horizon commit's lockfile.
The original Horizon checkout was not patched.

The real hooked native generator reproduced every captured block state and
quart biome. All **2,304 columns** matched native heightmaps and far-terrain
material spans. All 17 exported classic biome definitions loaded into the
actual native registry. Stored and queried biome IDs agreed; an in-coverage
biome locate succeeded. Clearing caches and reading the snapshot again retained
the chunk digest. Extreme signed quart Y values clamped without overflow.
Wrong seeds, missing terrain/LOD coverage, altered chunk bytes and a
checksum-valid unknown biome ID failed explicitly. Nether and End retained
their original pipelines. The primary native process exited 0; negative-case
processes exited nonzero as required.

## Deliverable scope

The combined `.hmod` contains the native code and the 17 real biome JSON
definitions; the `.hnmod` contains code only. Their native payload bytes match
the compiled/probed inputs, and their biome bytes match the qualified dev18
export. Neither includes Minecraft binaries, player-prepared game files or a
terrain snapshot. Snapshot preparation and matching local 26.3 resources are
still required. [Build/use instructions](../../horizon/README.md).

There was no rendered Horizon client, normal launcher installation or native
`level.dat` save/reload test. A Java save and a native snapshot-cache reload do
not establish a Horizon world-save round trip. Live infinite generation,
native preset controls, typed entities/block entities, structure metadata and
locate, scheduled ticks and wider native mod compatibility remain unfinished.

## Retained failures and corrections

- [Dev14 export failure](server-horizon-prepared-dev14-export.json): the Java
  biome restoration exposed a real swamp/marshland feature-order cycle. It is
  retained rather than treated as a successful export.
- [First dev18 export wrapper failure](server-horizon-prepared-dev18-export.json):
  the Java companion completed and saved, but Python's unguarded stdin close
  raised `OSError: [Errno 22] Invalid argument` after the JVM closed its pipe,
  preventing a complete process summary. The harness now handles that close
  error while keeping the actual exit/save/shutdown gates. The fresh verified
  export above passed with exit 0.
- The generic server harness's sparse-sample `customTerrainValidated` flag is
  not forced true by this separate dense exporter. Its genuine context check,
  complete corpus and native block/biome comparison provide the evidence here.
- A bounded DeepSeek first-pass review reached its output limit and returned no
  accepted draft. It was not retried or called a successful review. The retained
  reservation estimate was $0.0077883; Codex performed integration review and
  all final checks.

## Horizon repository review

[Horizon PR 1](https://github.com/TheApple05/horizon/pull/1) contains only the
constructor result spelling and documentation. It was merged after the hosted
CI and source review passed, at `21679eb68f770d3ac1c2cc1c51a535eea37787ab`.
The tested source pin remains `9dd5c4093977c718c5c0349032fd3a5434e86afc`;
both commits have tree `364e62339875b7e0362b7372c2a444adf7dd6e82`.
The fork had zero self-hosted runners and its inherited
default selected upstream-only labels for PR 1. The documented repository-only
`SELF_HOSTED_CI=off` fallback was set and verified, then the stalled run was
cancelled and [run 41815675509](https://github.com/TheApple05/horizon/actions/runs/41815675509)
restarted as attempt 2 on hosted Ubuntu. Commands, permissions and the
local-game-resource constraint were preserved. Attempt2's verify, parity-scope
and launcher jobs succeeded; vanilla parity, parity and JFR remained skipped
under their unchanged conditions. CI results remain separate from the native
results above.

The earlier [installed-nightly rehearsal](horizon-prepared-dev16.json) and
[current-Horizon/dev16 rehearsal](horizon-current-dev16-rehearsal.json) remain
historical evidence. Dev18 is also a rehearsal after subsequent Java changes;
it is not substituted for a later production artifact's validation.
