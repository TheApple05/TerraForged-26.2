//! MIT ReTerraForged source-owned climate categories/palette, not terrain samples.
use crate::{cell::BiomeType, noise_util};
use std::sync::LazyLock;
const TABLE: &[u8; 65536] = include_bytes!("data/biome-types.bin");
pub const ALL: [BiomeType; 11] = [
    BiomeType::TropicalRainforest,
    BiomeType::Savanna,
    BiomeType::Desert,
    BiomeType::TemperateRainforest,
    BiomeType::TemperateForest,
    BiomeType::Grassland,
    BiomeType::ColdSteppe,
    BiomeType::Steppe,
    BiomeType::Taiga,
    BiomeType::Tundra,
    BiomeType::Alpine,
];
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BiomeLookupError;
#[derive(Clone, Copy, Debug)]
pub struct Ranges {
    pub min_temperature: f32,
    pub max_temperature: f32,
    pub min_moisture: f32,
    pub max_moisture: f32,
}
static RANGES: LazyLock<[Ranges; 11]> = LazyLock::new(|| {
    let mut result = [Ranges {
        min_temperature: 1.0,
        max_temperature: 0.0,
        min_moisture: 1.0,
        max_moisture: 0.0,
    }; 11];
    for y in 0..256 {
        for x in 0..256 {
            let r = &mut result[TABLE[y * 256 + x] as usize];
            let t = x as f32 / 255.0;
            let m = y as f32 / 255.0;
            r.min_temperature = r.min_temperature.min(t);
            r.max_temperature = r.max_temperature.max(t);
            r.min_moisture = r.min_moisture.min(m);
            r.max_moisture = r.max_moisture.max(m);
        }
    }
    result
});
fn at(x: i32, y: i32) -> Result<BiomeType, BiomeLookupError> {
    if !(0..256).contains(&x) || !(0..256).contains(&y) {
        return Err(BiomeLookupError);
    }
    Ok(ALL[TABLE[y as usize * 256 + x as usize] as usize])
}
pub fn get_curve(temperature: f32, moisture: f32) -> Result<BiomeType, BiomeLookupError> {
    let x = noise_util::round_legacy(255.0 * temperature);
    let max = x.wrapping_add(255i32.wrapping_sub(x) / 2);
    let y = noise_util::round_legacy(max as f32 * moisture);
    at(x, x.min(y))
}
pub fn get_linear(temperature: f32, moisture: f32) -> Result<BiomeType, BiomeLookupError> {
    let x = noise_util::round_legacy(255.0 * temperature);
    let y = if moisture > temperature {
        x
    } else {
        noise_util::round_legacy(255.0 * moisture)
    };
    at(x, y)
}
const TEMPERATURE: [(f32, f32); 5] = [
    (-1.0, -0.45),
    (-0.45, -0.15),
    (-0.15, 0.2),
    (0.2, 0.55),
    (0.55, 1.0),
];
const HUMIDITY: [(f32, f32); 5] = [
    (-1.0, -0.35),
    (-0.35, -0.1),
    (-0.1, 0.1),
    (0.1, 0.3),
    (0.3, 1.0),
];
impl BiomeType {
    pub fn from_index(index: i32) -> Option<Self> {
        ALL.get(index as usize).copied()
    }
    pub fn name(self) -> &'static str {
        match self {
            Self::TropicalRainforest => "TROPICAL_RAINFOREST",
            Self::Savanna => "SAVANNA",
            Self::Desert => "DESERT",
            Self::TemperateRainforest => "TEMPERATE_RAINFOREST",
            Self::TemperateForest => "TEMPERATE_FOREST",
            Self::Grassland => "GRASSLAND",
            Self::ColdSteppe => "COLD_STEPPE",
            Self::Steppe => "STEPPE",
            Self::Taiga => "TAIGA",
            Self::Tundra => "TUNDRA",
            Self::Alpine => "ALPINE",
        }
    }
    pub fn ranges(self) -> Ranges {
        RANGES[self as usize]
    }
    fn pairs(self) -> &'static [(usize, usize)] {
        match self {
            Self::TropicalRainforest => &[(3, 3), (3, 4)],
            Self::Savanna => &[(3, 0), (3, 1)],
            Self::Desert => &[(4, 0), (4, 1), (4, 2), (4, 3), (4, 4)],
            Self::TemperateRainforest => &[(2, 4), (2, 1), (2, 1), (2, 1), (2, 1), (2, 1)],
            Self::TemperateForest => &[(2, 2), (2, 3), (2, 1), (2, 1), (2, 1), (2, 1), (2, 1)],
            Self::Grassland => &[(1, 0), (1, 1), (2, 0), (2, 1)],
            Self::ColdSteppe => &[(0, 4)],
            Self::Steppe => &[(1, 1), (1, 2), (2, 1), (2, 2)],
            Self::Taiga => &[(1, 3), (1, 4)],
            Self::Tundra => &[(0, 0), (0, 1), (0, 2), (0, 3)],
            Self::Alpine => &[(1, 4)],
        }
    }
    pub fn temperature_moisture(self, identity: f32) -> Result<(f32, f32), BiomeLookupError> {
        let pairs = self.pairs();
        let index = noise_util::round_legacy(identity * (pairs.len() - 1) as f32);
        let &(t, h) = pairs.get(index as usize).ok_or(BiomeLookupError)?;
        let t = TEMPERATURE[t];
        let h = HUMIDITY[h];
        Ok(((t.0 + t.1) / 2.0, (h.0 + h.1) / 2.0))
    }
    pub fn map_temperature(self, value: f32) -> f32 {
        let r = self.ranges();
        (value - r.min_temperature) / (r.max_temperature - r.min_temperature)
    }
    pub fn map_moisture(self, value: f32) -> f32 {
        let r = self.ranges();
        (value - r.min_moisture) / (r.max_moisture - r.min_moisture)
    }
}
