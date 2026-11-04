package raccoonman.reterraforged.world.worldgen.densityfunction;

import java.util.function.Supplier;
import java.lang.ref.WeakReference;

import org.jetbrains.annotations.Nullable;

import com.mojang.serialization.Codec;
import com.mojang.serialization.MapCodec;
import com.mojang.serialization.codecs.RecordCodecBuilder;

import net.minecraft.core.SectionPos;
import net.minecraft.util.KeyDispatchDataCodec;
import net.minecraft.util.StringRepresentable;
import net.minecraft.world.level.levelgen.DensityFunction;
import raccoonman.reterraforged.world.worldgen.biome.Continentalness;
import raccoonman.reterraforged.world.worldgen.cell.Cell;
import raccoonman.reterraforged.world.worldgen.cell.heightmap.ControlPoints;
import raccoonman.reterraforged.world.worldgen.cell.heightmap.Heightmap;
import raccoonman.reterraforged.world.worldgen.cell.heightmap.Levels;
import raccoonman.reterraforged.world.worldgen.cell.heightmap.WorldLookup;
import raccoonman.reterraforged.world.worldgen.cell.terrain.TerrainCategory;
import raccoonman.reterraforged.world.worldgen.densityfunction.CellSampler.Field;
import raccoonman.reterraforged.world.worldgen.densityfunction.tile.Tile;
import raccoonman.reterraforged.world.worldgen.noise.NoiseUtil;
import raccoonman.reterraforged.world.worldgen.util.PosUtil;

public record CellSampler(Supplier<WorldLookup> deferredLookup, Field field) implements MarkerFunction.Mapped {
	private static final ThreadLocal<Cache2d> CELL = ThreadLocal.withInitial(Cache2d::new);
    private static final ThreadLocal<Cache2d> ACCURATE_CELL = ThreadLocal.withInitial(() -> new Cache2d(1, true));
    private static final ThreadLocal<Integer> ACCURATE_DEPTH = ThreadLocal.withInitial(() -> 0);

    /** Scope exact parent-biome resolution without making spawn heuristics generate terrain tiles. */
    public static <T> T withAccurateSampling(Supplier<T> action) {
        int previous = ACCURATE_DEPTH.get();
        ACCURATE_DEPTH.set(previous + 1);
        try {
            return action.get();
        } finally {
            if (previous == 0) ACCURATE_DEPTH.remove();
            else ACCURATE_DEPTH.set(previous);
        }
    }
	
	@Override
	public double compute(FunctionContext ctx) {
		WorldLookup worldLookup = this.deferredLookup.get();
		Cell cell = (ACCURATE_DEPTH.get() > 0 ? ACCURATE_CELL : CELL).get().getAndUpdate(worldLookup, ctx.blockX(), ctx.blockZ(), true);
		return this.field.read(cell, worldLookup.getHeightmap());
	}

	@Override
	public double minValue() {
		return 0.0F;
	}

	@Override
	public double maxValue() {
		return 1.0F;
	}

	public static class Cache2d {
        private final long[] positions;
        private final Cell[] cells;
        private final boolean accurate;
        private WeakReference<WorldLookup> owner = new WeakReference<>(null);
        private boolean climate;
        private int size;
        private int next;

        /** The global caller only needs one entry, without retaining its world. */
        public Cache2d() { this(1, false); }

        /** Small per-NoiseChunk cache; four interpolation corners must survive Y changes. */
        public Cache2d(int capacity) { this(capacity, false); }

        private Cache2d(int capacity, boolean accurate) {
            if (capacity < 1 || capacity > 64) throw new IllegalArgumentException("Cell cache capacity must be 1..64");
            this.positions = new long[capacity];
            this.cells = new Cell[capacity];
            this.accurate = accurate;
        }

