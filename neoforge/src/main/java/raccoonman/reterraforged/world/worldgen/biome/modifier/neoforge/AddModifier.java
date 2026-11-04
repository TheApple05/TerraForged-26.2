package raccoonman.reterraforged.world.worldgen.biome.modifier.neoforge;

import java.util.List;
import java.util.Optional;
import com.mojang.serialization.MapCodec;
import com.mojang.serialization.codecs.RecordCodecBuilder;
import net.minecraft.core.Holder;
import net.minecraft.core.HolderSet;
import net.minecraft.world.level.biome.Biome;
import net.minecraft.world.level.levelgen.GenerationStep;
import net.minecraft.world.level.levelgen.placement.PlacedFeature;
import net.neoforged.neoforge.common.world.ModifiableBiomeInfo;
import raccoonman.reterraforged.world.worldgen.biome.modifier.BiomeModifier;
import raccoonman.reterraforged.world.worldgen.biome.modifier.Filter;
import raccoonman.reterraforged.world.worldgen.biome.modifier.Order;

record AddModifier(Order order, GenerationStep.Decoration step, Optional<Filter> biomes,
        HolderSet<PlacedFeature> features) implements BiomeModifier,
        net.neoforged.neoforge.common.world.BiomeModifier {
    static final MapCodec<AddModifier> CODEC = RecordCodecBuilder.mapCodec(instance -> instance.group(
        Order.CODEC.fieldOf("order").forGetter(AddModifier::order),
        GenerationStep.Decoration.CODEC.fieldOf("step").forGetter(AddModifier::step),
        Filter.CODEC.optionalFieldOf("biomes").forGetter(AddModifier::biomes),
        PlacedFeature.LIST_CODEC.fieldOf("features").forGetter(AddModifier::features)
    ).apply(instance, AddModifier::new));

    @Override public MapCodec<AddModifier> codec() { return CODEC; }

    @Override public void modify(Holder<Biome> biome, Phase phase, ModifiableBiomeInfo.BiomeInfo.Builder builder) {
        if (phase != Phase.AFTER_EVERYTHING || biomes.isPresent() && !biomes.get().test(biome)) return;
        List<Holder<PlacedFeature>> destination = builder.getGenerationSettings().getFeatures(step);
        List<Holder<PlacedFeature>> ordered = order.add(destination, features.stream().toList());
        destination.clear();
        destination.addAll(ordered);
    }
}
