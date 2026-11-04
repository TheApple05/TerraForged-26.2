package raccoonman.reterraforged.platform.fabric;

import java.nio.file.Path;
import java.util.Map;
import java.util.Optional;
import java.util.concurrent.CompletableFuture;
import com.mojang.datafixers.util.Pair;
import com.mojang.serialization.Codec;
import com.mojang.serialization.MapCodec;
import net.minecraft.core.Cloner;
import net.minecraft.core.Holder;
import net.minecraft.core.HolderLookup;
import net.minecraft.core.HolderSet;
import net.minecraft.core.Registry;
import net.minecraft.core.WritableRegistry;
import net.minecraft.data.DataProvider;
import net.minecraft.data.PackOutput;
import net.minecraft.resources.Identifier;
import net.minecraft.resources.ResourceKey;
import net.minecraft.world.level.biome.Biome;
import net.minecraft.world.level.levelgen.GenerationStep;
import net.minecraft.world.level.levelgen.placement.PlacedFeature;
import raccoonman.reterraforged.platform.PlatformServices;
import raccoonman.reterraforged.registries.RTFRegistries;
import raccoonman.reterraforged.world.worldgen.biome.modifier.BiomeModifier;
import raccoonman.reterraforged.world.worldgen.biome.modifier.Filter;
import raccoonman.reterraforged.world.worldgen.biome.modifier.Order;
import raccoonman.reterraforged.world.worldgen.biome.modifier.fabric.BiomeModifiersImpl;

public final class FabricPlatformServices implements PlatformServices {
    @Override public String loaderName() { return "fabric"; }
    @Override public Path getConfigPath() { return ConfigUtilImpl.getConfigPath(); }
    @Override public boolean isModLoaded(String modId) { return ModLoaderUtilImpl.isLoaded(modId); }
    @Override public <T> Registry<T> createRegistry(ResourceKey<? extends Registry<T>> key) {
        return RegistryUtilImpl.createRegistry(key);
    }
    @Override public <T> void register(Registry<T> registry, ResourceKey<T> key, T value) {
        RegistryUtilImpl.getWritable(registry).register(key, value, net.minecraft.core.RegistrationInfo.BUILT_IN);
    }
    @Override public <T> WritableRegistry<T> getWritable(Registry<T> registry) {
        return RegistryUtilImpl.getWritable(registry);
    }
    @Override public <T> void createDataRegistry(ResourceKey<? extends Registry<T>> key, Codec<T> codec) {
        RegistryUtilImpl.createDataRegistry(key, codec);
    }
    @Override public DataProvider createRegistryProvider(PackOutput output, CompletableFuture<HolderLookup.Provider> lookup) {
        return DataGenUtilImpl.createRegistryProvider(output, lookup);
    }
    @Override public ResourceKey<Registry<MapCodec<? extends BiomeModifier>>> biomeModifierTypeKey() {
        // Retain the already-tested Fabric registry identities for preset compatibility.
        return ResourceKey.createRegistryKey(Identifier.parse("forge:biome_modifier_serializers"));
    }
    @Override public ResourceKey<Registry<BiomeModifier>> biomeModifierRegistryKey() {
        return ResourceKey.createRegistryKey(Identifier.parse("forge:biome_modifier"));
    }
    @Override public void addPlatformRegistryCloners(Cloner.Factory cloners) {
        net.fabricmc.fabric.api.event.registry.DynamicRegistries.getDynamicRegistries()
            .forEach(data -> data.runWithArguments(cloners::addCodec));
    }
    @Override public void bootstrapBiomeModifiers() { BiomeModifiersImpl.bootstrap(); }
    @Override public BiomeModifier addBiomeModifier(Order order, GenerationStep.Decoration step,
            Optional<Pair<Filter.Behavior, HolderSet<Biome>>> biomes, HolderSet<PlacedFeature> features) {
        return BiomeModifiersImpl.add(order, step, biomes, features);
    }
    @Override public BiomeModifier replaceBiomeModifier(GenerationStep.Decoration step, Optional<HolderSet<Biome>> biomes,
            Map<ResourceKey<PlacedFeature>, Holder<PlacedFeature>> replacements) {
        return BiomeModifiersImpl.replace(step, biomes, replacements);
    }
}
