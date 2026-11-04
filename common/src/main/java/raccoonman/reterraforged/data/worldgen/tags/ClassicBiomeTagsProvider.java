package raccoonman.reterraforged.data.worldgen.tags;

import java.util.concurrent.CompletableFuture;
import net.minecraft.core.HolderLookup;
import net.minecraft.core.registries.Registries;
import net.minecraft.data.PackOutput;
import net.minecraft.data.tags.TagsProvider;
import net.minecraft.tags.TagKey;
import net.minecraft.world.level.biome.Biome;
import raccoonman.reterraforged.RTFCommon;
import raccoonman.reterraforged.world.worldgen.biome.ClassicBiomes;

/** Additive inheritance of the installed modern parent's actual biome tags. */
public final class ClassicBiomeTagsProvider extends TagsProvider<Biome> {
    private final HolderLookup.Provider original;

    public ClassicBiomeTagsProvider(PackOutput output, CompletableFuture<HolderLookup.Provider> lookup, HolderLookup.Provider original) {
        super(output, Registries.BIOME, lookup);
        this.original = original;
    }

    @Override
    protected void addTags(HolderLookup.Provider provider) {
        var biomes = this.original.lookupOrThrow(Registries.BIOME);
        for (var variant : ClassicBiomes.VARIANTS) {
            var parent = biomes.getOrThrow(variant.tagParent());
            biomes.listTags().filter(tag -> tag.contains(parent)).forEach(tag -> this.tag(tag.key()).add(variant.key()));
            this.tag(TagKey.create(Registries.BIOME, RTFCommon.location("classic_variants"))).add(variant.key());
        }
    }
}
