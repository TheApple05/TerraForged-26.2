package raccoonman.reterraforged.data.worldgen;

import java.util.Optional;
import net.minecraft.core.registries.Registries;
import net.minecraft.data.worldgen.BootstrapContext;
import net.minecraft.data.worldgen.biome.OverworldBiomes;
import net.minecraft.tags.BlockTags;
import net.minecraft.tags.TimelineTags;
import net.minecraft.util.ARGB;
import net.minecraft.util.valueproviders.UniformInt;
import net.minecraft.world.attribute.AmbientSounds;
import net.minecraft.world.attribute.BackgroundMusic;
import net.minecraft.world.attribute.BedRule;
import net.minecraft.world.attribute.EnvironmentAttributeMap;
import net.minecraft.world.attribute.EnvironmentAttributes;
import net.minecraft.world.clock.WorldClocks;
import net.minecraft.world.level.CardinalLighting;
import net.minecraft.world.level.dimension.DimensionType;

/** Vanilla 26.2 overworld behavior with TerraForged's configured vertical bounds. */
public final class OverworldDimensionData {
    private OverworldDimensionData() {
    }

    public static DimensionType create(BootstrapContext<DimensionType> context, int worldDepth, int worldHeight) {
        int totalHeight = Math.addExact(worldDepth, worldHeight);
        EnvironmentAttributeMap.Builder attributes = EnvironmentAttributeMap.builder();
        attributes.set(EnvironmentAttributes.BACKGROUND_MUSIC, BackgroundMusic.OVERWORLD);
        attributes.set(EnvironmentAttributes.AMBIENT_SOUNDS, AmbientSounds.LEGACY_CAVE_SETTINGS);
        attributes.set(EnvironmentAttributes.BED_RULE, BedRule.CAN_SLEEP_WHEN_DARK);
        attributes.set(EnvironmentAttributes.NETHER_PORTAL_SPAWNS_PIGLINS, true);
        attributes.set(EnvironmentAttributes.RESPAWN_ANCHOR_WORKS, false);
        attributes.set(EnvironmentAttributes.FOG_COLOR, 0xffc0d8ff);
        attributes.set(EnvironmentAttributes.SKY_COLOR, OverworldBiomes.calculateSkyColor(0.8F));
        attributes.set(EnvironmentAttributes.AMBIENT_LIGHT_COLOR, 0xff0a0a0a);
        attributes.set(EnvironmentAttributes.CLOUD_COLOR, ARGB.white(0.8F));
        attributes.set(EnvironmentAttributes.CLOUD_HEIGHT, 192.33F);
        return new DimensionType(
            false, true, false, false, 1.0,
            -worldDepth, totalHeight, totalHeight,
            context.lookup(Registries.BLOCK).getOrThrow(BlockTags.INFINIBURN_OVERWORLD),
            0.0F, new DimensionType.MonsterSettings(UniformInt.of(0, 7), 0),
            DimensionType.Skybox.OVERWORLD, CardinalLighting.Type.DEFAULT,
            attributes.build(), context.lookup(Registries.TIMELINE).getOrThrow(TimelineTags.IN_OVERWORLD),
            Optional.of(context.lookup(Registries.WORLD_CLOCK).getOrThrow(WorldClocks.OVERWORLD))
        );
    }
}
