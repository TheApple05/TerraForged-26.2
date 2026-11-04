// Numerical translation of MIT ReTerraForged Erosion.java, baseline
// 29239f1f164d19dbfcccaca4a8277d64e784207c. Scratch belongs to each call.
#[derive(Clone, Debug)]
pub struct Erosion {
    seed: i32,
    octaves: i32,
    strength: f32,
    grid_size: f32,
    amplitude: f32,
    lacunarity: f32,
    distance_falloff: f32,
    blend_mode: BlendMode,
}

#[derive(Clone, Copy, Debug)]
pub enum BlendMode {
    Constant,
    InputLinear,
    OutputLinear,
}

impl BlendMode {
    #[inline]
    pub fn blend(self, value: f32, erosion: f32, strength: f32) -> f32 {
        match self {
            BlendMode::Constant => 1.0f32 - strength,
            BlendMode::InputLinear => 1.0f32 - strength * value,
            BlendMode::OutputLinear => 1.0f32 - strength * erosion,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum ErosionError {
    InvalidOctaves,
}

impl Erosion {
    pub fn new(
        seed: i32,
        octaves: i32,
        strength: f32,
        grid_size: f32,
        amplitude: f32,
        lacunarity: f32,
        distance_falloff: f32,
        blend_mode: BlendMode,
    ) -> Result<Self, ErosionError> {
        if !(0..=32).contains(&octaves) {
            return Err(ErosionError::InvalidOctaves);
        }
        Ok(Self {
            seed,
            octaves,
            strength,
            grid_size,
            amplitude,
            lacunarity,
            distance_falloff,
            blend_mode,
        })
    }

    pub fn compute(&self, input: &crate::graph::Noise, x: f32, z: f32, seed: i32) -> f32 {
        let value = input.compute(x, z, seed);
        let mut cache = [-1.0f32; 25];
        let erosion = self.get_erosion_value(x, z, &mut cache, input);
        crate::noise_util::lerp_f32(
            erosion,
            value,
            self.blend_mode.blend(value, erosion, self.strength),
        )
    }

    fn get_erosion_value(
        &self,
        mut x: f32,
        mut y: f32,
        cache: &mut [f32; 25],
        input: &crate::graph::Noise,
    ) -> f32 {
        let mut sum = 0.0f32;
        let mut max = 0.0f32;
        let mut gain = 1.0f32;
        let mut distance = self.grid_size;
        let mut i = 0;
        while i < self.octaves {
            let mut value = self.get_single_erosion_value(x, y, distance, cache, input);
            value *= gain;
            sum += value;
            max += gain;
            gain *= self.amplitude;
            distance *= self.distance_falloff;
            x *= self.lacunarity;
            y *= self.lacunarity;
            i += 1;
        }
        sum / max
    }

    fn get_single_erosion_value(
        &self,
        x: f32,
        y: f32,
        grid_size: f32,
        cache: &mut [f32; 25],
        input: &crate::graph::Noise,
    ) -> f32 {
        for slot in cache.iter_mut() {
            *slot = -1.0f32;
        }
        let pix = crate::noise_util::floor_legacy(x / grid_size);
        let piy = crate::noise_util::floor_legacy(y / grid_size);
        let mut min_height2 = f32::MAX;
        let mut dy1 = -1i32;
        while dy1 <= 1 {
            let mut dx1 = -1i32;
            while dx1 <= 1 {
                let pax = pix.wrapping_add(dx1);
                let pay = piy.wrapping_add(dy1);
                let vec1 = crate::noise_util::cell(self.seed, pax, pay);
                let ax = (pax as f32 + vec1.0) * grid_size;
                let ay = (pay as f32 + vec1.1) * grid_size;
                let mut bx = ax;
                let mut by = ay;
                let mut lowest_neighbour = f32::MAX;
                let mut dy2 = -1i32;
                while dy2 <= 1 {
                    let mut dx2 = -1i32;
                    while dx2 <= 1 {
                        let pbx = pax.wrapping_add(dx2);
                        let pby = pay.wrapping_add(dy2);
                        let vec2 = if pbx == pax && pby == pay {
                            vec1
                        } else {
                            crate::noise_util::cell(self.seed, pbx, pby)
                        };
                        let candidate_x = (pbx as f32 + vec2.0) * grid_size;
                        let candidate_y = (pby as f32 + vec2.1) * grid_size;
                        let height = get_noise_value(
                            dx1 + dx2,
                            dy1 + dy2,
                            candidate_x,
                            candidate_y,
                            input,
                            cache,
                        );
                        if height < lowest_neighbour {
                            lowest_neighbour = height;
                            bx = candidate_x;
                            by = candidate_y;
                        }
                        dx2 += 1;
                    }
                    dy2 += 1;
                }
                let height2 = sd(x, y, ax, ay, bx, by);
                if height2 < min_height2 {
                    min_height2 = height2;
                }
                dx1 += 1;
            }
            dy1 += 1;
        }
        crate::noise_util::clamp_f32(
            crate::noise_util::sqrt(min_height2) / grid_size,
            0.0f32,
            1.0f32,
        )
    }
}

fn get_noise_value(
    dx: i32,
    dy: i32,
    px: f32,
    py: f32,
    input: &crate::graph::Noise,
    cache: &mut [f32; 25],
) -> f32 {
    let index = ((dy + 2) * 5 + (dx + 2)) as usize;
    let mut value = cache[index];
    if value == -1.0f32 {
        value = input.compute(px, py, 0);
        cache[index] = value;
    }
    value
}

fn sd(px: f32, py: f32, ax: f32, ay: f32, bx: f32, by: f32) -> f32 {
    let padx = px - ax;
    let pady = py - ay;
    let badx = bx - ax;
    let bady = by - ay;
    let paba = padx * badx + pady * bady;
    let baba = badx * badx + bady * bady;
    let h = crate::noise_util::clamp_f32(paba / baba, 0.0f32, 1.0f32);
    len2(padx, pady, badx * h, bady * h)
}

fn len2(x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
    let dx = x2 - x1;
    let dy = y2 - y1;
    dx * dx + dy * dy
}
