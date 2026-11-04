//! MIT ReTerraForged River geometry. Positions use owned float pairs in place of
//! packing/unpacking them through a long; finite coordinate bits are unchanged.
use crate::{
    graph::{java_max, java_min},
    line, noise_util as u,
    river_config::Variance,
    river_warp::RiverWarp,
};
pub const VALLEY_WIDTH: i32 = 275;
pub const MAIN_VALLEY: Variance = Variance {
    min: 0.8,
    range: 0.7,
};
pub const FORK_VALLEY: Variance = Variance {
    min: 0.4,
    range: 0.75,
};
pub const FORK_ANGLE: Variance = Variance {
    min: 0.075,
    range: 0.115,
};
pub const MAIN_SPACING: Variance = Variance {
    min: 0.1,
    range: 0.25,
};
pub const FORK_SPACING: Variance = Variance {
    min: 0.25,
    range: 0.25,
};
#[derive(Clone, Debug)]
pub struct River {
    pub x1: f32,
    pub z1: f32,
    pub x2: f32,
    pub z2: f32,
    pub dx: f32,
    pub dz: f32,
    pub ndx: f32,
    pub ndz: f32,
    pub norm_x: f32,
    pub norm_z: f32,
    pub length: f32,
    pub length2: f32,
    pub min_x: f32,
    pub min_z: f32,
    pub max_x: f32,
    pub max_z: f32,
}
impl River {
    pub fn new(x1: f32, z1: f32, x2: f32, z2: f32) -> Self {
        Self::with_radius(x1, z1, x2, z2, 275.0)
    }
    pub fn with_radius(x1: f32, z1: f32, x2: f32, z2: f32, mut radius: f32) -> Self {
        radius *= 2.0;
        let dx = x2 - x1;
        let dz = z2 - z1;
        let length = u::sqrt(dx * dx + dz * dz);
        let length2 = length * length;
        let ndx = dx / length;
        let ndz = dz / length;
        Self {
            x1,
            z1,
            x2,
            z2,
            dx,
            dz,
            ndx,
            ndz,
            norm_x: ndz,
            norm_z: -ndx,
            length,
            length2,
            min_x: java_min(x1, x2) - radius,
            min_z: java_min(z1, z2) - radius,
            max_x: java_max(x1, x2) + radius,
            max_z: java_max(z1, z2) + radius,
        }
    }
    pub fn from_nodes(a: (i32, i32), b: (i32, i32)) -> Self {
        Self::with_radius(a.0 as f32, a.1 as f32, b.0 as f32, b.1 as f32, 300.0)
    }
    pub fn angle(&self) -> f32 {
        (self.dx as f64).atan2(self.dz as f64) as f32
    }
    pub fn pos(&self, t: f32) -> (f32, f32) {
        (self.x1 + self.dx * t, self.z1 + self.dz * t)
    }
    pub fn pos_warped(&self, t: f32, warp: &RiverWarp) -> (f32, f32) {
        let (mut x, mut z) = self.pos(t);
        if warp.test(t) {
            let (dx, dz) = warp.offset(x, z, t, self);
            x -= dx;
            z -= dz;
        }
        (x, z)
    }
    pub fn intersects(&self, other: &Self) -> bool {
        line::intersect(
            other.x1, other.z1, other.x2, other.z2, self.x1, self.z1, self.x2, self.z2,
        )
    }
    pub fn intersects_extended(&self, other: &Self, extend: f32) -> bool {
        let a = u::clamp_f32(extend / self.length, 0.0, 1.0);
        let b = u::clamp_f32(extend / other.length, 0.0, 1.0);
        let ax = self.x2 - self.x1;
        let az = self.z2 - self.z1;
        let bx = other.x2 - other.x1;
        let bz = other.z2 - other.z1;
        line::intersect(
            self.x1 - ax * a,
            self.z1 - az * a,
            self.x2 + ax * a,
            self.z2 + az * a,
            other.x1 - bx * b,
            other.z1 - bz * b,
            other.x2 + bx * b,
            other.z2 + bz * b,
        )
    }
    pub fn contains(&self, x: f32, z: f32) -> bool {
        x >= self.min_x && x <= self.max_x && z >= self.min_z && z <= self.max_z
    }
    pub fn overlaps(&self, other: &Self) -> bool {
        self.overlaps_box(other.min_x, other.min_z, other.max_x, other.max_z)
    }
    pub fn overlaps_box(&self, min_x: f32, min_z: f32, max_x: f32, max_z: f32) -> bool {
        self.min_x < max_x && self.max_x > min_x && self.min_z < max_z && self.max_z > min_z
    }
    pub fn overlaps_circle_bounds(&self, center: (f32, f32), radius: f32) -> bool {
        self.overlaps_box(
            center.0 - radius,
            center.1 - radius,
            center.0 + radius,
            center.1 + radius,
        )
    }
    pub fn shorten(&self, distance: i32) -> Self {
        let factor = distance as f32 / self.length;
        let dx = self.x2 - self.x1;
        let dz = self.z2 - self.z1;
        let x = u::round_legacy(self.x1 + dx * factor) as f32;
        let z = u::round_legacy(self.z1 + dz * factor) as f32;
        Self::new(x, z, self.x2, self.z2)
    }
}
