//! Bounded owned procedural column sampler. Only complete successful tiles publish.
use crate::{
    cell::Cell,
    graph::{GraphError, Noise},
    heightmap::Heightmap,
    preset::KernelPreset,
    tile::TileBuffer,
    tile_work_cache::TileWorkCache,
    world_filters::WorldFilters,
};
use std::sync::Arc;
type R<T> = Result<T, GraphError>;
#[derive(Clone, Copy, Debug)]
pub struct SamplerOptions {
    pub tile_factor: i32,
    pub border_chunks: i32,
    pub cache_capacity: usize,
    pub max_in_flight: usize,
}
impl SamplerOptions {
    pub fn for_preset(preset: &KernelPreset) -> Self {
        Self {
            tile_factor: 3,
            border_chunks: (preset.filters.erosion.droplet_lifetime / 16).clamp(1, 2),
            cache_capacity: 8,
            max_in_flight: 4,
        }
    }
}
#[derive(Debug)]
pub struct KernelSampler {
    heightmap: Heightmap,
    filters: WorldFilters,
    options: SamplerOptions,
    cache: TileWorkCache,
}
impl KernelSampler {
    pub fn new(
        preset: &KernelPreset,
        seed: i32,
        ground: Noise,
        edge: Noise,
        options: SamplerOptions,
    ) -> R<Self> {
        if !(0..=4).contains(&options.tile_factor)
            || !(0..=4).contains(&options.border_chunks)
            || !(1..=32).contains(&options.cache_capacity)
            || !(1..=8).contains(&options.max_in_flight)
        {
            return Err(GraphError::InvalidSourceParameters(
                "bounded native sampler factor/border/capacity/concurrency",
            ));
        }
        let heightmap = Heightmap::new(preset, seed, ground, edge)?;
        let filters = WorldFilters::new(preset, seed, heightmap.levels)?;
        Ok(Self {
            heightmap,
            filters,
            options,
            cache: TileWorkCache::new(options.cache_capacity, options.max_in_flight),
        })
    }
    pub fn heightmap(&self) -> &Heightmap {
        &self.heightmap
    }
    pub fn options(&self) -> SamplerOptions {
        self.options
    }
    pub fn sample(&self, x: i32, z: i32) -> R<Cell> {
        let shift = 4 + self.options.tile_factor;
        let tile = self.tile(x >> shift, z >> shift)?;
        Ok(*tile.lookup(x, z))
    }
    pub fn tile(&self, x: i32, z: i32) -> R<Arc<TileBuffer>> {
        self.cache.get((x, z), || {
            let mut tile =
                TileBuffer::new(x, z, self.options.tile_factor, self.options.border_chunks)?;
            tile.populate(&self.heightmap)?;
            self.filters
                .apply(&mut tile, true, &self.heightmap.terrains)?;
            Ok(Arc::new(tile))
        })
    }
    /// Older in-flight work may finish for its callers but cannot repopulate this
    /// cleared tile-cache generation. Retained caller Arc values remain valid.
    pub fn clear_tiles(&self) -> R<()> {
        self.cache.clear()
    }
    pub fn cache_counts(&self) -> R<(usize, usize)> {
        self.cache.counts()
    }
    pub fn release_caches(&self) -> R<()> {
        self.clear_tiles()?;
        self.heightmap.clear_river_cache()?;
        self.filters.clear_brush_cache()
    }
}
