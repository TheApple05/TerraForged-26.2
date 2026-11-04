package raccoonman.reterraforged.mixin.terrablender;

import net.minecraft.world.level.biome.Climate;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Unique;
import raccoonman.reterraforged.world.worldgen.terrablender.TBTargetPoint;

@Mixin(Climate.TargetPoint.class)
class MixinTargetPoint implements TBTargetPoint {
    @Unique
    private double reterraforged$uniqueness = Double.NaN;

    @Override
    public void setUniqueness(double uniqueness) { this.reterraforged$uniqueness = uniqueness; }

    @Override
    public double getUniqueness() { return this.reterraforged$uniqueness; }
}
