package raccoonman.reterraforged.mixin;

import org.spongepowered.asm.mixin.Final;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

import net.minecraft.server.level.ChunkMap;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.world.level.levelgen.RandomState;
import raccoonman.reterraforged.world.worldgen.RTFRandomState;

@Mixin(ChunkMap.class)
public class MixinChunkMap {
    @Shadow @Final
    private RandomState randomState;
    @Shadow @Final
    private ServerLevel level;

    @Inject(method = "<init>", at = @At("TAIL"))
    private void reterraforged$initializeRegistryContext(CallbackInfo callback) {
        if ((Object) this.randomState instanceof RTFRandomState rtfRandomState) {
            rtfRandomState.initialize(this.level.registryAccess());
        }
    }
}
