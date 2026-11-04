// Ported from ReTerraForged MIT-licensed sources.
// Provenance baseline: 29239f1f164d19dbfcccaca4a8277d64e784207c
// Original: raccoonman.reterraforged.world.worldgen.noise.module
// (PerlinRidge, SimplexRidge, Billow)

use crate::noise_util::{clamp_f32, map_clamped_range, pow_f32};
use crate::perlin::{Interpolation, Perlin};
use crate::simplex2::Simplex2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RidgeError {
    InvalidOctaves,
}

// Source clamps the stored/active octave count to 30. PerlinRidge and Billow
// allocate spectral weights and compute the max bound using the *original*
// requested octave count (0..=32), but only iterate up to the active count.
const MAX_ACTIVE_OCTAVES: i32 = 30;
const MAX_REQUESTED_OCTAVES: i32 = 32;
// SimplexRidge's source array has exactly 30 entries.
const SIMPLEX_WEIGHT_COUNT: usize = 30;

// Java: NoiseUtil.pow(frequency, -1.0F) then frequency *= lacunarity, for
// `octaves` iterations.
fn calculate_spectral_weights(octaves: usize, lacunarity: f32) -> Vec<f32> {
    let mut frequency = 1.0f32;
    let mut weights = Vec::with_capacity(octaves);
    for _ in 0..octaves {
        weights.push(pow_f32(frequency, -1.0f32));
        frequency *= lacunarity;
    }
    weights
}

// Java: Spectra's fixed 30-entry weight array ignoring octave count.
fn calculate_spectral_weights_fixed(lacunarity: f32) -> Vec<f32> {
    calculate_spectral_weights(SIMPLEX_WEIGHT_COUNT, lacunarity)
}

// Identical math shared by PerlinRidge and Billow calculateMaxBound.
fn calculate_max_bound(spectral_weights: &[f32], gain: f32) -> f32 {
    let mut amp = 2.0f32;
    let mut value = 0.0f32;
    let mut weight = 1.0f32;
    for &spectral_weight in spectral_weights.iter() {
        let mut noise = 1.0f32;
        noise *= weight;
        weight = noise * amp;
        weight = java_min(1.0f32, java_max(0.0f32, weight));
        value += noise * spectral_weight;
        amp *= gain;
    }
    value
}

#[derive(Debug, Clone)]
pub struct PerlinRidge {
    frequency: f32,
    octaves: i32,
    lacunarity: f32,
    gain: f32,
    interpolation: Interpolation,
    spectral_weights: Vec<f32>,
    min: f32,
    max: f32,
}

impl PerlinRidge {
    pub fn new(
        frequency: f32,
        octaves: i32,
        lacunarity: f32,
        gain: f32,
        interpolation: Interpolation,
    ) -> Result<Self, RidgeError> {
        if !(0..=MAX_REQUESTED_OCTAVES).contains(&octaves) {
            return Err(RidgeError::InvalidOctaves);
        }
        let spectral_weights = calculate_spectral_weights(octaves as usize, lacunarity);
        let max = calculate_max_bound(&spectral_weights, gain);
        Ok(Self {
            frequency,
            octaves: octaves.min(MAX_ACTIVE_OCTAVES),
            lacunarity,
            gain,
            interpolation,
            spectral_weights,
            min: 0.0f32,
            max,
        })
    }

    pub fn compute(&self, x: f32, z: f32, seed: i32) -> f32 {
        let mut x = x * self.frequency;
        let mut z = z * self.frequency;
        let mut amp = 2.0f32;
        let mut value = 0.0f32;
        let mut weight = 1.0f32;
        let mut octave = 0;
        while octave < self.octaves {
            let mut signal = Perlin::sample(x, z, seed.wrapping_add(octave), self.interpolation);
            signal = 1.0f32 - signal.abs();
            signal *= signal;
            signal *= weight;
            weight = signal * amp;
            weight = clamp_f32(weight, 0.0f32, 1.0f32);
            value += signal * self.spectral_weights[octave as usize];
            x *= self.lacunarity;
            z *= self.lacunarity;
            amp *= self.gain;
            octave += 1;
        }
        map_clamped_range(value, self.min, self.max, (self.max - self.min).abs())
    }

