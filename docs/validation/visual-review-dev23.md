# Dev23 native capture review

Reviewed actual byte-exact native captures on 2026-11-04. The image manifests bind the production reports and all 30 PNG hashes. No generated or retouched scene image is used.

The two `client-fabric-dev23-pillar-scenes` images show natural terracotta Bryce spires and tall stone columns with grass, soil and vegetation caps. The same closed source world passed actual surface-to-FULL feature checks and cold reload. Camera poses, classification, bounded loaded inner chunks and completed section meshes are recorded by the scene QA companion. This confirms visible pillars; it does not establish an original-version visual match or the user's aesthetic acceptance.

The base Fabric terrain image shows nearby slopes and a lava pool with no menu overlay. The NeoForge editor image shows English continent controls, a populated preview and working navigation layout. Both actual editor flows have independent callback, preview-digest and world-creation checks.

The NeoForge content/shader image shows BOP vegetation, terrain and active shader lighting. The Fabric subsequent-vanilla shader image shows the river/forest surface, with conspicuous incomplete distant LOD edges. The NeoForge shader capture also retains the Distant Horizons informational overlay. These brief captures support nearby render readiness and active shader operation. They do not certify completed distant meshes, artifact-free far LOD rendering, FPS or all shader settings. The subsequent vanilla world has no TerraForged generator context; that observation does not by itself identify the cause of distant rendering artifacts.

The pinned DH/Iris/shader combination should be treated as experimentally exercised compatibility, not a finished far-distance visual recommendation. User visual acceptance remains outstanding.

Evidence: `client-dev23-images.json`, `client-dev23-pillar-images.json`, the four base/content client reports and `client-fabric-dev23-pillar-scenes.json`.
