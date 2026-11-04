import java.nio.file.Path;
import net.minecraft.SharedConstants;
import net.minecraft.data.CachedOutput;
import net.minecraft.data.PackOutput;
import net.minecraft.server.Bootstrap;
import raccoonman.reterraforged.client.data.RTFLanguageProvider;

/** Executes the production language provider; no hand-copied translation table. */
public final class LanguageAssetGenerator {
    public static void main(String[] args) {
        SharedConstants.tryDetectVersion();
        Bootstrap.bootStrap();
        new RTFLanguageProvider.EnglishUS(new PackOutput(Path.of(args[0])))
            .run(CachedOutput.NO_CACHE).join();
        Bootstrap.shutdownStdout();
    }
}
