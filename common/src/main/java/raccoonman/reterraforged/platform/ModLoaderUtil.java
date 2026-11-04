package raccoonman.reterraforged.platform;


public class ModLoaderUtil {
	
	public static boolean isLoaded(String modId) {
		return PlatformServices.get().isModLoaded(modId);
	}
}
