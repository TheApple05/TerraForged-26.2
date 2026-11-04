package raccoonman.reterraforged.mixin;

import java.nio.file.Path;
import java.util.function.Consumer;

import com.mojang.datafixers.util.Pair;
import net.minecraft.client.gui.screens.worldselection.CreateWorldScreen;
import net.minecraft.server.packs.repository.PackRepository;
import net.minecraft.world.level.WorldDataConfiguration;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.gen.Invoker;

@Mixin(CreateWorldScreen.class)
public interface CreateWorldScreenInvoker {
    @Invoker("getDataPackSelectionSettings")
    Pair<Path, PackRepository> reterraforged$getDataPackSelectionSettings(WorldDataConfiguration configuration);

    @Invoker("tryApplyNewDataPacks")
    void reterraforged$tryApplyNewDataPacks(PackRepository repository, boolean safeMode,
                                         Consumer<WorldDataConfiguration> callback);
}
