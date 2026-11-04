/*
 * MIT License. Copyright (c) 2020-2021 TerraForged.
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 */
//! Classic pillar graphs, material bands and palettes; world placement is separate.
use crate::{
    classic_pillars,
    graph::{GraphError, Noise, java_max, java_min},
    java_random::JavaRandom,
    noises,
};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Material {
    Orange,
    Terracotta,
    Brown,
    Yellow,
    White,
    LightGray,
    Stone,
    Dirt,
    Grass,
}
impl Material {
    pub fn id(self) -> &'static str {
        match self {
            Self::Orange => "minecraft:orange_terracotta",
            Self::Terracotta => "minecraft:terracotta",
            Self::Brown => "minecraft:brown_terracotta",
            Self::Yellow => "minecraft:yellow_terracotta",
            Self::White => "minecraft:white_terracotta",
            Self::LightGray => "minecraft:light_gray_terracotta",
            Self::Stone => "minecraft:stone",
            Self::Dirt => "minecraft:dirt",
            Self::Grass => "minecraft:grass_block",
        }
    }
}
#[derive(Clone, Debug)]
pub struct Palette {
    materials: [Material; 6],
    ends: [f32; 6],
}
impl Palette {
    pub fn ends(&self) -> [f32; 6] {
        self.ends
    }
    pub fn at(&self, y: i32, height: i32) -> Result<Material, GraphError> {
        if height <= 0 {
            return Err(GraphError::InvalidSourceParameters(
                "positive pillar palette height",
            ));
        }
        let alpha = (y % height) as f32 / height as f32;
        for (i, &end) in self.ends.iter().enumerate() {
            if alpha < end {
                return Ok(self.materials[i]);
            }
        }
        Ok(self.materials[5])
    }
}
#[derive(Clone, Debug)]
pub struct PillarSampler {
    seed: i32,
    bryce: Noise,
    stone_a: Noise,
    stone_b: Noise,
    detail: Noise,
    bands: Noise,
    palettes: [Palette; 9],
}
fn ridge(
    offset: i32,
    scale: i32,
    lower: f32,
    maximum: f32,
    power: bool,
) -> Result<Noise, GraphError> {
    let value = Noise::clamp(
        noises::perlin_ridge(offset, scale, 4)?,
        Noise::constant(lower)?,
        Noise::constant(0.95)?,
    )?;
    let value = Noise::map(value, Noise::constant(0.0)?, Noise::constant(maximum)?)?;
    if power {
        Noise::power(value, 1.5)
    } else {
        Ok(value)
    }
}
impl PillarSampler {
    pub fn new(seed: i32) -> Result<Self, GraphError> {
        let mut random = JavaRandom::new(i64::from(seed.wrapping_add(12_341_341)));
        let palettes = std::array::from_fn(|i| {
            let materials = [
                Material::Orange,
                Material::Terracotta,
                Material::Orange,
                if i % 3 == 2 {
                    Material::White
                } else if i % 3 == 1 {
                    Material::Yellow
                } else {
                    Material::Brown
                },
                if i % 3 == 0 {
                    Material::Yellow
                } else {
                    Material::LightGray
                },
                Material::Terracotta,
            ];
            let mut ends = [0.0_f32; 6];
            let mut total = 0.0_f32;
            for (band, width) in ends.iter_mut().enumerate() {
                *width = if band == 2 || band == 5 { 0.2 } else { 0.1 } + random.next_f32() * 0.1;
                total += *width;
            }
            for band in 1..6 {
                ends[band] += ends[band - 1];
            }
            for end in &mut ends {
                *end /= total;
            }
            Palette { materials, ends }
        });
        Ok(Self {
            seed,
            bryce: ridge(12_341_342, 60, 0.8, 0.999, false)?,
            stone_a: ridge(-2_345_768, 50, 0.7, 1.0, true)?,
            stone_b: ridge(-2_345_767, 55, 0.7, 1.0, true)?,
            detail: Noise::alpha(
                noises::perlin(seed.wrapping_add(12_341_343), 4, 1)?,
                Noise::constant(0.05)?,
            )?,
            bands: noises::perlin(seed.wrapping_add(12_341_345), 100, 1)?,
            palettes,
        })
    }
    pub fn palette(&self, index: usize) -> Option<&Palette> {
        self.palettes.get(index)
    }
    pub fn palette_index(region: f32) -> usize {
        ((java_max(0.0, java_min(0.999999, region)) * 9.0) as i32).min(8) as usize
    }
    pub fn shape(&self, x: i32, z: i32, stone: bool) -> f32 {
        let (x, z) = (x as f32, z as f32);
        if stone {
            java_max(
                classic_pillars::terrace(self.stone_a.compute(x, z, self.seed), 1.0),
                classic_pillars::terrace(self.stone_b.compute(x, z, self.seed), 1.0),
            )
        } else {
            classic_pillars::terrace(self.bryce.compute(x, z, self.seed), 0.999)
                * self.detail.compute(x, z, self.seed)
        }
    }
    pub fn band_value(&self, x: i32, z: i32) -> f32 {
        self.bands.compute(x as f32, z as f32, self.seed)
    }
    pub fn band_shift(&self, x: i32, z: i32) -> i32 {
        (self.band_value(x, z) * 4.0) as i32
    }
    pub fn stone_material(relative_y: i32, height: i32) -> Material {
        if relative_y == height {
            Material::Grass
        } else if relative_y == height.wrapping_sub(1) {
            Material::Dirt
        } else {
            Material::Stone
        }
    }
}
