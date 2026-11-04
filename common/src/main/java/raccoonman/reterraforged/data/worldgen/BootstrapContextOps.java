package raccoonman.reterraforged.data.worldgen;

import java.util.Optional;
import com.google.gson.JsonElement;
import com.mojang.serialization.JsonOps;
import com.mojang.serialization.Lifecycle;
import net.minecraft.core.HolderOwner;
import net.minecraft.core.Registry;
import net.minecraft.data.worldgen.BootstrapContext;
import net.minecraft.resources.RegistryOps;
import net.minecraft.resources.ResourceKey;

/** Decode-only context: new holders belong to RegistrySetBuilder's patch owner. */
public final class BootstrapContextOps {
    private BootstrapContextOps() {
    }

    public static RegistryOps<JsonElement> forDecoding(BootstrapContext<?> context) {
        return RegistryOps.create(JsonOps.INSTANCE, new RegistryOps.RegistryInfoLookup() {
            @Override
            public <T> Optional<RegistryOps.RegistryInfo<T>> lookup(ResourceKey<? extends Registry<? extends T>> key) {
                return Optional.of(new RegistryOps.RegistryInfo<>(new HolderOwner<T>() {}, context.lookup(key), Lifecycle.stable()));
            }
        });
    }
}
