package terraforged.qa;

import com.google.gson.JsonParser;
import com.mojang.serialization.JsonOps;
import java.nio.file.*;
import java.util.List;
import net.fabricmc.api.ModInitializer;
import net.fabricmc.fabric.api.event.lifecycle.v1.ServerLifecycleEvents;
import raccoonman.reterraforged.concurrent.cache.CacheManager;
import raccoonman.reterraforged.data.worldgen.preset.settings.Preset;
import raccoonman.reterraforged.data.worldgen.preset.settings.SpawnType;
import raccoonman.reterraforged.registries.RTFRegistries;
import raccoonman.reterraforged.world.worldgen.GeneratorContext;

/** Actual server registry/context and packaged spawn methods, no Rust expectations. */
public final class NativeSpawnCaptureMod implements ModInitializer {
    @Override public void onInitialize() {
        ServerLifecycleEvents.SERVER_STARTED.register(server -> {
            try {
                Path jar=Path.of(System.getProperty("terraforged.qa.production")).toRealPath();
                for(Class<?> type:List.of(SpawnType.class,GeneratorContext.class,Preset.class)) {
                    Path origin=Path.of(type.getProtectionDomain().getCodeSource().getLocation().toURI()).toRealPath();
                    if(!jar.equals(origin))throw new AssertionError("Wrong class origin: "+type);
                    System.out.println("SPAWN_PACKAGED_CLASS_ORIGIN="+type.getName()+"="+origin);
                }
                var input=JsonParser.parseString(Files.readString(Path.of(System.getProperty("terraforged.qa.inputs")))).getAsJsonObject();
                Preset preset=Preset.DIRECT_CODEC.parse(JsonOps.INSTANCE,input.get("preset")).getOrThrow();
                var noises=server.registryAccess().lookupOrThrow(RTFRegistries.NOISE);
                int owners=CacheManager.activeCacheCount();StringBuilder out=new StringBuilder();int count=0;
                for(long seed:new long[]{0,1,-1,8675309,2147483647L,-2147483648L,4294967296L,4294967297L}) {
                    try(var context=GeneratorContext.makeCached(preset.copy(),noises,(int)seed,3,6,false)) {
                        for(SpawnType mode:SpawnType.values()) {
                            var center=mode.getSearchCenter(context);
                            out.append(seed).append('/').append(mode.getSerializedName()).append('\t').append(center.getX()).append(',').append(center.getZ()).append('\n');count++;
                        }
                    }
                }
                if(CacheManager.activeCacheCount()!=owners)throw new AssertionError("Spawn reference leaked managed owners");
                Files.writeString(Path.of(System.getProperty("terraforged.qa.output")),out,StandardOpenOption.CREATE_NEW);
                System.out.println("NATIVE_SPAWN_CAPTURE_OK records="+count+" pid="+ProcessHandle.current().pid());
            } catch(Throwable error) {error.printStackTrace();System.err.println("NATIVE_SPAWN_CAPTURE_FAILED");}
            finally {server.halt(false);}
        });
    }
}
