//! MIT ReTerraForged WorldFilters ordering and required coast/quart corrections.
use crate::{
    cell::Cell,
    graph::GraphError,
    layout::Levels,
    preset::{ControlPoints, FilterSettings, KernelPreset},
    terrain::{Category, TerrainRegistry},
    tile::TileBuffer,
    tile_erosion::Erosion,
    tile_filters_basic::{Modifier, Smoothing, Steepness},
};
use std::sync::{Arc, Mutex};
type R<T> = Result<T, GraphError>;
#[derive(Debug)]
pub struct WorldFilters {
    settings: FilterSettings,
    levels: Levels,
    transition: ControlPoints,
    seed: i32,
    smoothing: Smoothing,
    steepness: Steepness,
    erosion: Mutex<Option<(i32, Arc<Erosion>)>>,
}
impl WorldFilters {
    pub fn clear_brush_cache(&self) -> R<()> {
        *self
            .erosion
            .lock()
            .map_err(|_| GraphError::InvalidSourceParameters("erosion brush cache poisoned"))? =
            None;
        Ok(())
    }
    pub fn new(preset: &KernelPreset, seed: i32, levels: Levels) -> R<Self> {
        let s = &preset.filters;
        if !(0..=4096).contains(&s.erosion.droplets_per_chunk)
            || !(0..=1000).contains(&s.erosion.droplet_lifetime)
            || !(0..=64).contains(&s.smoothing.iterations)
            || !(0.0..=64.0).contains(&s.smoothing.smoothing_radius)
            || ![
                s.erosion.droplet_volume,
                s.erosion.droplet_velocity,
                s.erosion.erosion_rate,
                s.erosion.deposite_rate,
                s.smoothing.smoothing_rate,
            ]
            .iter()
            .all(|v| v.is_finite())
        {
            return Err(GraphError::InvalidSourceParameters(
                "bounded finite native filter settings",
            ));
        }
        Ok(Self {
            settings: s.clone(),
            levels,
            transition: preset.world.control_points.make(),
            seed: seed.wrapping_add(12768),
            smoothing: Smoothing::new(s.smoothing, levels),
            steepness: Steepness::new(1, 10.0, levels),
            erosion: Mutex::new(None),
        })
    }
    pub fn apply(
        &self,
        tile: &mut TileBuffer,
        optional: bool,
        terrains: &TerrainRegistry,
    ) -> R<()> {
        if optional {
            let erosion = {
                let mut cached = self.erosion.lock().map_err(|_| {
                    GraphError::InvalidSourceParameters("erosion brush cache poisoned")
                })?;
                if !cached
                    .as_ref()
                    .is_some_and(|(size, _)| *size == tile.size.total)
                {
                    *cached = Some((
                        tile.size.total,
                        Arc::new(Erosion::new(
                            self.seed,
                            tile.size.total,
                            self.settings.erosion,
                            Modifier::range(self.levels.ground, self.levels.ground(15)),
                        )?),
                    ));
                }
                cached
                    .as_ref()
                    .expect("erosion cache initialized")
                    .1
                    .clone()
            };
            erosion.apply(tile, self.settings.erosion.droplets_per_chunk, terrains)?;
            self.smoothing
                .apply(tile, self.settings.smoothing.iterations, terrains)?;
        }
        self.steepness.apply(tile);
        self.beach(tile, terrains)?;
        if optional {
            self.corrections(tile, terrains)?;
        }
        Ok(())
    }
    pub fn beach(&self, tile: &mut TileBuffer, terrains: &TerrainRegistry) -> R<()> {
        for x in 0..tile.size.total {
            for z in 0..tile.size.total {
                let index = tile.size.index_of(x, z) as usize;
                let cell = tile.cells[index];
                let terrain =
                    terrains
                        .get(cell.terrain)
                        .ok_or(GraphError::InvalidSourceParameters(
                            "owned beach terrain ID",
                        ))?;
                if terrain.is_coast()
                    && !terrain.is_wetland()
                    && cell.continent_edge < self.transition.beach
                {
                    let gx = grad(tile.get_raw(x + 8, z), tile.get_raw(x - 8, z), &cell);
                    let gz = grad(tile.get_raw(x, z - 8), tile.get_raw(x, z + 8), &cell);
                    if gx * gx + gz * gz < 0.275 {
                        tile.cells[index].terrain = 4;
                    }
                }
            }
        }
        Ok(())
    }
    pub fn corrections(&self, tile: &mut TileBuffer, terrains: &TerrainRegistry) -> R<()> {
        for qx in 0..(tile.size.total >> 2) {
            for qz in 0..(tile.size.total >> 2) {
                let mut beach = false;
                'search: for x in (qx << 2)..((qx + 1) << 2) {
                    for z in (qz << 2)..((qz + 1) << 2) {
                        let cell = &tile.cells[tile.size.index_of(x, z) as usize];
                        let terrain = terrains.get(cell.terrain).ok_or(
                            GraphError::InvalidSourceParameters("owned correction terrain ID"),
                        )?;
                        if terrain.category() == Category::Beach
                            || ((terrain.is_shallow_ocean() || terrain.is_deep_ocean())
                                && cell.height > self.levels.water)
                        {
                            beach = true;
                            break 'search;
                        }
                    }
                }
                if beach {
                    for x in (qx << 2)..((qx + 1) << 2) {
                        for z in (qz << 2)..((qz + 1) << 2) {
                            let index = tile.size.index_of(x, z) as usize;
                            tile.cells[index].terrain = 4;
                        }
                    }
                }
            }
        }
        Ok(())
    }
}
fn grad(a: Option<&Cell>, b: Option<&Cell>, default: &Cell) -> f32 {
    let mut distance = 17;
    if a.is_none() {
        distance -= 8;
    }
    if b.is_none() {
        distance -= 8;
    }
    (a.unwrap_or(default).height - b.unwrap_or(default).height) / distance as f32
}
