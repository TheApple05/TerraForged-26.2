package terraforged.qa;

import java.nio.file.Path;
import net.fabricmc.api.ModInitializer;
import net.fabricmc.fabric.api.event.lifecycle.v1.ServerLifecycleEvents;

/** Test-only companion: the packaged sampler needs Fabric's real registry mixins. */
public final class NativePillarCaptureMod implements ModInitializer {
    @Override public void onInitialize() {
        ServerLifecycleEvents.SERVER_STARTED.register(server -> {
            try {
                if (!net.minecraft.SharedConstants.getCurrentVersion().name().equals("26.2")) {
                    throw new AssertionError("Wrong running game version");
                }
                Class.forName("NativePillarSamplerProbe").getMethod("main", String[].class)
                    .invoke(null, (Object)new String[] {
                        System.getProperty("terraforged.qa.inputs"),
                        System.getProperty("terraforged.qa.production"),
                        System.getProperty("terraforged.qa.output")
                    });
                System.out.println("PILLAR_FABRIC_SERVER_CAPTURE_OK minecraft=26.2");
            } catch (Throwable error) {
                error.printStackTrace();
                System.err.println("PILLAR_FABRIC_SERVER_CAPTURE_FAILED");
            } finally {
                server.halt(false);
            }
        });
    }
}
