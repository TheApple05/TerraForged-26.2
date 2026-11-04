package raccoonman.reterraforged.neoforge;

import net.neoforged.api.distmarker.Dist;
import net.neoforged.bus.api.IEventBus;
import net.neoforged.fml.common.Mod;
import net.neoforged.fml.loading.FMLLoader;
import raccoonman.reterraforged.RTFCommon;
import raccoonman.reterraforged.platform.PlatformServices;
import raccoonman.reterraforged.platform.neoforge.NeoForgePlatformServices;

@Mod(RTFCommon.MOD_ID)
public final class RTFNeoForge {
    public RTFNeoForge(IEventBus modBus) {
        NeoForgePlatformServices platform = (NeoForgePlatformServices) PlatformServices.get();
        platform.attach(modBus);
        RTFCommon.bootstrap();
        if (FMLLoader.getCurrent().getDist() == Dist.CLIENT) {
            RTFNeoForgeClient.attach(modBus);
        }
    }
}
