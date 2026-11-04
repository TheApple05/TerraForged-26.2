//! MIT ReTerraForged Wetland, preserving sequential pooling and mound updates.
use crate::{
    bounds::BoundsBuilder,
    cell::Cell,
    graph::{GraphError, Noise as N, java_max, java_min},
    layout::Levels,
    line::{dist_sq, distance_on_line},
    noise_util as u, noises as n,
};
#[derive(Clone, Debug)]
pub struct Wetland {
    a: (f32, f32),
    b: (f32, f32),
    radius: f32,
    radius2: f32,
    bed: f32,
    banks: f32,
    mound_min: f32,
    mound_max: f32,
    mound_variance: f32,
    mound_shape: N,
    mound_height: N,
    terrain_edge: N,
}
impl Wetland {
    pub fn new(
        mut seed: i32,
        a: (f32, f32),
        b: (f32, f32),
        radius: f32,
        levels: Levels,
    ) -> Result<Self, GraphError> {
        let bed = levels.water(-1) - 0.5 / levels.world_height as f32;
        let banks = levels.ground(3);
        let mound_min = levels.water(1);
        let mound_max = levels.water(2);
        let mound_variance = mound_max - mound_min;
        seed = seed.wrapping_add(1);
        let shape = N::clamp(
            n::perlin(seed, 10, 1)?,
            N::constant(0.3)?,
            N::constant(0.6)?,
        )?;
        let mound_shape = n::map_range(shape, 0.0, 1.0)?;
        seed = seed.wrapping_add(1);
        let height = N::clamp(
            n::simplex(seed, 20, 1)?,
            N::constant(0.0)?,
            N::constant(0.3)?,
        )?;
        let mound_height = n::map_range(height, 0.0, 1.0)?;
        seed = seed.wrapping_add(1);
        let edge = N::clamp(n::perlin(seed, 8, 1)?, N::constant(0.2)?, N::constant(0.8)?)?;
        let terrain_edge = n::map_range(edge, 0.0, 0.9)?;
        Ok(Self {
            a,
            b,
            radius,
            radius2: radius * radius,
            bed,
            banks,
            mound_min,
            mound_max,
            mound_variance,
            mound_shape,
            mound_height,
            terrain_edge,
        })
    }
    pub fn apply(&self, cell: &mut Cell, rx: f32, rz: f32, x: f32, z: f32) {
        if cell.height < self.bed {
            return;
        }
        let t = distance_on_line(rx, rz, self.a.0, self.a.1, self.b.0, self.b.1);
        let d2 = if t <= 0.0 {
            dist_sq(rx, rz, self.a.0, self.a.1)
        } else if t >= 1.0 {
            dist_sq(rx, rz, self.b.0, self.b.1)
        } else {
            dist_sq(
                rx,
                rz,
                self.a.0 + t * (self.b.0 - self.a.0),
                self.a.1 + t * (self.b.1 - self.a.1),
            )
        };
        if d2 > self.radius2 {
            return;
        }
        let dist = 1.0 - d2 / self.radius2;
        if dist <= 0.0 {
            return;
        }
        let valley_alpha = u::map_clamped_range(dist, 0.0, 0.65, 0.65);
        if cell.height > self.banks {
            cell.height = u::lerp_f32(cell.height, self.banks, valley_alpha);
        }
        let pools_alpha = u::map_clamped_range(dist, 0.65, 0.7, 0.050000012);
        if cell.height > self.bed && cell.height <= self.banks {
            cell.height = u::lerp_f32(cell.height, self.bed, pools_alpha);
        }
        if pools_alpha >= 1.0 {
            cell.erosion_mask = true;
        }
        if dist > 0.65 && pools_alpha > self.terrain_edge.compute(x, z, 0) {
            cell.terrain = 7;
        }
        if cell.height >= self.bed && cell.height < self.mound_max {
            let shape_alpha = self.mound_shape.compute(x, z, 0) * pools_alpha;
            let mounds = self.mound_min + self.mound_height.compute(x, z, 0) * self.mound_variance;
            cell.height = u::lerp_f32(cell.height, mounds, shape_alpha);
        }
        cell.river_mask = java_min(cell.river_mask, 1.0 - valley_alpha);
    }
    pub fn record_bounds(&self, b: &mut BoundsBuilder) {
        b.record(
            java_min(self.a.0, self.b.0) - self.radius,
            java_min(self.a.1, self.b.1) - self.radius,
        );
        b.record(
            java_max(self.a.0, self.b.0) + self.radius,
            java_max(self.a.1, self.b.1) + self.radius,
        );
    }
}
