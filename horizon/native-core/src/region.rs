// MIT ReTerraForged RegionModule geometry, baseline 29239f1f164d19dbfcccaca4a8277d64e784207c.
use crate::cell::Cell;
use crate::domain::Domain;
use crate::graph::{GraphError, Noise};
use crate::noise_util::{cell, floor_legacy, map_clamped_range, pow_f32, val_coord_2d};
use crate::worley::{DistanceFunction, EdgeFunction};

#[derive(Clone, Debug)]
pub struct RegionConfig {
    pub seed: i32,
    pub scale: i32,
    pub warp_x: Noise,
    pub warp_z: Noise,
    pub warp_strength: f32,
}

#[derive(Clone, Debug)]
pub struct RegionModule {
    seed: i32,
    frequency: f32,
    edge_min: f32,
    edge_max: f32,
    edge_range: f32,
    warp: Domain,
}

impl RegionModule {
    pub fn new(
        seed: i32,
        scale: i32,
        warp_x: Noise,
        warp_z: Noise,
        warp_strength: f32,
    ) -> Result<Self, GraphError> {
        let edge_min = 0.0f32;
        let edge_max = 0.5f32;
        let edge_range = edge_max - edge_min;
        let frequency = 1.0f32 / (scale as f32);
        let warp = Domain::warp(warp_x, warp_z, Noise::constant(warp_strength)?)?;
        Ok(Self {
            seed: seed.wrapping_add(7),
            frequency,
            edge_min,
            edge_max,
            edge_range,
            warp,
        })
    }

    pub fn apply(&self, cell_ref: &mut Cell, x: f32, z: f32) {
        let ox = self.warp.offset_x(x, z, 0);
        let oz = self.warp.offset_z(x, z, 0);
        let mut px = x + ox;
        let mut py = z + oz;
        px *= self.frequency;
        py *= self.frequency;
        let mut cell_x: i32 = 0;
        let mut cell_y: i32 = 0;
        let xi = floor_legacy(px);
        let yi = floor_legacy(py);
        let mut edge_distance = f32::MAX;
        let mut edge_distance2 = f32::MAX;
        let dist = DistanceFunction::Natural;
        let mut dy: i32 = -1;
        while dy <= 1 {
            let mut dx: i32 = -1;
            while dx <= 1 {
                let cx = xi.wrapping_add(dx);
                let cy = yi.wrapping_add(dy);
                let (vec_x_raw, vec_y_raw) = cell(self.seed, cx, cy);
                let vec_x = (cx as f32) + vec_x_raw * 0.7f32;
                let vec_y = (cy as f32) + vec_y_raw * 0.7f32;
                let distance = dist.apply(vec_x - px, vec_y - py);
                if distance < edge_distance {
                    edge_distance2 = edge_distance;
                    edge_distance = distance;
                    cell_x = cx;
                    cell_y = cy;
                } else if distance < edge_distance2 {
                    edge_distance2 = distance;
                }
                dx += 1;
            }
            dy += 1;
        }
        cell_ref.terrain_region_id = self.cell_value(self.seed, cell_x, cell_y);
        cell_ref.terrain_region_edge = self.edge_value(edge_distance, edge_distance2);
    }

    fn cell_value(&self, seed: i32, cell_x: i32, cell_y: i32) -> f32 {
        let value = val_coord_2d(seed, cell_x, cell_y);
        map_clamped_range(value, -1.0f32, 1.0f32, 2.0f32)
    }

    fn edge_value(&self, distance: f32, distance2: f32) -> f32 {
        let edge = EdgeFunction::Distance2Div;
        let value = edge.apply(distance, distance2);
        let mut edge_value =
            1.0f32 - map_clamped_range(value, edge.min(), edge.max(), edge.range());
        edge_value = pow_f32(edge_value, 1.5f32);
        if edge_value < self.edge_min {
            return 0.0f32;
        }
        if edge_value > self.edge_max {
            return 1.0f32;
        }
        (edge_value - self.edge_min) / self.edge_range
    }
}