        public synchronized Cell getAndUpdate(WorldLookup lookup, int blockX, int blockZ, boolean sampleClimate) {
            this.bind(lookup, sampleClimate);
            long packedPos = PosUtil.pack(blockX, blockZ);
            for (int index = 0; index < this.size; index++) {
                if (this.positions[index] == packedPos) return this.cells[index];
            }
            // Future helping can re-enter this synchronized cache while applyCell
            // waits. Keep the unfinished cell private, including the one-entry cache.
            Cell cell = new Cell();
            try {
                // Global biome queries must agree before/after terrain generation
                // and reload. Native one-column structure samplers retain their
                // deliberately approximate, bounded per-NoiseChunk path.
                lookup.applyCell(cell, blockX, blockZ, this.accurate, sampleClimate);
            } catch (RuntimeException | Error failure) {
                this.size = this.next = 0;
                throw failure;
            }
            // Nested work may have changed the lookup, mode, size or insertion
            // cursor. Revalidate and choose a slot only after computation finishes.
            this.bind(lookup, sampleClimate);
            for (int index = 0; index < this.size; index++) {
                if (this.positions[index] == packedPos) {
                    this.cells[index] = cell;
                    return cell;
                }
            }
            int index = this.size < this.positions.length ? this.size : this.next;
            this.positions[index] = packedPos;
            this.cells[index] = cell;
            if (this.size < this.positions.length) this.size++;
            this.next = (index + 1) % this.positions.length;
            // Snapshots are never reset while another reader may still hold one.
            return cell;
        }

        private void bind(WorldLookup lookup, boolean sampleClimate) {
            if (this.owner.get() != lookup || this.climate != sampleClimate) {
                this.owner = new WeakReference<>(lookup);
                this.climate = sampleClimate;
                this.size = this.next = 0;
            }
        }
	}
	
	public class CacheChunk implements MarkerFunction.Mapped {
		@Nullable
		private Tile.Chunk chunk;
		private Cache2d cache2d;
		private int chunkX, chunkZ;
		
		public CacheChunk(@Nullable Tile.Chunk chunk, @Nullable Cache2d cache2d, int chunkX, int chunkZ) {
			this.chunk = chunk;
			this.cache2d = cache2d != null ? cache2d : new Cache2d(16);
			this.chunkX = chunkX;
			this.chunkZ = chunkZ;
		}

		@Override
		public double compute(FunctionContext ctx) {
			int blockX = ctx.blockX();
			int blockZ = ctx.blockZ();
			int chunkX = SectionPos.blockToSectionCoord(blockX);
			int chunkZ = SectionPos.blockToSectionCoord(blockZ);
			WorldLookup worldLookup = CellSampler.this.deferredLookup.get();
			Cell cell = (this.chunk != null && this.chunkX == chunkX && this.chunkZ == chunkZ) ? 
				this.chunk.getCell(blockX, blockZ) :
				this.cache2d.getAndUpdate(worldLookup, blockX, blockZ, false);
			return this.structureRiverFix(cell, CellSampler.this.field.read(cell, worldLookup.getHeightmap()));
		}

		@Override
		public double minValue() {
			return CellSampler.this.minValue();
		}

		@Override
		public double maxValue() {
			return CellSampler.this.maxValue();
		}
		
		private float structureRiverFix(Cell cell, float value) {
			if(CellSampler.this.field == Field.HEIGHT) {
				if(cell.riverMask < 0.1F) {
					return value;
				}
			}
			return value;
		}
	}
	
	public record Marker(Field field) implements MarkerFunction {
		public static final MapCodec<Marker> CODEC = RecordCodecBuilder.mapCodec(instance -> instance.group(
			Field.CODEC.fieldOf("field").forGetter(Marker::field)
		).apply(instance, Marker::new));
		
		@Override
		public KeyDispatchDataCodec<Marker> codec() {
			return KeyDispatchDataCodec.of(CODEC);
		}

		@Override
		public DensityFunction mapAll(Visitor visitor) {
			return visitor.apply(this);
		}
	}
	
