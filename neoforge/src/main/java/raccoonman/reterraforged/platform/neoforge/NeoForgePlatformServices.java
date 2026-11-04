package raccoonman.reterraforged.platform.neoforge;

import java.nio.file.Path;
import java.util.ArrayList;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.concurrent.CompletableFuture;
import com.mojang.datafixers.util.Pair;
import com.mojang.serialization.Codec;
import com.mojang.serialization.MapCodec;
import net.minecraft.core.Cloner;
import net.minecraft.core.Holder;
import net.minecraft.core.HolderLookup;
import net.minecraft.core.Registry;
import net.minecraft.core.WritableRegistry;
import net.minecraft.core.HolderSet;
import net.minecraft.data.DataProvider;
import net.minecraft.data.PackOutput;
import net.minecraft.data.registries.RegistriesDatapackGenerator;
import net.minecraft.resources.ResourceKey;
import net.minecraft.world.level.biome.Biome;
import net.minecraft.world.level.levelgen.GenerationStep;
import net.minecraft.world.level.levelgen.placement.PlacedFeature;
import net.neoforged.bus.api.IEventBus;
import net.neoforged.fml.loading.FMLLoader;
import net.neoforged.fml.loading.FMLPaths;
import net.neoforged.neoforge.registries.DataPackRegistryEvent;
import net.neoforged.neoforge.registries.NeoForgeRegistries;
import net.neoforged.neoforge.registries.NewRegistryEvent;
import net.neoforged.neoforge.registries.RegisterEvent;
import net.neoforged.neoforge.registries.RegistryBuilder;
import raccoonman.reterraforged.platform.PlatformServices;
import raccoonman.reterraforged.world.worldgen.biome.modifier.BiomeModifier;
import raccoonman.reterraforged.world.worldgen.biome.modifier.Filter;
import raccoonman.reterraforged.world.worldgen.biome.modifier.Order;
import raccoonman.reterraforged.world.worldgen.biome.modifier.neoforge.NeoBiomeModifiers;

/** Queues common registration calls until NeoForge opens the matching registry. */
public final class NeoForgePlatformServices implements PlatformServices {
    private final List<Registry<?>> customRegistries = new ArrayList<>();
    private final Map<ResourceKey<? extends Registry<?>>, List<Pending<?>>> pending = new LinkedHashMap<>();
    private final List<DataRegistry<?>> dataRegistries = new ArrayList<>();

    public void attach(IEventBus bus) {
        bus.addListener(this::newRegistries);
        bus.addListener(this::registerEntries);
        bus.addListener(this::dataRegistries);
    }

    private void newRegistries(NewRegistryEvent event) {
        customRegistries.forEach(event::register);
    }

    private void registerEntries(RegisterEvent event) {
        List<Pending<?>> entries = pending.remove(event.getRegistryKey());
        if (entries != null) entries.forEach(entry -> entry.apply(event));
    }

    private void dataRegistries(DataPackRegistryEvent.NewRegistry event) {
        dataRegistries.forEach(registry -> registry.apply(event));
    }

    @Override public String loaderName() { return "neoforge"; }
    @Override public Path getConfigPath() { return FMLPaths.CONFIGDIR.get(); }
    @Override public boolean isModLoaded(String id) {
        // Mixin selection runs before ModList is constructed.
        return FMLLoader.getCurrent().getLoadingModList().getModFileById(id) != null;
    }

    @Override @SuppressWarnings({"unchecked", "rawtypes"})
    public <T> Registry<T> createRegistry(ResourceKey<? extends Registry<T>> key) {
        if (key.equals(NeoForgeRegistries.Keys.BIOME_MODIFIER_SERIALIZERS)) {
            // Only this mod's entries cross this boundary, and each implements both interfaces.
            return (Registry) NeoForgeRegistries.BIOME_MODIFIER_SERIALIZERS;
        }
        Registry<T> registry = new RegistryBuilder<>(key).create();
        customRegistries.add(registry);
        return registry;
    }

    @Override public <T> void register(Registry<T> registry, ResourceKey<T> key, T value) {
        pending.computeIfAbsent(registry.key(), unused -> new ArrayList<>())
            .add(new Pending<>(registry.key(), key, value));
    }

    @Override public <T> WritableRegistry<T> getWritable(Registry<T> registry) {
        return (WritableRegistry<T>) registry;
    }

    @Override public <T> void createDataRegistry(ResourceKey<? extends Registry<T>> key, Codec<T> codec) {
        dataRegistries.add(new DataRegistry<>(key, codec));
    }

    @Override public DataProvider createRegistryProvider(PackOutput output, CompletableFuture<HolderLookup.Provider> lookup) {
        // NeoForge extends this provider to include its complete datapack registry list.
        return new RegistriesDatapackGenerator(output, lookup);
    }

    @Override @SuppressWarnings({"unchecked", "rawtypes"})
    public ResourceKey<Registry<MapCodec<? extends BiomeModifier>>> biomeModifierTypeKey() {
        return (ResourceKey) NeoForgeRegistries.Keys.BIOME_MODIFIER_SERIALIZERS;
    }

    @Override @SuppressWarnings({"unchecked", "rawtypes"})
    public ResourceKey<Registry<BiomeModifier>> biomeModifierRegistryKey() {
        return (ResourceKey) NeoForgeRegistries.Keys.BIOME_MODIFIERS;
    }

    @Override public void addPlatformRegistryCloners(Cloner.Factory cloners) {
        // RegistrySetBuilder clones the complete original provider. Retain every
        // loader/mod registry with its real registered codec, including native
        // biome and structure modifiers belonging to other mods.
        net.neoforged.neoforge.registries.DataPackRegistriesHooks.getDataPackRegistriesWithDimensions()
            .forEach(data -> data.runWithArguments(cloners::addCodec));
    }

    @Override public void bootstrapBiomeModifiers() { NeoBiomeModifiers.bootstrap(); }
    @Override public BiomeModifier addBiomeModifier(Order order, GenerationStep.Decoration step,
            Optional<Pair<Filter.Behavior, HolderSet<Biome>>> biomes, HolderSet<PlacedFeature> features) {
        return NeoBiomeModifiers.add(order, step, biomes, features);
    }
    @Override public BiomeModifier replaceBiomeModifier(GenerationStep.Decoration step,
            Optional<HolderSet<Biome>> biomes, Map<ResourceKey<PlacedFeature>, Holder<PlacedFeature>> replacements) {
        return NeoBiomeModifiers.replace(step, biomes, replacements);
    }

    private record Pending<T>(ResourceKey<? extends Registry<T>> registry, ResourceKey<T> key, T value) {
        void apply(RegisterEvent event) { event.register(registry, key.identifier(), () -> value); }
    }
    private record DataRegistry<T>(ResourceKey<? extends Registry<T>> key, Codec<T> codec) {
        void apply(DataPackRegistryEvent.NewRegistry event) {
            event.dataPackRegistry(ResourceKey.createRegistryKey(key.identifier()), codec);
        }
    }
}
