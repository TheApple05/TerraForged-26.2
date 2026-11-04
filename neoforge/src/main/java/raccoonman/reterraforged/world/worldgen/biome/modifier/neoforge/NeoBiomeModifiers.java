package raccoonman.reterraforged.world.worldgen.biome.modifier.neoforge;

import java.util.Map;
import java.util.Optional;
import com.mojang.datafixers.util.Pair;
import net.minecraft.core.Holder;
import net.minecraft.core.HolderSet;
import net.minecraft.resources.ResourceKey;
import net.minecraft.world.level.biome.Biome;
import net.minecraft.world.level.levelgen.GenerationStep;
import net.minecraft.world.level.levelgen.placement.PlacedFeature;
import raccoonman.reterraforged.world.worldgen.biome.modifier.BiomeModifier;
import raccoonman.reterraforged.world.worldgen.biome.modifier.BiomeModifiers;
import raccoonman.reterraforged.world.worldgen.biome.modifier.Filter;
import raccoonman.reterraforged.world.worldgen.biome.modifier.Order;

public final class NeoBiomeModifiers {
    private NeoBiomeModifiers() { }

    public static void bootstrap() {
        BiomeModifiers.register("add", AddModifier.CODEC);
        BiomeModifiers.register("replace", ReplaceModifier.CODEC);
    }

    public static BiomeModifier add(Order order, GenerationStep.Decoration step,
            Optional<Pair<Filter.Behavior, HolderSet<Biome>>> biomes, HolderSet<PlacedFeature> features) {
        return new AddModifier(order, step, biomes.map(pair -> new Filter(pair.getSecond(), pair.getFirst())), features);
    }

    public static BiomeModifier replace(GenerationStep.Decoration step, Optional<HolderSet<Biome>> biomes,
            Map<ResourceKey<PlacedFeature>, Holder<PlacedFeature>> replacements) {
        return new ReplaceModifier(step, biomes, replacements);
    }
}
