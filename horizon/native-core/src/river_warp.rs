//! MIT ReTerraForged RiverWarp, including identity-sensitive NONE semantics.
use crate::{java_random::JavaRandom, noise_util as u, river::River, simplex2::Simplex2};
#[derive(Clone, Debug)]
pub struct RiverWarp {
    seed: i32,
    lower: f32,
    upper: f32,
    lower_range: f32,
    upper_range: f32,
    frequency: f32,
    scale: f32,
    none: bool,
}
impl RiverWarp {
    pub fn new(seed: i32, lower: f32, upper: f32, frequency: f32, scale: f32) -> Self {
        Self {
            seed,
            lower,
            upper,
            lower_range: 1.0 / lower,
            upper_range: 1.0 / (1.0 - upper),
            frequency,
            scale,
            none: false,
        }
    }
    pub fn none() -> Self {
        let mut value = Self::new(0, 0.0, 0.0, 0.0, 0.0);
        value.none = true;
        value
    }
    pub fn parameters(&self) -> (i32, [f32; 6]) {
        (
            self.seed,
            [
                self.lower,
                self.upper,
                self.lower_range,
                self.upper_range,
                self.frequency,
                self.scale,
            ],
        )
    }
    pub fn create_child(
        &self,
        lower: f32,
        upper: f32,
        factor: f32,
        random: &mut JavaRandom,
    ) -> Self {
        Self::new(
            random.next_i32(),
            lower,
            upper,
            self.frequency * factor,
            self.scale * factor,
        )
    }
    pub fn test(&self, t: f32) -> bool {
        !self.none && t >= 0.0 && t <= 1.0
    }
    pub fn offset(&self, x: f32, z: f32, t: f32, river: &River) -> (f32, f32) {
        let alpha = self.warp_alpha(t);
        let px = x * self.frequency;
        let pz = z * self.frequency;
        let distance = alpha * self.scale;
        let noise = Simplex2::sample(px, pz, self.seed);
        let mut dx = river.norm_x * noise * distance;
        let mut dz = river.norm_z * noise * distance;
        let wiggle_alpha = u::map_clamped_range(t, 0.0, 0.075, 0.075);
        let factor = river.length * 4.0E-4;
        let wiggle_frequency = 8.0 * factor;
        let wiggle_distance = u::clamp_f32(wiggle_alpha * 25.0 * factor, 2.0, 45.0);
        let rads = noise + t * 6.2831855 * wiggle_frequency;
        dx += u::cos(rads) * river.norm_x * wiggle_distance;
        dz += u::sin(rads) * river.norm_z * wiggle_distance;
        (dx, dz)
    }
    fn warp_alpha(&self, t: f32) -> f32 {
        if t < 0.0 || t > 1.0 {
            return 0.0;
        }
        if t < self.lower {
            return t * self.lower_range;
        }
        if t > self.upper {
            return (1.0 - t) * self.upper_range;
        }
        1.0
    }
    pub fn create_with_fade(fade: f32, random: &mut JavaRandom) -> Self {
        Self::create(fade, 1.0 - fade, random)
    }
    pub fn create(lower: f32, upper: f32, random: &mut JavaRandom) -> Self {
        let scale = 125.0
            + random
                .bounded_i32(50)
                .expect("fixed positive JavaRandom bound") as f32;
        let frequency = 5.0E-4 + random.next_f32() * 5.0E-4;
        Self::new(random.next_i32(), lower, upper, frequency, scale)
    }
}
