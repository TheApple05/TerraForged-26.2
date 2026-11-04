package raccoonman.reterraforged.data.worldgen.tags;

import java.util.concurrent.CompletableFuture;

import net.minecraft.core.HolderLookup;
import net.minecraft.core.HolderLookup.Provider;
import net.minecraft.core.registries.Registries;
import net.minecraft.data.PackOutput;
import net.minecraft.data.tags.TagsProvider;
import net.minecraft.tags.BlockTags;
import net.minecraft.world.level.block.Block;
import net.minecraft.world.level.block.Blocks;
import raccoonman.reterraforged.data.worldgen.preset.settings.MiscellaneousSettings;
import raccoonman.reterraforged.data.worldgen.preset.settings.Preset;
import raccoonman.reterraforged.tags.RTFBlockTags;

public class RTFBlockTagsProvider extends TagsProvider<Block> {
	private Preset preset;
	
	public RTFBlockTagsProvider(Preset preset, PackOutput packOutput, CompletableFuture<Provider> completableFuture) {
		super(packOutput, Registries.BLOCK, completableFuture);

		this.preset = preset;
	}

	@Override
	protected void addTags(HolderLookup.Provider provider) {
//		MiscellaneousSettings miscellaneousSettings = this.preset.miscellaneous();

		this.tag(RTFBlockTags.SOIL).add(Blocks.DIRT.builtInRegistryHolder().key(), Blocks.COARSE_DIRT.builtInRegistryHolder().key());
		this.tag(RTFBlockTags.CLAY).add(Blocks.CLAY.builtInRegistryHolder().key());
		this.tag(RTFBlockTags.SEDIMENT).add(Blocks.SAND.builtInRegistryHolder().key(), Blocks.GRAVEL.builtInRegistryHolder().key());
		this.tag(RTFBlockTags.ERODIBLE).add(Blocks.SNOW_BLOCK.builtInRegistryHolder().key()).add(Blocks.POWDER_SNOW.builtInRegistryHolder().key()).add(Blocks.GRAVEL.builtInRegistryHolder().key()).addOptionalTag(BlockTags.DIRT);
		
//		if(!miscellaneousSettings.oreCompatibleStoneOnly) {
			this.tag(RTFBlockTags.ROCK).add(Blocks.GRANITE.builtInRegistryHolder().key(), Blocks.ANDESITE.builtInRegistryHolder().key(), Blocks.STONE.builtInRegistryHolder().key(), Blocks.DIORITE.builtInRegistryHolder().key());
//		} else{
			//TODO
//		}
	}
}