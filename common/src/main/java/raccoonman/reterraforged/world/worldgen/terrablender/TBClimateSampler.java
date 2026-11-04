package raccoonman.reterraforged.world.worldgen.terrablender;

import org.jetbrains.annotations.Nullable;

import net.minecraft.world.level.levelgen.DensityFunction;

public interface TBClimateSampler {
	void setUniqueness(DensityFunction function);
	
	@Nullable
	DensityFunction getUniqueness();
}
