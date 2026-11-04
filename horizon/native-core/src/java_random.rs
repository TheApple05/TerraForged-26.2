//! java.util.Random's documented 48-bit algorithm, needed by original terrain/rivers.
//! Implemented from the public Java SE API contract, not Minecraft code.
//! https://docs.oracle.com/en/java/javase/25/docs/api/java.base/java/util/Random.html
const MULTIPLIER: u64 = 0x5DEECE66D;
const ADDEND: u64 = 0xB;
const MASK: u64 = (1u64 << 48) - 1;
#[derive(Clone, Debug)]
pub struct JavaRandom {
    state: u64,
}
#[derive(Clone, Copy, Debug)]
pub struct InvalidBound;
impl std::fmt::Display for InvalidBound {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("JavaRandom bound must be positive and shuffle length fit signed32")
    }
}
impl std::error::Error for InvalidBound {}
impl JavaRandom {
    pub fn new(seed: i64) -> Self {
        let mut r = Self { state: 0 };
        r.set_seed(seed);
        r
    }
    pub fn set_seed(&mut self, seed: i64) {
        self.state = ((seed as u64) ^ MULTIPLIER) & MASK;
    }
    fn next(&mut self, bits: u32) -> u32 {
        self.state = self.state.wrapping_mul(MULTIPLIER).wrapping_add(ADDEND) & MASK;
        (self.state >> (48 - bits)) as u32
    }
    pub fn next_i32(&mut self) -> i32 {
        self.next(32) as i32
    }
    pub fn next_bool(&mut self) -> bool {
        self.next(1) != 0
    }
    pub fn next_f32(&mut self) -> f32 {
        self.next(24) as f32 / (1u32 << 24) as f32
    }
    pub fn bounded_i32(&mut self, bound: i32) -> Result<i32, InvalidBound> {
        if bound <= 0 {
            return Err(InvalidBound);
        }
        if (bound & bound.wrapping_neg()) == bound {
            return Ok(((bound as i64) * (self.next(31) as i64) >> 31) as i32);
        }
        loop {
            let bits = self.next(31) as i32;
            let value = bits % bound;
            if bits.wrapping_sub(value).wrapping_add(bound - 1) >= 0 {
                return Ok(value);
            }
        }
    }
    /// Matches Collections.shuffle(List, Random)'s descending swap/draw sequence.
    pub fn shuffle<T>(&mut self, values: &mut [T]) -> Result<(), InvalidBound> {
        if values.len() > i32::MAX as usize {
            return Err(InvalidBound);
        }
        for size in (2..=values.len()).rev() {
            let selected = self.bounded_i32(size as i32)? as usize;
            values.swap(size - 1, selected);
        }
        Ok(())
    }
}
