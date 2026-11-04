package raccoonman.reterraforged.world.worldgen.feature.template;

import java.util.ArrayList;
import java.util.List;
import java.util.Map;

import com.mojang.serialization.Lifecycle;

import net.minecraft.SharedConstants;
import net.minecraft.core.Holder;
import net.minecraft.core.MappedRegistry;
import net.minecraft.core.registries.Registries;
import net.minecraft.resources.Identifier;
import net.minecraft.resources.ResourceKey;
import net.minecraft.server.Bootstrap;
import net.minecraft.world.level.biome.Biome;
import raccoonman.reterraforged.world.worldgen.feature.template.decorator.DecoratorConfig;
import raccoonman.reterraforged.world.worldgen.feature.template.decorator.TemplateDecorator;
import raccoonman.reterraforged.world.worldgen.feature.template.template.NoopTemplateContext;

/** Exercises the production holder-to-biome override path with real game holders. */
public final class TemplateBiomeDecoratorProbe {
    private static void same(Object actual, Object expected, String message) {
        if (actual != expected) throw new AssertionError(message);
    }

    public static void main(String[] args) throws Exception {
        SharedConstants.tryDetectVersion();
        // Fabric's real registry builder must run before vanilla freezes built-in registries.
        Class.forName("raccoonman.reterraforged.registries.RTFBuiltInRegistries");
        Bootstrap.bootStrap();

        MappedRegistry<Biome> owner = new MappedRegistry<>(Registries.BIOME, Lifecycle.stable());
        Identifier plainsId = Identifier.withDefaultNamespace("plains");
        Identifier forestId = Identifier.withDefaultNamespace("forest");
        Holder<Biome> plains = Holder.Reference.createStandAlone(owner, ResourceKey.create(Registries.BIOME, plainsId));
        Holder<Biome> forest = Holder.Reference.createStandAlone(owner, ResourceKey.create(Registries.BIOME, forestId));
        Holder<Biome> desert = Holder.Reference.createStandAlone(owner, ResourceKey.create(Registries.BIOME, Identifier.withDefaultNamespace("desert")));

        List<TemplateDecorator<NoopTemplateContext>> defaults = new ArrayList<>();
        List<TemplateDecorator<NoopTemplateContext>> plainsOverride = new ArrayList<>();
        List<TemplateDecorator<NoopTemplateContext>> forestOverride = new ArrayList<>();
        DecoratorConfig<NoopTemplateContext> config = new DecoratorConfig<>(defaults, Map.of(plainsId, plainsOverride, forestId, forestOverride));

        same(TemplateFeature.decoratorsForBiome(config, plains), plainsOverride, "Plains must select its biome override");
        same(TemplateFeature.decoratorsForBiome(config, forest), forestOverride, "Forest must select a distinct override");
        same(TemplateFeature.decoratorsForBiome(config, desert), defaults, "Unlisted biomes must retain the default decorator list");
        System.out.println("TEMPLATE_BIOME_DECORATOR_QA=pass assertions=3");
    }
}
