package raccoonman.reterraforged.world.worldgen.feature.template;

import java.util.function.BiPredicate;

import net.minecraft.core.BlockPos;
import net.minecraft.core.registries.Registries;
import net.minecraft.resources.Identifier;
import net.minecraft.tags.BlockTags;
import net.minecraft.tags.TagKey;
import net.minecraft.world.level.BlockGetter;
import net.minecraft.world.level.LevelAccessor;
import net.minecraft.world.level.block.Blocks;
import net.minecraft.world.level.block.Block;
import net.minecraft.world.level.block.state.BlockState;
import net.minecraft.world.level.levelgen.feature.TreeFeature;
import net.minecraft.world.level.pathfinder.PathComputationType;

public class BlockUtils {

    private static final TagKey<Block> SAPLINGS = TagKey.create(Registries.BLOCK, Identifier.withDefaultNamespace("saplings"));

    public static boolean isSoil(LevelAccessor world, BlockPos pos) {
        return isLegacySoil(world.getBlockState(pos));
    }

    public static boolean isLeavesOrLogs(BlockState state) {
        return state.is(BlockTags.LOGS) || state.is(BlockTags.LEAVES);
    }

    public static boolean isVegetation(LevelAccessor world, BlockPos pos) {
        BlockState state = world.getBlockState(pos);
        return state.is(SAPLINGS) || state.is(BlockTags.FLOWERS) || state.is(Blocks.VINE);
    }

    public static boolean canTreeReplace(LevelAccessor world, BlockPos pos) {
        return TreeFeature.isAirOrLeaves(world, pos) || isVegetation(world, pos);
    }

    public static boolean isSolid(BlockGetter reader, BlockPos pos) {
        BlockState state = reader.getBlockState(pos);
        return isSolid(state, reader, pos);
    }

    public static boolean isSolid(BlockState state, BlockGetter reader, BlockPos pos) {
        return state.canOcclude() || !state.isPathfindable(PathComputationType.LAND);
    }

    public static boolean isSoilOrRock(LevelAccessor world, BlockPos pos) {
        BlockState block = world.getBlockState(pos);
        return isLegacySoil(block) || block.is(BlockTags.BASE_STONE_OVERWORLD);
    }

    private static boolean isLegacySoil(BlockState state) {
        // The 26.2 dirt tag no longer includes the other legacy tree soils.
        return state.is(BlockTags.DIRT) || state.is(Blocks.GRASS_BLOCK) || state.is(Blocks.PODZOL)
                || state.is(Blocks.MYCELIUM) || state.is(Blocks.MOSS_BLOCK) || state.is(Blocks.MUD)
                || state.is(Blocks.MUDDY_MANGROVE_ROOTS);
    }

    public static boolean isClearOverhead(LevelAccessor world, BlockPos pos, int height, BiPredicate<LevelAccessor, BlockPos> predicate) {
        BlockPos.MutableBlockPos mutable = new BlockPos.MutableBlockPos();
        // world.getMaxHeight ?
        int max = Math.min(world.getMaxY(), pos.getY() + height);
        for (int y = pos.getY(); y < max; y++) {
            mutable.set(pos.getX(), y, pos.getZ());
            if (!predicate.test(world, mutable)) {
                return false;
            }
        }
        return true;
    }
}
