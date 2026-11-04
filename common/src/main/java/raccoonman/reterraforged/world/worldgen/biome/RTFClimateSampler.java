package raccoonman.reterraforged.world.worldgen.biome;

import net.minecraft.core.BlockPos;

public interface RTFClimateSampler {
	void setGeneratorContext(raccoonman.reterraforged.world.worldgen.GeneratorContext context);
	raccoonman.reterraforged.world.worldgen.GeneratorContext getGeneratorContext();

	void setSpawnSearchCenter(BlockPos center);
	
	BlockPos getSpawnSearchCenter();
}
