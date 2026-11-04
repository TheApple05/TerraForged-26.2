// MIT ReTerraForged White.java/Cubic.java, baseline29239f1f164d19dbfcccaca4a8277d64e784207c.
use crate::noise_util as n;

#[derive(Clone, Copy, Debug)]
pub struct White {
    frequency: f32,
}
impl White {
    pub fn new(frequency: f32) -> Self {
        Self { frequency }
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
        Self::sample(x, z, seed).abs()
    }
    pub fn sample(x: f32, z: f32, seed: i32) -> f32 {
        n::val_coord_2d(seed, n::round_legacy(x), n::round_legacy(z))
    }
    pub fn sample_offset(x: f32, z: f32, seed: i32, offset: i32) -> f32 {
        Self::sample(x, z, n::hash(seed, offset))
    }
}

#[derive(Clone, Debug)]
pub struct Cubic {
    frequency: f32,
    octaves: i32,
    lacunarity: f32,
    gain: f32,
    min: f32,
    max: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SourceError {
    InvalidOctaves,
}

impl Cubic {
    pub fn new(
        frequency: f32,
        octaves: i32,
        lacunarity: f32,
        gain: f32,
    ) -> Result<Self, SourceError> {
        if !(0..=32).contains(&octaves) {
            return Err(SourceError::InvalidOctaves);
        }
        Ok(Self {
            frequency,
            octaves,
            lacunarity,
            gain,
            min: Self::bound(-0.75, octaves, gain),
            max: Self::bound(0.75, octaves, gain),
        })
    }
    // The source record reports these pre-normalization endpoints even though
    // compute normalizes. Do not replace them with 0/1 metadata.
    pub fn min_value(&self) -> f32 {
        self.min
    }
    pub fn max_value(&self) -> f32 {
        self.max
    }
    pub fn compute(&self, mut x: f32, mut z: f32, seed: i32) -> f32 {
        x *= self.frequency;
        z *= self.frequency;
        let mut sum = Self::sample(x, z, seed);
        let mut amplifier = 1.0f32;
        for i in 1..self.octaves {
            x *= self.lacunarity;
            z *= self.lacunarity;
            amplifier *= self.gain;
            sum += Self::sample(x, z, seed.wrapping_add(i)) * amplifier;
        }
        n::map_clamped_range(sum, self.min, self.max, self.max - self.min)
    }
    pub fn sample(x: f32, y: f32, seed: i32) -> f32 {
        let x2 = n::floor_legacy(x);
        let y2 = n::floor_legacy(y);
        let xs = x - x2 as f32;
        let ys = y - y2 as f32;
        let row = |row_y: i32| {
            n::cubic_lerp(
                n::val_coord_2d(seed, x2.wrapping_sub(1), row_y),
                n::val_coord_2d(seed, x2, row_y),
                n::val_coord_2d(seed, x2.wrapping_add(1), row_y),
                n::val_coord_2d(seed, x2.wrapping_add(2), row_y),
                xs,
            )
        };
        n::cubic_lerp(
            row(y2.wrapping_sub(1)),
            row(y2),
            row(y2.wrapping_add(1)),
            row(y2.wrapping_add(2)),
            ys,
        ) * 0.44444445f32
    }
    fn bound(signal: f32, octaves: i32, gain: f32) -> f32 {
        let mut amp = 1.0f32;
        let mut value = signal;
        for _ in 1..octaves {
            amp *= gain;
            value += signal * amp;
        }
        value
    }
}
