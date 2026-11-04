// MIT ReTerraForged Line.java geometry, baseline 29239f1f164d19dbfcccaca4a8277d64e784207c.
use crate::{graph::Noise, noise_util};
#[derive(Clone, Debug)]
pub struct Line {
    x1: f32,
    z1: f32,
    x2: f32,
    z2: f32,
    dx: f32,
    dz: f32,
    length_sq: f32,
    feather: f32,
    feather_bias: f32,
}
impl Line {
    pub fn new(x1: f32, z1: f32, x2: f32, z2: f32, feather: f32) -> Self {
        let dx = x2 - x1;
        let dz = z2 - z1;
        Self {
            x1,
            z1,
            x2,
            z2,
            dx,
            dz,
            length_sq: dx * dx + dz * dz,
            feather,
            feather_bias: 1.0 - feather,
        }
    }
    pub fn compute(
        &self,
        radius: &Noise,
        fade_in: &Noise,
        fade_out: &Noise,
        x: f32,
        z: f32,
        seed: i32,
    ) -> f32 {
        let width = self.width(fade_in, fade_out, x, z, seed);
        let t = (x - self.x1) * self.dx + (z - self.z1) * self.dz;
        let s = noise_util::clamp_f32(t / self.length_sq, 0.0, 1.0);
        let distance = dist_sq(x, z, self.x1 + s * self.dx, self.z1 + s * self.dz);
        let radius = radius.compute(x, z, seed) * width;
        if distance > radius {
            return 0.0;
        }
        let value = distance / radius;
        if self.feather == 0.0 {
            return 1.0 - value;
        }
        let feather = self.feather_bias + width * self.feather;
        (1.0 - value) * feather
    }
    fn width(&self, fade_in: &Noise, fade_out: &Noise, x: f32, z: f32, seed: i32) -> f32 {
        let d1 = dist_sq(x, z, self.x1, self.z1);
        if d1 == 0.0 {
            return 0.0;
        }
        let d2 = dist_sq(x, z, self.x2, self.z2);
        if d2 == 0.0 {
            return 0.0;
        }
        let mut fade = 1.0;
        let input = fade_in.compute(x, z, seed);
        let output = fade_out.compute(x, z, seed);
        if input > 0.0 {
            let distance = input * self.length_sq;
            if d1 < distance {
                fade *= d1 / distance;
            }
        }
        if output > 0.0 {
            let distance = output * self.length_sq;
            if d2 < distance {
                fade *= d2 / distance;
            }
        }
        fade
    }
}
pub fn distance_on_line(x: f32, y: f32, ax: f32, ay: f32, bx: f32, by: f32) -> f32 {
    let dx = bx - ax;
    let dy = by - ay;
    let value = (x - ax) * dx + (y - ay) * dy;
    value / (dx * dx + dy * dy)
}
pub fn dist_sq(x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
    let dx = x2 - x1;
    let dy = y2 - y1;
    dx * dx + dy * dy
}
pub fn intersect(
    ax1: f32,
    ay1: f32,
    ax2: f32,
    ay2: f32,
    bx1: f32,
    by1: f32,
    bx2: f32,
    by2: f32,
) -> bool {
    ccw(ax1, ay1, ax2, ay2, bx1, by1) * ccw(ax1, ay1, ax2, ay2, bx2, by2) <= 0
        && ccw(bx1, by1, bx2, by2, ax1, ay1) * ccw(bx1, by1, bx2, by2, ax2, ay2) <= 0
}
fn ccw(x1: f32, y1: f32, mut x2: f32, mut y2: f32, mut px: f32, mut py: f32) -> i32 {
    x2 -= x1;
    y2 -= y1;
    px -= x1;
    py -= y1;
    // Java evaluates each RHS in float before assigning its double variable.
    let mut value = px * y2 - py * x2;
    if value == 0.0 {
        value = px * x2 + py * y2;
        if value > 0.0 {
            px -= x2;
            py -= y2;
            value = px * x2 + py * y2;
            if value < 0.0 {
                value = 0.0;
            }
        }
    }
    if value < 0.0 {
        -1
    } else if value > 0.0 {
        1
    } else {
        0
    }
}
