package raccoonman.reterraforged.platform;

import com.mojang.serialization.Codec;

import net.minecraft.core.Registry;
import net.minecraft.core.WritableRegistry;
import net.minecraft.resources.ResourceKey;
import raccoonman.reterraforged.registries.RTFRegistries;

@Deprecated
public final class RegistryUtil {
	
	public static <T> void register(Registry<T> registry, String name, T value) {
		PlatformServices.get().register(registry, RTFRegistries.createKey(registry.key(), name), value);
	}
	
	public static <T> WritableRegistry<T> getWritable(Registry<T> registry) {
		return PlatformServices.get().getWritable(registry);
	}
	
	public static <T> Registry<T> createRegistry(ResourceKey<? extends Registry<T>> key) {
		return PlatformServices.get().createRegistry(key);
	}

	public static <T> void createDataRegistry(ResourceKey<? extends Registry<T>> key, Codec<T> codec) {
		PlatformServices.get().createDataRegistry(key, codec);
	}
}