    pub fn min_value(&self) -> f32 {
        0.0f32
    }

    pub fn max_value(&self) -> f32 {
        1.0f32
    }
}

#[derive(Debug, Clone)]
pub struct SimplexRidge {
    frequency: f32,
    octaves: i32,
    lacunarity: f32,
    gain: f32,
    _interpolation: Interpolation,
    spectral_weights: Vec<f32>,
    min: f32,
    max: f32,
}

impl SimplexRidge {
    pub fn new(
        frequency: f32,
        octaves: i32,
        lacunarity: f32,
        gain: f32,
        interpolation: Interpolation,
    ) -> Result<Self, RidgeError> {
        if !(0..=MAX_ACTIVE_OCTAVES).contains(&octaves) {
            return Err(RidgeError::InvalidOctaves);
        }
        let spectral_weights = calculate_spectral_weights_fixed(lacunarity);
        let max = Simplex2::normalization_max(octaves, gain);
        Ok(Self {
            frequency,
            octaves,
            lacunarity,
            gain,
            _interpolation: interpolation,
            spectral_weights,
            min: 0.0f32,
            max,
        })
    }

    pub fn compute(&self, x: f32, z: f32, seed: i32) -> f32 {
        let mut x = x * self.frequency;
        let mut z = z * self.frequency;
        let mut value = 0.0f32;
        let mut weight = 1.0f32;
        let offset = 1.0f32;
        let mut amp = 2.0f32;
        let mut octave = 0;
        while octave < self.octaves {
            let mut signal = Simplex2::sample(x, z, seed.wrapping_add(octave));
            signal = signal.abs();
            signal = offset - signal;
            signal *= signal;
            signal *= weight;
            weight = signal * amp;
            weight = clamp_f32(weight, 0.0f32, 1.0f32);
            value += signal * self.spectral_weights[octave as usize];
            x *= self.lacunarity;
            z *= self.lacunarity;
            amp *= self.gain;
            octave += 1;
        }
        map_clamped_range(value, self.min, self.max, (self.max - self.min).abs())
    }

    pub fn min_value(&self) -> f32 {
        0.0f32
    }

    pub fn max_value(&self) -> f32 {
        1.0f32
    }
}

#[derive(Debug, Clone)]
pub struct Billow {
    inner: PerlinRidge,
}

impl Billow {
    pub fn new(
        frequency: f32,
        octaves: i32,
        lacunarity: f32,
        gain: f32,
        interpolation: Interpolation,
    ) -> Result<Self, RidgeError> {
        // Identical constructor/math to PerlinRidge; we only differ at compute.
        Ok(Self {
            inner: PerlinRidge::new(frequency, octaves, lacunarity, gain, interpolation)?,
        })
    }

    pub fn compute(&self, x: f32, z: f32, seed: i32) -> f32 {
        // Billow computes the PerlinRidge-mapped value then inverts it.
        1.0f32 - self.inner.compute(x, z, seed)
    }

    pub fn min_value(&self) -> f32 {
        0.0f32
    }

    pub fn max_value(&self) -> f32 {
        1.0f32
    }
}

fn java_min(a: f32, b: f32) -> f32 {
    if a.is_nan() {
        return a;
    }
    if a == 0.0 && b == 0.0 && b.to_bits() == 0x80000000 {
        return b;
    }
    if a <= b { a } else { b }
}
fn java_max(a: f32, b: f32) -> f32 {
    if a.is_nan() {
        return a;
    }
    if a == 0.0 && b == 0.0 && a.to_bits() == 0x80000000 {
        return b;
    }
    if a >= b { a } else { b }
}
