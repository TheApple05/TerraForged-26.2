/* SPDX-License-Identifier: MIT
 * Provenance: baseline29239f1f164d19dbfcccaca4a8277d64e784207c
 * Rust translation of raccoonman.reterraforged.world.worldgen.util.FastRandom
 */

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RandomError {
    NonPositiveBound,
}

#[derive(Debug, Clone, Copy)]
pub struct FastRandom {
    seed: u64,
    gamma: u64,
}

const DEFAULT_GAMMA: i64 = -7046029254386353131;

impl FastRandom {
    pub fn new(seed: i64) -> Self {
        Self::with_gamma(seed, DEFAULT_GAMMA)
    }

    pub fn with_gamma(seed: i64, gamma: i64) -> Self {
        Self {
            seed: seed as u64,
            gamma: gamma as u64,
        }
    }

    pub fn seed(&mut self, seed: i64) -> &mut Self {
        self.seed = seed as u64;
        self
    }

    pub fn seed_with_gamma(&mut self, seed: i64, gamma: i64) -> &mut Self {
        self.seed = seed as u64;
        self.gamma = mix_gamma(gamma as u64);
        self
    }

    pub fn set_gamma(&mut self, gamma: i64) -> &mut Self {
        self.gamma = gamma as u64;
        self
    }

    pub fn next_int(&mut self) -> i32 {
        mix32(self.next_seed())
    }

    pub fn next_int_bound(&mut self, bound: i32) -> Result<i32, RandomError> {
        if bound <= 0 {
            return Err(RandomError::NonPositiveBound);
        }
        let bound_u = bound as u32;
        let mut r = mix32(self.next_seed()) as u32;
        let m: u32 = bound_u.wrapping_sub(1);
        if (bound_u & m) == 0 {
            r &= m;
        } else {
            let mut u: u32 = r >> 1;
            loop {
                r = u % bound_u;
                let sum = u.wrapping_add(m).wrapping_sub(r);
                let signed = sum as i32;
                if signed >= 0 {
                    break;
                }
                u = (mix32(self.next_seed()) as u32) >> 1;
            }
        }
        Ok(r as i32)
    }

    pub fn next_float(&mut self) -> f32 {
        let v = (mix32(self.next_seed()) as u32) >> 8;
        (v as f32) * 5.9604645e-8f32
    }

    pub fn next_boolean(&mut self) -> bool {
        mix32(self.next_seed()) < 0
    }

    fn next_seed(&mut self) -> u64 {
        self.seed = self.seed.wrapping_add(self.gamma);
        self.seed
    }
}

#[inline]
fn mix32(z: u64) -> i32 {
    let z = (z ^ (z >> 33)).wrapping_mul(7109453100751455733u64);
    let out = ((z ^ (z >> 28)).wrapping_mul(-3808689974395783757i64 as u64)) >> 32;
    out as i32
}

#[inline]
fn mix_gamma(z: u64) -> u64 {
    let z = (z ^ (z >> 33)).wrapping_mul(-49064778989728563i64 as u64);
    let z = (z ^ (z >> 33)).wrapping_mul(-4265267296055464877i64 as u64);
    let z = (z ^ (z >> 33)) | 0x1u64;
    let n = (z ^ (z >> 1)).count_ones();
    if n < 24 { z ^ 0xAAAAAAAAAAAAAAAAu64 } else { z }
}
