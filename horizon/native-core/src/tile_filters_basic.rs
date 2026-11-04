//! MIT ReTerraForged modifiers, in-place smoothing and steepness.
use crate::{
    cell::Cell,
    graph::{GraphError, java_max, java_min},
    layout::Levels,
    noise_util as u,
    preset::SmoothingSettings,
    terrain::TerrainRegistry,
    tile::TileBuffer,
};

#[derive(Clone, Copy, Debug)]
pub struct Modifier {
    min: f32,
    max: f32,
    range: f32,
    inversions: usize,
}

impl Modifier {
    pub fn range(min: f32, max: f32) -> Self {
        Self {
            min,
            max,
            range: max - min,
            inversions: 0,
        }
    }

    pub fn invert(self) -> Self {
        Self {
            min: self.min,
            max: self.max,
            range: self.range,
            inversions: self.inversions + 1,
        }
    }

    pub fn value(&self, value: f32) -> f32 {
        let mut raw = if value > self.max {
            1.0
        } else if value < self.min {
            0.0
        } else {
            (value - self.min) / self.range
        };
        for _ in 0..self.inversions {
            raw = 1.0 - raw;
        }
        raw
    }

    pub fn modify(
        &self,
        cell: &Cell,
        value: f32,
        terrains: &TerrainRegistry,
    ) -> Result<f32, GraphError> {
        let terrain = terrains
            .get(cell.terrain)
            .ok_or(GraphError::InvalidSourceParameters(
                "owned filter terrain ID",
            ))?;
        let mut strength_modifier = 1.0;
        let erosion_modifier = terrain.erosion_modifier();
        if erosion_modifier != 1.0 {
            let alpha = u::map_clamped_range(cell.terrain_region_edge, 0.0, 0.15, 0.15);
            strength_modifier = u::lerp_f32(1.0, erosion_modifier, alpha);
        }
        if cell.river_mask < 0.1 {
            strength_modifier *= u::map_clamped_range(cell.river_mask, 0.002, 0.1, 0.098);
        }
        Ok(self.value(cell.height) * strength_modifier * value)
    }
}

#[derive(Clone, Debug)]
pub struct Smoothing {
    smoothing_radius: f32,
    smoothing_rate: f32,
    modifier: Modifier,
}

impl Smoothing {
    pub fn new(settings: SmoothingSettings, levels: Levels) -> Self {
        Self {
            smoothing_radius: settings.smoothing_radius,
            smoothing_rate: settings.smoothing_rate,
            modifier: Modifier::range(levels.ground(1), levels.ground(120)).invert(),
        }
    }

    pub fn apply(
        &self,
        tile: &mut TileBuffer,
        iterations: i32,
        terrains: &TerrainRegistry,
    ) -> Result<(), GraphError> {
        let mut remaining = iterations;
        while remaining > 0 {
            remaining -= 1;
            self.apply_once(tile, terrains)?;
        }
        Ok(())
    }

    fn apply_once(
        &self,
        tile: &mut TileBuffer,
        terrains: &TerrainRegistry,
    ) -> Result<(), GraphError> {
        let radius = u::round_legacy(self.smoothing_radius + 0.5);
        let radius_sq = self.smoothing_radius * self.smoothing_radius;

        let total = tile.size.total;
        let max_z = total - radius;
        let max_x = total - radius;

        let mut z = radius;
        while z < max_z {
            let mut x = radius;
            while x < max_x {
                let idx = match tile.raw_index(x, z) {
                    Some(i) => i,
                    None => {
                        x += 1;
                        continue;
                    }
                };
                let cell = tile.cells[idx];
                if !cell.erosion_mask {
                    let mut total_weighted = 0.0;
                    let mut weights = 0.0;
                    let mut dz = -radius;
                    while dz <= radius {
                        let mut dx = -radius;
                        while dx <= radius {
                            let dist2 = (dx * dx + dz * dz) as f32;
                            if dist2 <= radius_sq {
                                let px = x + dx;
                                let pz = z + dz;
                                if let Some(neighbour) = tile.get_raw(px, pz) {
                                    let value = neighbour.height;
                                    let weight = 1.0 - dist2 / radius_sq;
                                    total_weighted += value * weight;
                                    weights += weight;
                                }
                            }
                            dx += 1;
                        }
                        dz += 1;
                    }
                    if weights > 0.0 {
                        let dif = cell.height - total_weighted / weights;
                        let amount =
                            self.modifier
                                .modify(&cell, dif * self.smoothing_rate, terrains)?;
                        let cell_mut = &mut tile.cells[idx];
                        cell_mut.height -= amount;
                    }
                }
                x += 1;
            }
            z += 1;
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct Steepness {
    radius: i32,
    scaler: f32,
    water_level: f32,
}

impl Steepness {
    pub fn new(radius: i32, scaler: f32, levels: Levels) -> Self {
        Self {
            radius,
            scaler,
            water_level: levels.water,
        }
    }

    pub fn apply(&self, tile: &mut TileBuffer) {
        let total = tile.size.total;
        let mut z = 0;
        while z < total {
            let mut x = 0;
            while x < total {
                if let Some(idx) = tile.raw_index(x, z) {
                    let cell = tile.cells[idx];
                    let mut total_height_dif = 0.0;
                    let mut dz = -1;
                    while dz <= 2 {
                        let mut dx = -1;
                        while dx <= 2 {
                            if dx != 0 || dz != 0 {
                                let nx = x + dx * self.radius;
                                let nz = z + dz * self.radius;
                                if let Some(neighbour) = tile.get_raw(nx, nz) {
                                    let height = java_max(neighbour.height, self.water_level);
                                    total_height_dif +=
                                        (cell.height - height).abs() / self.radius as f32;
                                }
                            }
                            dx += 1;
                        }
                        dz += 1;
                    }
                    tile.cells[idx].gradient = java_min(1.0, total_height_dif * self.scaler);
                }
                x += 1;
            }
            z += 1;
        }
    }
}
