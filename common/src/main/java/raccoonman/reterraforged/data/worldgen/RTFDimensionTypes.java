package raccoonman.reterraforged.data.worldgen;

import java.util.OptionalLong;

import net.minecraft.data.worldgen.BootstrapContext;
import net.minecraft.tags.BlockTags;
import net.minecraft.util.valueproviders.UniformInt;
import net.minecraft.world.level.dimension.BuiltinDimensionTypes;
import net.minecraft.world.level.dimension.DimensionType;
import raccoonman.reterraforged.data.worldgen.preset.Preset;
import raccoonman.reterraforged.data.worldgen.preset.WorldSettings;

public final class RTFDimensionTypes {
	
	public static void bootstrap(Preset preset, BootstrapContext<DimensionType> ctx) {
		WorldSettings worldSettings = preset.world();
		WorldSettings.Properties properties = worldSettings.properties;
		int worldHeight = properties.worldHeight;
		int worldDepth = properties.worldDepth;
		int totalHeight = worldDepth + worldHeight;
		
        ctx.register(BuiltinDimensionTypes.OVERWORLD, raccoonman.reterraforged.data.worldgen.OverworldDimensionData.create(ctx, worldDepth, worldHeight));
	}
}
