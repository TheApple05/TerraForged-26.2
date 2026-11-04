package raccoonman.reterraforged.mixin;

import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.gen.Invoker;

import net.minecraft.core.BlockPos;
import net.minecraft.world.level.biome.Biome;

@Mixin(Biome.class)
public interface BiomeTemperatureInvoker {
    @Invoker("getTemperature")
    float reterraforged$getTemperature(BlockPos pos, int seaLevel);
}
