package raccoonman.reterraforged.platform;

import com.mojang.serialization.Codec;

import net.minecraft.core.RegistrationInfo;
import net.minecraft.core.Registry;
import net.minecraft.core.WritableRegistry;
import net.minecraft.resources.ResourceKey;
import raccoonman.reterraforged.platform.fabric.RegistryUtilImpl;
import raccoonman.reterraforged.registries.RTFRegistries;

@Deprecated
public final class RegistryUtil {
	
	public static <T> void register(Registry<T> registry, String name, T value) {
		getWritable(registry).register(RTFRegistries.createKey(registry.key(), name), value, RegistrationInfo.BUILT_IN);
	}
	
	public static <T> WritableRegistry<T> getWritable(Registry<T> registry) {
		return RegistryUtilImpl.getWritable(registry);
	}
	
	public static <T> Registry<T> createRegistry(ResourceKey<? extends Registry<T>> key) {
		return RegistryUtilImpl.createRegistry(key);
	}

	public static <T> void createDataRegistry(ResourceKey<? extends Registry<T>> key, Codec<T> codec) {
		RegistryUtilImpl.createDataRegistry(key, codec);
	}
}
