package raccoonman.reterraforged.fabric.mixin;

import java.util.concurrent.CompletableFuture;

import org.spongepowered.asm.mixin.Implements;
import org.spongepowered.asm.mixin.Interface;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

import net.minecraft.server.MinecraftServer;
import net.minecraft.server.packs.resources.ResourceManager;
import raccoonman.reterraforged.server.RTFMinecraftServer;
import raccoonman.reterraforged.world.worldgen.feature.template.template.FeatureTemplateManager;

@Implements(@Interface(iface = RTFMinecraftServer.class, prefix = "reterraforged$RTFMinecraftServer$"))
@Mixin(MinecraftServer.class)
public abstract class MixinMinecraftServer {
	private FeatureTemplateManager templateManager;

	@Inject(
		method = "<init>",
		at = @At("TAIL")
	)
	public void MinecraftServer(CallbackInfo callback) {
		this.templateManager = new FeatureTemplateManager(this.getResourceManager());
	}
	
	public FeatureTemplateManager reterraforged$RTFMinecraftServer$getFeatureTemplateManager() {
		return this.templateManager;
	}
	
	@Inject(method = "reloadResources", at = @At("RETURN"), cancellable = true)
    private void reterraforged$reloadTemplates(CallbackInfoReturnable<CompletableFuture<Void>> callback) {
        callback.setReturnValue(callback.getReturnValue().thenRun(() ->
            this.templateManager.onReload(this.getResourceManager())));
    }

    @Shadow
    public abstract ResourceManager getResourceManager();
}
