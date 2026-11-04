package raccoonman.reterraforged.data.worldgen.preset;

import com.mojang.serialization.Codec;
import com.mojang.serialization.codecs.RecordCodecBuilder;

import net.minecraft.core.HolderLookup;
import net.minecraft.core.Registry;
import net.minecraft.core.RegistryAccess;
import net.minecraft.core.RegistrySetBuilder;
import net.minecraft.core.registries.BuiltInRegistries;
import net.minecraft.core.registries.Registries;
import net.minecraft.data.worldgen.BootstrapContext;
import net.minecraft.resources.ResourceKey;
import raccoonman.reterraforged.data.worldgen.BiomeModifierData;
import raccoonman.reterraforged.data.worldgen.NoiseData;
import raccoonman.reterraforged.data.worldgen.RTFBiomeData;
import raccoonman.reterraforged.data.worldgen.RTFConfiguredCarvers;
import raccoonman.reterraforged.data.worldgen.RTFConfiguredFeatures;
import raccoonman.reterraforged.data.worldgen.RTFDimensionTypes;
import raccoonman.reterraforged.data.worldgen.RTFNoiseGeneratorSettings;
import raccoonman.reterraforged.data.worldgen.RTFNoiseRouterData;
import raccoonman.reterraforged.data.worldgen.RTFPlacedFeatures;
import raccoonman.reterraforged.data.worldgen.StructureRuleData;
import raccoonman.reterraforged.registries.RTFRegistries;

public record Preset(WorldSettings world, CaveSettings caves, ClimateSettings climate, TerrainSettings terrain, RiverSettings rivers, FilterSettings filters, StructureSettings structures, MiscellaneousSettings miscellaneous) {
	public static final Codec<Preset> DIRECT_CODEC = RecordCodecBuilder.create(instance -> instance.group(
		WorldSettings.CODEC.fieldOf("world").forGetter(Preset::world),
		CaveSettings.CODEC.optionalFieldOf("caves", new CaveSettings(0.0F, 1.5625F, 1.0F, 1.0F, 1.0F, 0.14285715F, 0.07F, 0.02F, true, false)).forGetter((o) -> o.caves),
		ClimateSettings.CODEC.fieldOf("climate").forGetter(Preset::climate),
		TerrainSettings.CODEC.fieldOf("terrain").forGetter(Preset::terrain),
		RiverSettings.CODEC.fieldOf("rivers").forGetter(Preset::rivers),
		FilterSettings.CODEC.fieldOf("filters").forGetter(Preset::filters),
		StructureSettings.CODEC.fieldOf("structures").forGetter(Preset::structures),
		MiscellaneousSettings.CODEC.fieldOf("miscellaneous").forGetter(Preset::miscellaneous)
	).apply(instance, Preset::new));
	
	@Deprecated
	public static final ResourceKey<raccoonman.reterraforged.data.worldgen.preset.settings.Preset> KEY = raccoonman.reterraforged.data.worldgen.preset.settings.Preset.KEY;
	
	public Preset copy() {
		return new Preset(this.world.copy(), this.caves.copy(), this.climate.copy(), this.terrain.copy(), this.rivers.copy(), this.filters.copy(), this.structures.copy(), this.miscellaneous.copy());
	}

	/** Compatibility bridge for the superseded duplicate settings package. */
	public HolderLookup.Provider buildPatch(RegistryAccess registries) {
		return this.toCurrentPreset().buildPatch(registries);
	}

	public raccoonman.reterraforged.data.worldgen.preset.settings.Preset toCurrentPreset() {
		com.google.gson.JsonElement source = DIRECT_CODEC.encodeStart(com.mojang.serialization.JsonOps.INSTANCE, this).getOrThrow();
		var converted = raccoonman.reterraforged.data.worldgen.preset.settings.Preset.DIRECT_CODEC.parse(com.mojang.serialization.JsonOps.INSTANCE, source).getOrThrow();
		com.google.gson.JsonElement convertedJson = raccoonman.reterraforged.data.worldgen.preset.settings.Preset.DIRECT_CODEC.encodeStart(com.mojang.serialization.JsonOps.INSTANCE, converted).getOrThrow();
		Preset roundTrip = DIRECT_CODEC.parse(com.mojang.serialization.JsonOps.INSTANCE, convertedJson).getOrThrow();
		if (!source.equals(DIRECT_CODEC.encodeStart(com.mojang.serialization.JsonOps.INSTANCE, roundTrip).getOrThrow())) {
			throw new IllegalStateException("Legacy preset conversion would lose settings");
		}
		return converted;
	}
}
