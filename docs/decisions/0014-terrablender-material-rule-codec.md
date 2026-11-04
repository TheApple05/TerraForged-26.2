# 0014: Correct TerraBlender's material-rule serializer boundary

## Reproduced modpack failure

Exact dev8 builds boot with Biomes O' Plenty26.2.0.0.28, GlitchCore26.2.0.0.0 and TerraBlender26.2.0.0.2 on both Fabric and NeoForge. The real biome registry contains68 BOP biomes;59 appear in the overworld source's possible-biome set. Actual preset export then fails in registry cloning: `Unregistered holder` for `minecraft:worldgen/material_rule`, referring to `NamespacedSurfaceRuleSource.CODEC`. Both processes exit cleanly, but both QA callbacks fail. A clean process exit is not export acceptance.

Retained failed reports: [Fabric dev8](../validation/server-modpack-biomesoplenty-fabric-dev8-export.json), [NeoForge dev8](../validation/modpack-biomesoplenty-neoforge-dev8-export.json). Partially written outputs remain isolated QA evidence and are not accepted preset packs.

## Exact cause

The published TerraBlender26.2.0.0.2 `MixinBuiltInRegistries` registers the expression `NamespacedSurfaceRuleSource.CODEC.codec()` as `terrablender:merged`. That expression is a `Codec`. Minecraft26.2's `MATERIAL_RULE` registry expects `MapCodec` values, and the rule source's `codec()` method returns its actual `MapCodec` constant. The registry therefore contains the wrong object/type. Encoding cannot find that actual MapCodec by identity. This is a registration API-migration defect, not a reason to discard BOP rules or skip serialization errors.

The exact bytecode call is `Registry.register(Registry, Identifier, Object):Object`; the problematic value comes from `MapCodec.codec()` immediately before it. [Machine-readable bytecode evidence](../validation/terrablender-26.2-codec-registration-evidence.json) records the descriptor and class hashes.

Checked the official Modrinth release list on November4,2026:26.2.0.0.2 remains the latest26.2 artifact for [Fabric](https://modrinth.com/mod/terrablender/version/KQix25Qc) and [NeoForge](https://modrinth.com/mod/terrablender/version/TzDe72Nr). Downloaded official CDN files and verified their published SHA1/SHA512. Their ZIP hashes differ from the resolved Maven artifacts, but the decisive `MixinBuiltInRegistries.class` bytes are identical for each loader. No dependency was silently swapped or upgraded. [Pinned content-stack artifacts and in-JAR dependency declarations](../validation/modpack-biomesoplenty-26.2-artifacts.json) also retain the BOP/GlitchCore evidence. In-JAR BOP metadata requires TerraBlender even though its marketplace dependency list omits that entry.

## Correction and acceptance gate

The optional compatibility correction is restricted to the exact `MATERIAL_RULE` registry and `terrablender:merged` registration. It replaces the erroneous Codec argument with the actual `NamespacedSurfaceRuleSource.CODEC` MapCodec. An already-correct MapCodec value remains unchanged, so a future upstream fix is not wrapped or replaced. It uses the existing optional-mod mixin gate and does not mutate frozen registries after startup.

The separate runtime QA companion now reads the actual registered entry whenever TerraBlender is loaded and requires both `instanceof MapCodec` and identity with the rule source's real static CODEC. It also retains the actual climate sampler/TargetPoint finite-uniqueness check and the BOP registry/source membership inventories.

Coherent dev9 artifacts must export with the complete content stack, then load **that newly exported pack**, generate custom terrain, save and reload under the normal watchdog. Exporting elsewhere and only loading that prior pack is a diagnostic, not proof this defect is fixed. Runtime results are recorded separately; this decision alone does not claim they passed. Registration and possible-biome membership do not prove every biome is reachable, nor do these checks establish arbitrary modpack compatibility.
