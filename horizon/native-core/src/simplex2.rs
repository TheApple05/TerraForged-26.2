// Mechanical translation of MIT ReTerraForged Simplex2.java, baseline
// 29239f1f164d19dbfcccaca4a8277d64e784207c. See LICENSE-RETERRAFORGED.
use crate::noise_util::map_clamped_range;
use crate::perlin::Interpolation;

const SIGNALS: [f32; 7] = [1.0, 0.989, 0.81, 0.781, 0.708, 0.702, 0.696];

#[derive(Clone, Debug)]
pub struct Simplex2 {
    frequency: f32,
    octaves: i32,
    lacunarity: f32,
    gain: f32,
    _interpolation: Interpolation,
    min: f32,
    max: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Simplex2Error {
    InvalidOctaves(i32),
}

impl Simplex2 {
    pub fn new(
        frequency: f32,
        octaves: i32,
        lacunarity: f32,
        gain: f32,
        interpolation: Interpolation,
    ) -> Result<Self, Simplex2Error> {
        if !(0..=32).contains(&octaves) {
            return Err(Simplex2Error::InvalidOctaves(octaves));
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
        crate::simplex::Simplex::single_simplex(x, y, seed, 99.83685f32)
    }
}