	public enum Field implements StringRepresentable {
		HEIGHT("height") {
			
			@Override
			public float read(Cell cell, Heightmap heightmap) {
				return cell.height;
			}
		},
		CONTINENT("continent") {
			
			@Override
			public float read(Cell cell, Heightmap heightmap) {
				Levels levels = heightmap.levels();
				ControlPoints controlPoints = heightmap.controlPoints();
				
				float deepOcean = controlPoints.deepOcean();
				float shallowOcean = controlPoints.shallowOcean();
				float beach = controlPoints.beach();
				float coast = controlPoints.coast();
				float inland = controlPoints.inland();
				
				if(cell.terrain.isDeepOcean()) {
					float alpha = NoiseUtil.clamp(cell.continentEdge, 0.0F, deepOcean);
					alpha = NoiseUtil.lerp(alpha, 0.0F, deepOcean, 0.0F, 1.0F);
					return NoiseUtil.lerp(Continentalness.DEEP_OCEAN.min() + 0.05F, Continentalness.DEEP_OCEAN.max(), alpha);					
				}
				
				if(cell.terrain.isShallowOcean()) {
					float alpha = NoiseUtil.clamp(cell.continentEdge, deepOcean, shallowOcean);
					alpha = NoiseUtil.lerp(alpha, deepOcean, shallowOcean, 0.0F, 0.98F);
					return NoiseUtil.lerp(Continentalness.OCEAN.min(), Continentalness.OCEAN.max(), alpha);
				}
				
				if(cell.terrain.getDelegate() == TerrainCategory.BEACH && cell.height + cell.beachNoise < levels.water(5)) {
					float alpha = NoiseUtil.clamp(cell.continentEdge, shallowOcean, beach);
					alpha = NoiseUtil.lerp(alpha, shallowOcean, beach, 0.0F, 1.0F);
					return NoiseUtil.lerp(Continentalness.COAST.min(), Continentalness.COAST.max(), alpha);
				}
			
				float alpha = NoiseUtil.clamp(cell.continentEdge, beach, inland);
				alpha = NoiseUtil.lerp(alpha, beach, inland, 0.0F, 1.0F);
				return NoiseUtil.lerp(Continentalness.NEAR_INLAND.mid(), Continentalness.FAR_INLAND.max(), alpha);
			}
		},
		EROSION("erosion") {
			
			@Override
			public float read(Cell cell, Heightmap heightmap) {
				return cell.erosion;
			}
		},
		WEIRDNESS("weirdness") {
			
			@Override
			public float read(Cell cell, Heightmap heightmap) {
				return cell.weirdness;
			}
		},
		BIOME_REGION("biome_region") {
			
			@Override
			public float read(Cell cell, Heightmap heightmap) {
				return cell.biomeRegionId;
			}
		},
		TEMPERATURE("temperature") {
			
			@Override
			public float read(Cell cell, Heightmap heightmap) {
				return cell.temperature;
			}
		},
		MOISTURE("moisture") {
			
			@Override
			public float read(Cell cell, Heightmap heightmap) {
				return cell.moisture;
			}
		},
		GRADIENT("gradient") {
			
			@Override
			public float read(Cell cell, Heightmap heightmap) {
				return cell.gradient;
			}
		},
		HEIGHT_EROSION("height_erosion") {
			
			@Override
			public float read(Cell cell, Heightmap heightmap) {
				return cell.heightErosion;
			}
		},
		SEDIMENT("sediment") {
			
			@Override
			public float read(Cell cell, Heightmap heightmap) {
				return cell.sediment;
			}
		};

		public static final Codec<Field> CODEC = StringRepresentable.fromEnum(Field::values);
		
		private String name;
		
		private Field(String name) {
			this.name = name;
		}
		
		@Override
		public String getSerializedName() {
			return this.name;
		}
		
		public abstract float read(Cell cell, Heightmap heightmap);
	}
}
