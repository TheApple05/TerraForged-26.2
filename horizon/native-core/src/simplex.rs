// Mechanical translation of MIT ReTerraForged Simplex.java, baseline
// 29239f1f164d19dbfcccaca4a8277d64e784207c. See LICENSE-RETERRAFORGED.
use crate::noise_util::{floor_legacy, grad_coord_2d_24, map_clamped_range};
use crate::perlin::Interpolation;

const SIGNALS: [f32; 7] = [1.0, 0.989, 0.81, 0.781, 0.708, 0.702, 0.696];

#[derive(Clone, Debug)]
pub struct Simplex {
    frequency: f32,
    octaves: i32,
    lacunarity: f32,
    gain: f32,
    _interpolation: Interpolation,
    min: f32,
    max: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SimplexError {
    InvalidOctaves(i32),
}

impl Simplex {
    pub fn new(
        frequency: f32,
        octaves: i32,
        lacunarity: f32,
        gain: f32,
        interpolation: Interpolation,
    ) -> Result<Self, SimplexError> {
        if !(0..=32).contains(&octaves) {
            return Err(SimplexError::InvalidOctaves(octaves));
        }
        let signal = SIGNALS[octaves.min(6) as usize];
        let mut max = 0.0f32;
        let mut amp = 1.0f32;
        for _ in 0..octaves {
            max += amp * signal;
            amp *= gain;
        }
        Ok(Self {
            frequency,
            octaves,
            lacunarity,
            gain,
            _interpolation: interpolation,
            min: -max,
            max,
        })
    }

    pub fn min_value(&self) -> f32 {
        0.0
    }
    pub fn max_value(&self) -> f32 {
        1.0
    }

    pub fn compute(&self, mut x: f32, mut z: f32, seed: i32) -> f32 {
        x *= self.frequency;
        z *= self.frequency;
        let mut sum = 0.0f32;
        let mut amp = 1.0f32;
        for i in 0..self.octaves {
            sum += Self::sample(x, z, seed.wrapping_add(i)) * amp;
            x *= self.lacunarity;
            z *= self.lacunarity;
            amp *= self.gain;
        }
        map_clamped_range(sum, self.min, self.max, self.max - self.min)
    }

    pub fn sample(x: f32, y: f32, seed: i32) -> f32 {
        Self::single_simplex(x, y, seed, 79.869484f32)
    }

    pub fn single_simplex(x: f32, y: f32, seed: i32, scaler: f32) -> f32 {
        let mut t = (x + y) * 0.36602542f32;
        let i = floor_legacy(x + t);
        let j = floor_legacy(y + t);
        t = i.wrapping_add(j) as f32 * 0.21132487f32;
        let x0 = i as f32 - t;
        let y0 = j as f32 - t;
        let x2 = x - x0;
        let y2 = y - y0;
        let (i2, j2): (i32, i32) = if x2 > y2 { (1, 0) } else { (0, 1) };
        let x3 = x2 - i2 as f32 + 0.21132487f32;
        let y3 = y2 - j2 as f32 + 0.21132487f32;
        let x4 = x2 - 1.0f32 + 0.42264974f32;
        let y4 = y2 - 1.0f32 + 0.42264974f32;
        t = 0.5f32 - x2 * x2 - y2 * y2;
        let n0 = if t < 0.0f32 {
            0.0f32
        } else {
            t *= t;
            t * t * grad_coord_2d_24(seed, i, j, x2, y2)
        };
        t = 0.5f32 - x3 * x3 - y3 * y3;
        let n2 = if t < 0.0f32 {
            0.0f32
        } else {
            t *= t;
            t * t * grad_coord_2d_24(seed, i.wrapping_add(i2), j.wrapping_add(j2), x3, y3)
        };
        t = 0.5f32 - x4 * x4 - y4 * y4;
        let n3 = if t < 0.0f32 {
            0.0f32
        } else {
            t *= t;
            t * t * grad_coord_2d_24(seed, i.wrapping_add(1), j.wrapping_add(1), x4, y4)
        };
        scaler * (n0 + n2 + n3)
    }
}
