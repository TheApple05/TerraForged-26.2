package raccoonman.reterraforged.data.worldgen.preset;

import java.util.ArrayList;
import java.util.List;

import com.google.common.collect.ImmutableList;

import net.minecraft.core.Holder;
import net.minecraft.core.HolderGetter;
import net.minecraft.world.level.biome.Biome;
import net.minecraft.data.worldgen.SurfaceRuleData;
import net.minecraft.world.level.levelgen.DensityFunction;
import net.minecraft.world.level.levelgen.SurfaceRules;
import raccoonman.reterraforged.RTFCommon;
import raccoonman.reterraforged.data.worldgen.preset.settings.MiscellaneousSettings;
import raccoonman.reterraforged.data.worldgen.preset.settings.Preset;
import raccoonman.reterraforged.tags.RTFBlockTags;
import raccoonman.reterraforged.world.worldgen.noise.module.Noise;
import raccoonman.reterraforged.world.worldgen.surface.rule.RTFSurfaceRules;
import raccoonman.reterraforged.world.worldgen.surface.rule.StrataRule.Strata;

public final class PresetSurfaceRuleData {

    public static SurfaceRules.RuleSource overworld(Preset preset, HolderGetter<DensityFunction> densityFunctions, HolderGetter<Noise> noise,
            net.minecraft.core.HolderLookup.Provider original, net.minecraft.data.worldgen.BootstrapContext<?> context) {
        SurfaceRules.RuleSource vanilla = SurfaceRuleData.overworld(original.lookupOrThrow(net.minecraft.core.registries.Registries.BIOME));
        var json = SurfaceRules.RuleSource.CODEC.encodeStart(original.createSerializationContext(com.mojang.serialization.JsonOps.INSTANCE), vanilla).getOrThrow();
        if (preset.miscellaneous().classicBiomeVariants) expandClassicConditions(json);
        vanilla = SurfaceRules.RuleSource.CODEC.parse(raccoonman.reterraforged.data.worldgen.BootstrapContextOps.forDecoding(context), json).getOrThrow();
        return overworld(preset, densityFunctions, noise, vanilla);
    }

    private static void expandClassicConditions(com.google.gson.JsonElement json) {
        if (json.isJsonObject()) {
            var object = json.getAsJsonObject();
            if (object.has("biome_is") && object.get("biome_is").isJsonArray()) {
                var entries = object.getAsJsonArray("biome_is");
                var names = new java.util.HashSet<String>();
                entries.forEach(entry -> { if (entry.isJsonPrimitive()) names.add(entry.getAsString()); });
                for (var variant : raccoonman.reterraforged.world.worldgen.biome.ClassicBiomes.VARIANTS) {
                    var parent = variant.key().equals(PresetBiomeData.STEPPE) ? net.minecraft.world.level.biome.Biomes.SAVANNA : variant.parent();
                    if (names.contains(parent.identifier().toString()) && !names.contains(variant.key().identifier().toString())) entries.add(variant.key().identifier().toString());
                }
            }
            object.entrySet().forEach(entry -> expandClassicConditions(entry.getValue()));
        } else if (json.isJsonArray()) json.getAsJsonArray().forEach(PresetSurfaceRuleData::expandClassicConditions);
    }

    // maybe have a custom meadow or cherry forest surface ?
    public static SurfaceRules.RuleSource overworld(Preset preset, HolderGetter<DensityFunction> densityFunctions, HolderGetter<Noise> noise, HolderGetter<Biome> biomes) {
        return overworld(preset, densityFunctions, noise, SurfaceRuleData.overworld(biomes));
    }
    
    public static SurfaceRules.RuleSource overworld(Preset preset, HolderGetter<DensityFunction> densityFunctions, HolderGetter<Noise> noise, SurfaceRules.RuleSource defaultRules) {
    	MiscellaneousSettings miscellaneous = preset.miscellaneous();
    	ImmutableList.Builder<SurfaceRules.RuleSource> builder = ImmutableList.builder();
    	builder.add(defaultRules);
		if(miscellaneous.strataDecorator) {
			builder.add(makeStrataRule(noise));
		}
        return SurfaceRules.sequence(builder.build().toArray(SurfaceRules.RuleSource[]::new));
    }
    
	private static SurfaceRules.RuleSource makeStrataRule(HolderGetter<Noise> noise) {
		Holder<Noise> depth = noise.getOrThrow(PresetStrataNoise.STRATA_DEPTH);
		
		List<Strata> strata = new ArrayList<>();
		strata.add(new Strata(RTFBlockTags.SOIL, depth, 3, 0, 1, 0.1F, 0.25F));
		strata.add(new Strata(RTFBlockTags.SEDIMENT, depth, 3, 0, 2, 0.05F, 0.15F));
		strata.add(new Strata(RTFBlockTags.CLAY, depth, 3, 0, 2, 0.05F, 0.1F));
		strata.add(new Strata(RTFBlockTags.ROCK, depth, 3, 10, 30, 0.1F, 1.5F));
		return RTFSurfaceRules.strata(RTFCommon.location("overworld_strata"), noise.getOrThrow(PresetStrataNoise.STRATA_SELECTOR), strata, 100);
	}
}
