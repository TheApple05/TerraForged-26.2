// MIT ReTerraForged Lake carving and bounds.
use crate::bounds::BoundsBuilder;
use crate::cell::Cell;
use crate::graph::java_min;
use crate::noise_util::{lerp_f32, map_clamped_range};
use crate::river_config::LakeConfig;

const LAKE_TERRAIN: usize = 6;

#[derive(Clone, Debug)]
pub struct Lake {
    valley: f32,
    valley2: f32,
    lake_distance2: f32,
    valley_distance2: f32,
    bank_alpha_min: f32,
    bank_alpha_max: f32,
    bank_alpha_range: f32,
    depth: f32,
    bank_min: f32,
    bank_max: f32,
    center: (f32, f32),
}

impl Lake {
    pub fn new(center: (f32, f32), radius: f32, multiplier: f32, config: &LakeConfig) -> Self {
        let lake_radius = radius * multiplier;
        let valley = 275.0 * multiplier;
        let mut lake = Lake {
            valley,
            valley2: valley * valley,
            lake_distance2: 0.0,
            valley_distance2: 0.0,
            bank_alpha_min: config.bank_min,
            bank_alpha_max: 0.0,
            bank_alpha_range: 0.0,
            depth: config.depth,
            bank_min: config.bank_min,
            bank_max: config.bank_max,
            center,
        };
        lake.bank_alpha_max = java_min(1.0, lake.bank_alpha_min + 0.275);
        lake.bank_alpha_range = lake.bank_alpha_max - lake.bank_alpha_min;
        lake.lake_distance2 = lake_radius * lake_radius;
        lake.valley_distance2 = lake.valley2 - lake.lake_distance2;
        lake
    }

    #[allow(clippy::manual_clamp)] // Keep the independently observed Java comparison branches.
    pub fn apply(&self, cell: &mut Cell, x: f32, z: f32) {
        let distance2 = self.get_distance2(x, z);
        if distance2 > self.valley2 {
            return;
        }
        let bank_height = self.get_bank_height(cell);
        if distance2 <= self.lake_distance2 {
            cell.height = java_min(bank_height, cell.height);
            if distance2 < self.lake_distance2 {
                let mut depth_alpha = 1.0 - distance2 / self.lake_distance2;
                if depth_alpha < 0.0 {
                    depth_alpha = 0.0;
                } else if depth_alpha > 1.0 {
                    depth_alpha = 1.0;
                }
                let lake_depth = java_min(cell.height, self.depth);
                cell.height = lerp_f32(cell.height, lake_depth, depth_alpha);
                cell.terrain = LAKE_TERRAIN;
                cell.river_mask = java_min(cell.river_mask, 1.0 - depth_alpha);
            }
            return;
        }
        if cell.height < bank_height {
            return;
        }
        let mut valley_alpha = 1.0 - (distance2 - self.lake_distance2) / self.valley_distance2;
        if valley_alpha < 0.0 {
            valley_alpha = 0.0;
        } else if valley_alpha > 1.0 {
            valley_alpha = 1.0;
        }
        cell.height = lerp_f32(cell.height, bank_height, valley_alpha);
        cell.river_mask *= 1.0 - valley_alpha;
        cell.river_mask = java_min(cell.river_mask, 1.0 - valley_alpha);
    }

    pub fn record_bounds(&self, builder: &mut BoundsBuilder) {
        builder.record(
            self.center.0 - self.valley * 1.2,
            self.center.1 - self.valley * 1.2,
        );
        builder.record(
            self.center.0 + self.valley * 1.2,
            self.center.1 + self.valley * 1.2,
        );
    }

    pub fn overlaps(&self, x: f32, z: f32, radius2: f32) -> bool {
        let dist2 = self.get_distance2(x, z);
        dist2 < self.lake_distance2 + radius2
    }

    fn get_distance2(&self, x: f32, z: f32) -> f32 {
        let dx = self.center.0 - x;
        let dz = self.center.1 - z;
        dx * dx + dz * dz
    }

    fn get_bank_height(&self, cell: &Cell) -> f32 {
        let bank_height_alpha = map_clamped_range(
            cell.height,
            self.bank_alpha_min,
            self.bank_alpha_max,
            self.bank_alpha_range,
        );
        lerp_f32(self.bank_min, self.bank_max, bank_height_alpha)
    }
}
