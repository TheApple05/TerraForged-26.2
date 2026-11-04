package raccoonman.reterraforged.platform;

import raccoonman.reterraforged.platform.fabric.ModLoaderUtilImpl;

public class ModLoaderUtil {
	
	public static boolean isLoaded(String modId) {
		return ModLoaderUtilImpl.isLoaded(modId);
	}
}
