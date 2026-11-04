package raccoonman.reterraforged.mixin.terrablender;

import net.minecraft.core.QuartPos;
import net.minecraft.world.level.biome.Climate;
import net.minecraft.world.level.biome.Climate.TargetPoint;
import net.minecraft.world.level.levelgen.DensityFunction;
import org.jetbrains.annotations.Nullable;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Unique;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;
import raccoonman.reterraforged.world.worldgen.terrablender.TBClimateSampler;
import raccoonman.reterraforged.world.worldgen.terrablender.TBTargetPoint;

@Mixin(Climate.Sampler.class)
class MixinClimateSampler implements TBClimateSampler {
    @Unique @Nullable
    private DensityFunction reterraforged$uniqueness;

    @Inject(at = @At("RETURN"), method = "sample", require = 1)
    private void reterraforged$sampleUniqueness(int quartX, int quartY, int quartZ,
                                              CallbackInfoReturnable<TargetPoint> callback) {
        if (this.reterraforged$uniqueness != null && (Object) callback.getReturnValue() instanceof TBTargetPoint target) {
            DensityFunction.SinglePointContext context = new DensityFunction.SinglePointContext(
                QuartPos.toBlock(quartX), QuartPos.toBlock(quartY), QuartPos.toBlock(quartZ));
            target.setUniqueness(this.reterraforged$uniqueness.compute(context));
        }
    }

    @Override
    public void setUniqueness(DensityFunction uniqueness) { this.reterraforged$uniqueness = uniqueness; }

    @Override @Nullable
    public DensityFunction getUniqueness() { return this.reterraforged$uniqueness; }
}
