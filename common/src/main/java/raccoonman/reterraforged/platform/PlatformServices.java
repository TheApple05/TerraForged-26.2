package raccoonman.reterraforged.platform;

import java.nio.file.Path;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.ServiceLoader;
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
import net.minecraft.resources.ResourceKey;
import net.minecraft.world.level.biome.Biome;
import net.minecraft.world.level.levelgen.GenerationStep;
import net.minecraft.world.level.levelgen.placement.PlacedFeature;
import raccoonman.reterraforged.world.worldgen.biome.modifier.BiomeModifier;
import raccoonman.reterraforged.world.worldgen.biome.modifier.Filter;
import raccoonman.reterraforged.world.worldgen.biome.modifier.Order;

/** One loader-owned provider is packaged with each artifact. */
public interface PlatformServices {
    static PlatformServices get() {
        return ProviderHolder.INSTANCE;
    }

    String loaderName();
    Path getConfigPath();
    boolean isModLoaded(String modId);

    <T> Registry<T> createRegistry(ResourceKey<? extends Registry<T>> key);
    <T> void register(Registry<T> registry, ResourceKey<T> key, T value);
    <T> WritableRegistry<T> getWritable(Registry<T> registry);
    <T> void createDataRegistry(ResourceKey<? extends Registry<T>> key, Codec<T> codec);
    DataProvider createRegistryProvider(PackOutput output, CompletableFuture<HolderLookup.Provider> providerLookup);

    ResourceKey<Registry<MapCodec<? extends BiomeModifier>>> biomeModifierTypeKey();
    ResourceKey<Registry<BiomeModifier>> biomeModifierRegistryKey();
    void addPlatformRegistryCloners(Cloner.Factory cloners);
    void bootstrapBiomeModifiers();
    BiomeModifier addBiomeModifier(Order order, GenerationStep.Decoration step,
        Optional<Pair<Filter.Behavior, HolderSet<Biome>>> biomes, HolderSet<PlacedFeature> features);
    BiomeModifier replaceBiomeModifier(GenerationStep.Decoration step, Optional<HolderSet<Biome>> biomes,
        Map<ResourceKey<PlacedFeature>, Holder<PlacedFeature>> replacements);

    final class ProviderHolder {
        private static final PlatformServices INSTANCE = load();

        private ProviderHolder() {
        }

        private static PlatformServices load() {
            List<ServiceLoader.Provider<PlatformServices>> providers = ServiceLoader
                .load(PlatformServices.class, PlatformServices.class.getClassLoader()).stream().toList();
            if (providers.size() != 1) {
                throw new IllegalStateException("Expected exactly one TerraForged platform provider, found "
                    + providers.stream().map(provider -> provider.type().getName()).toList());
            }
            return providers.getFirst().get();
        }
    }
}
