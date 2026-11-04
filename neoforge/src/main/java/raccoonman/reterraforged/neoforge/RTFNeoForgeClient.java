package raccoonman.reterraforged.neoforge;

import net.minecraft.world.level.levelgen.presets.WorldPresets;
import net.neoforged.bus.api.IEventBus;
import net.neoforged.neoforge.client.event.RegisterPresetEditorsEvent;
import raccoonman.reterraforged.client.gui.screen.presetconfig.PresetConfigScreen;

final class RTFNeoForgeClient {
    static void attach(IEventBus modBus) {
        modBus.addListener(RTFNeoForgeClient::registerPresetEditors);
    }

    private static void registerPresetEditors(RegisterPresetEditorsEvent event) {
        event.register(WorldPresets.NORMAL, (screen, context) -> new PresetConfigScreen(screen));
    }
}
