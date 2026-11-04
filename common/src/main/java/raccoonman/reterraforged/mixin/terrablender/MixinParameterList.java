package raccoonman.reterraforged.mixin.terrablender;

import net.minecraft.core.RegistryAccess;
import net.minecraft.world.level.biome.Climate;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Unique;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.Redirect;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;
import raccoonman.reterraforged.world.worldgen.noise.NoiseUtil;
import raccoonman.reterraforged.world.worldgen.terrablender.TBTargetPoint;
import terrablender.api.RegionType;
import terrablender.worldgen.IExtendedParameterList;

// TerraBlender adds the positional API at its default priority before this mixin.
@Mixin(value = Climate.ParameterList.class, priority = 900)
class MixinParameterList<T> {
    @Unique
    private int reterraforged$maxIndex;

    @Inject(at = @At("TAIL"), method = "initializeForTerraBlender", remap = false, require = 1)
    private void reterraforged$recordRegionCount(RegistryAccess registryAccess, RegionType regionType,
                                               long seed, CallbackInfo callback) {
        this.reterraforged$maxIndex = Math.max(0, ((IExtendedParameterList<?>) (Object) this).getTreeCount() - 1);
    }

    @Redirect(method = "findValuePositional", remap = false, require = 1, at = @At(
        value = "INVOKE",
        target = "Lnet/minecraft/world/level/biome/Climate$ParameterList;getUniqueness(III)I",
        remap = false
    ))
    private int reterraforged$selectRegion(Climate.ParameterList<T> parameters, int x, int y, int z,
                                         Climate.TargetPoint target, int sampleX, int sampleY, int sampleZ) {
        if ((Object) target instanceof TBTargetPoint point) {
            double uniqueness = point.getUniqueness();
            if (Double.isFinite(uniqueness)) {
                float bounded = (float) Math.clamp(uniqueness, 0.0, 1.0);
                return NoiseUtil.round(this.reterraforged$maxIndex * bounded);
            }
        }
        return ((IExtendedParameterList<?>) (Object) parameters).getUniqueness(x, y, z);
    }
}
