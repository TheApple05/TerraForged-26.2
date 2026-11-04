package raccoonman.reterraforged.platform;

import java.util.concurrent.CompletableFuture;

import net.minecraft.core.HolderLookup;
import net.minecraft.data.DataProvider;
import net.minecraft.data.PackOutput;

public final class DataGenUtil {

	public static DataProvider createRegistryProvider(PackOutput output, CompletableFuture<HolderLookup.Provider> providerLookup) {
		return PlatformServices.get().createRegistryProvider(output, providerLookup);
	}
}
