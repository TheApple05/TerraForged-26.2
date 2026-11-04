package terraforged.qa.mixin;

import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;
import net.minecraft.world.level.levelgen.feature.FeaturePlaceContext;
import raccoonman.reterraforged.world.worldgen.feature.ClassicPillarFeature;
import terraforged.qa.ClassicPillarPlacementQa;

@Mixin(ClassicPillarFeature.class)
public abstract class ClassicPillarFeatureProbe {
    @Inject(method = "place", at = @At("HEAD"), require = 1)
    private void terraforged$qa$beforePillar(FeaturePlaceContext<ClassicPillarFeature.Config> placement, CallbackInfoReturnable<Boolean> callback) {
        ClassicPillarPlacementQa.before(placement);
    }

    @Inject(method = "place", at = @At("RETURN"), require = 1)
    private void terraforged$qa$afterPillar(FeaturePlaceContext<ClassicPillarFeature.Config> placement, CallbackInfoReturnable<Boolean> callback) {
        ClassicPillarPlacementQa.after(placement, callback.getReturnValue());
    }
}
