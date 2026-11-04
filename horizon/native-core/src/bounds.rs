//! MIT ReTerraForged Boundsf, including the positive Float.MIN_VALUE maxima.
use crate::graph::{java_max, java_min};
#[derive(Clone, Copy, Debug)]
pub struct Bounds {
    pub min_x: f32,
    pub min_z: f32,
    pub max_x: f32,
    pub max_z: f32,
}
impl Bounds {
    pub const NONE: Self = Self {
        min_x: 1.0,
        min_z: 1.0,
        max_x: -1.0,
        max_z: -1.0,
    };
    pub fn contains(&self, x: f32, z: f32) -> bool {
        x >= self.min_x && x <= self.max_x && z >= self.min_z && z <= self.max_z
    }
}
#[derive(Clone, Copy, Debug)]
pub struct BoundsBuilder {
    value: Bounds,
}
impl Default for BoundsBuilder {
    fn default() -> Self {
        Self::new()
    }
}
impl BoundsBuilder {
    pub fn new() -> Self {
        Self {
            value: Bounds {
                min_x: f32::MAX,
                min_z: f32::MAX,
                max_x: f32::from_bits(1),
                max_z: f32::from_bits(1),
            },
        }
    }
    pub fn record(&mut self, x: f32, z: f32) {
        self.value.min_x = java_min(self.value.min_x, x);
        self.value.min_z = java_min(self.value.min_z, z);
        self.value.max_x = java_max(self.value.max_x, x);
        self.value.max_z = java_max(self.value.max_z, z);
    }
    pub fn build(&self) -> Bounds {
        self.value
    }
}
