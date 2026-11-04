# TerraForged Revival dev23 for Minecraft 26.2

An experimental modpack-evaluation prerelease for **Fabric and NeoForge**, using the same tested generation source. Install one loader's JAR in a new Minecraft Java 26.2 instance with Java 25.

- Fabric: Loader 0.20.1 and Fabric API 0.161.0+26.2.
- NeoForge: NeoForge 26.2.0.93; Fabric API is not needed.

This update restores all 17 classic TerraForged biome variants, including actual Bryce and grass-capped stone-forest pillars. New built-in presets enable Classic Biome Variants independently of custom vegetation; older presets missing that field keep it disabled. It also fixes cold biome queries, nested terrain-cache publication, cache ownership/worker configuration, TerraBlender wrapper state, and NeoForge exports that had baked modified parent features into new original biomes.

The exact JARs pass natural generation/stored-biome/cold-reload checks for all 17 variants, actual preset/editor/preview/world transitions on both clients, pinned optimizer/community-ModernFix/content/structure stacks, real combined-stack client/server joins and Linux generation/reload. Native captures show both restored pillar types. Checksums identify the unchanged tested JAR bytes; QA companions and third-party mods are excluded.

Controlled performance passes all 24 fresh trials and eight cold reloads without compiler interference or budget violations. On the recorded i9-11900K host, custom-generation medians are approximately 5.7-6.0 seconds per 64 requested FULL chunks in the base stack and 4.2-4.4 seconds with the tested C2ME/Lithium/FerriteCore stack; custom process peaks are approximately 1.30-1.34 GiB. The matched vanilla generator is faster. These are bounded local generation measurements, not FPS or server-capacity claims.

Pinned content tests use BOP 26.2.0.0.28 beta, GlitchCore 26.2.0.0.0 beta, TerraBlender 26.2.0.0.2 and Repurposed Structures 7.7.6. C2ME files are alpha. ModernFix-mVUS is a separately tested community port. Sodium/Iris/Distant Horizons with an active Complementary Reimagined shader passes nearby client rendering, but brief captures show incomplete distant LOD edges; far-distance visual quality is not accepted.

Use **Singleplayer > Create New World > World > Customize**, select Default and edit it with **>>**, then Done and wait for preset validation before creation. Keep the complete world/datapack/dependency set when moving a closed world to a matching-loader server. See INSTALLATION.md and the repository's dev23 readiness/performance records for reproduction and exact scope.

This is a prerelease for new-world testing. Full classic Engine/visual parity, geological identity, every preset/import extreme, island-preference spawning, old-world/cross-loader migration, public authentication/player load, arbitrary packs and user visual acceptance remain open.

The separate Horizon 26.3 source port has a tested finite prepared-world bridge and Java-matched native noise/utility foundations. Infinite native terrain and full native client/save acceptance remain unfinished; the Horizon prototype is excluded from these Java release assets.
