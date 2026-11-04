package raccoonman.reterraforged.world.worldgen.biome.modifier.neoforge;

import java.util.Map;
import java.util.Optional;
import com.mojang.serialization.Codec;
import com.mojang.serialization.MapCodec;
import com.mojang.serialization.codecs.RecordCodecBuilder;
import net.minecraft.core.Holder;
import net.minecraft.core.HolderSet;
import net.minecraft.core.registries.Registries;
import net.minecraft.resources.ResourceKey;
import net.minecraft.world.level.biome.Biome;
import net.minecraft.world.level.levelgen.GenerationStep;
import net.minecraft.world.level.levelgen.placement.PlacedFeature;
import net.neoforged.neoforge.common.world.ModifiableBiomeInfo;
import raccoonman.reterraforged.world.worldgen.biome.modifier.BiomeModifier;

record ReplaceModifier(GenerationStep.Decoration step, Optional<HolderSet<Biome>> biomes,
        Map<ResourceKey<PlacedFeature>, Holder<PlacedFeature>> replacements) implements BiomeModifier,
        net.neoforged.neoforge.common.world.BiomeModifier {
    static final MapCodec<ReplaceModifier> CODEC = RecordCodecBuilder.mapCodec(instance -> instance.group(
        GenerationStep.Decoration.CODEC.fieldOf("step").forGetter(ReplaceModifier::step),
        Biome.LIST_CODEC.optionalFieldOf("biomes").forGetter(ReplaceModifier::biomes),
        Codec.unboundedMap(ResourceKey.codec(Registries.PLACED_FEATURE), PlacedFeature.CODEC)
            .fieldOf("replacements").forGetter(ReplaceModifier::replacements)
    ).apply(instance, ReplaceModifier::new));

    @Override public MapCodec<ReplaceModifier> codec() { return CODEC; }

    @Override public void modify(Holder<Biome> biome, Phase phase, ModifiableBiomeInfo.BiomeInfo.Builder builder) {
        if (phase != Phase.AFTER_EVERYTHING || biomes.isPresent() && !biomes.get().contains(biome)) return;
        builder.getGenerationSettings().getFeatures(step).replaceAll(feature ->
            feature.unwrapKey().map(key -> replacements.getOrDefault(key, feature)).orElse(feature));
    }
}
