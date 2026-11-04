// Provenance: mechanically translated from the MIT-licensed ReTerraForged `Perlin` record
// (package raccoonman.reterraforged.world.worldgen.noise.module). Codec/Visitor boilerplate
// intentionally omitted. Operation order and Java i32 wrapping preserved.

use crate::noise_util::{
    floor_legacy, grad_coord_2d, interp_hermite, interp_quintic, lerp_f32, map_clamped_range,
};

const SIGNALS: [f32; 7] = [1.0, 0.9, 0.83, 0.75, 0.64, 0.62, 0.61];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Interpolation {
    Linear,
    Curve3,
    Curve4,
}

impl Interpolation {
    pub(crate) fn apply(self, t: f32) -> f32 {
        match self {
            Interpolation::Linear => t,
            Interpolation::Curve3 => interp_hermite(t),
            Interpolation::Curve4 => interp_quintic(t),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Perlin {
    seed: i32,
    frequency: f32,
    octaves: i32,
    lacunarity: f32,
    gain: f32,
    interpolation: Interpolation,
    min: f32,
    max: f32,
}

impl Perlin {
    pub fn new(
        seed: i32,
        frequency: f32,
        octaves: i32,
        lacunarity: f32,
        gain: f32,
        interpolation: Interpolation,
    ) -> Result<Self, PerlinError> {
        if !(0..=256).contains(&octaves) {
            return Err(PerlinError::InvalidOctaves(octaves));
        }
        let min = Self::min_impl(octaves, gain);
        let max = Self::max_impl(octaves, gain);
        Ok(Perlin {
            seed,
            frequency,
            octaves,
            lacunarity,
            gain,
            interpolation,
            min,
            max,
        })
    }

    pub fn compute(&self, mut x: f32, mut z: f32, _world_seed: i32) -> f32 {
        x *= self.frequency;
        z *= self.frequency;

        let mut sum = 0.0_f32;
        let mut amplitude = self.gain;
        for i in 0..self.octaves {
            let octave_seed = self.seed.wrapping_add(i);
            sum += Self::sample(x, z, octave_seed, self.interpolation) * amplitude;
            x *= self.lacunarity;
            z *= self.lacunarity;
            amplitude *= self.gain;
        }
        let span = self.max - self.min;
        map_clamped_range(sum, self.min, self.max, span)
    }

    pub fn min_value(&self) -> f32 {
        0.0
    }

    pub fn max_value(&self) -> f32 {
        1.0
    }

    pub fn sample(x: f32, y: f32, seed: i32, interpolation: Interpolation) -> f32 {
        let x2 = floor_legacy(x);
        let y2 = floor_legacy(y);
        let x3 = x2.wrapping_add(1);
        let y3 = y2.wrapping_add(1);
        let xs = interpolation.apply(x - x2 as f32);
        let ys = interpolation.apply(y - y2 as f32);
        let xd0 = x - x2 as f32;
        let yd0 = y - y2 as f32;
        let xd2 = xd0 - 1.0;
        let yd2 = yd0 - 1.0;
        let xf0 = lerp_f32(
            grad_coord_2d(seed, x2, y2, xd0, yd0),
            grad_coord_2d(seed, x3, y2, xd2, yd0),
            xs,
        );
        let xf2 = lerp_f32(
            grad_coord_2d(seed, x2, y3, xd0, yd2),
            grad_coord_2d(seed, x3, y3, xd2, yd2),
            xs,
        );
        lerp_f32(xf0, xf2, ys)
    }

    fn min_impl(octaves: i32, gain: f32) -> f32 {
        -Self::max_impl(octaves, gain)
    }

    fn max_impl(octaves: i32, gain: f32) -> f32 {
        let signal = Self::signal(octaves);
        let mut sum = 0.0_f32;
        let mut amp = gain;
        for _ in 0..octaves {
            sum += signal * amp;
            amp *= gain;
        }
        sum
    }

    fn signal(octaves: i32) -> f32 {
        let index = octaves.min((SIGNALS.len() - 1) as i32);
        SIGNALS[index as usize]
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PerlinError {
    InvalidOctaves(i32),
}
