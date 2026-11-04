# 0013: TerraBlender method-merge injection priority

The first real Fabric 26.2 + TerraBlender 26.2.0.0.2 server test failed before startup. `MixinParameterList` attempted an `INVOKE` redirect inside TerraBlender's merged `findValuePositional` method. Mixin rejected our priority 900 against TerraBlender's 1000. Matching priority 1000 also failed; both [dev5](../validation/server-2026-11-04-dev5-terrablender.json) and [dev6](../validation/server-2026-11-04-dev6-terrablender.json) failures remain recorded.

Inspection of the actual resolved `net.fabricmc:sponge-mixin:0.17.5+mixin.0.8.7` bytecode confirms `Injector.find` calls `InjectionPoint.checkPriority(mergedPriority, injectorPriority)`. Its conditional accepts only `mergedPriority < injectorPriority`, not equality. Method merging happens before injection preparation. The compatibility mixin now uses priority **1100**, with the existing required descriptors and behavior retained. No injection is disabled or made optional to hide a failure.

The exact [dev7 Fabric package](../validation/server-2026-11-04-dev7-terrablender.json) then starts and generates the full 144-chunk boundary workload with TerraBlender installed. Its [reload](../validation/server-2026-11-04-dev7-terrablender-reload.json) matches the saved 576-column digest, verifies actual uniqueness density installation and finite target-point propagation, flushes the save, and exits 0 under the normal watchdog. The test companion is separate from the production JAR.

This demonstrates the tested loader/dependency pair and workload. It does not establish that every region/biome content mod using TerraBlender works, or that all optional integrations have been tested. NeoForge requires its own exact-package result even though this common mixin is shared.
