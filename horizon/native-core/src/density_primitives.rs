//! MIT ReTerraForged custom density primitives. Host graph binding is separate.
use crate::{
    cell::Cell,
    graph::{GraphError, Noise},
    layout::Levels,
    noise_util as u,
    preset::ControlPoints,
    terrain::{Category, TerrainRegistry},
};
type R<T> = Result<T, GraphError>;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CellField {
    Height,
    Continent,
    Erosion,
    Weirdness,
    BiomeRegion,
    Temperature,
    Moisture,
    Gradient,
    HeightErosion,
    Sediment,
}
impl CellField {
    pub const ALL: [Self; 10] = [
        Self::Height,
        Self::Continent,
        Self::Erosion,
        Self::Weirdness,
        Self::BiomeRegion,
        Self::Temperature,
        Self::Moisture,
        Self::Gradient,
        Self::HeightErosion,
        Self::Sediment,
    ];
    pub fn name(self) -> &'static str {
        match self {
            Self::Height => "height",
            Self::Continent => "continent",
            Self::Erosion => "erosion",
            Self::Weirdness => "weirdness",
            Self::BiomeRegion => "biome_region",
            Self::Temperature => "temperature",
            Self::Moisture => "moisture",
            Self::Gradient => "gradient",
            Self::HeightErosion => "height_erosion",
            Self::Sediment => "sediment",
        }
    }
    pub fn parse(name: &str) -> R<Self> {
        Self::ALL
            .into_iter()
            .find(|f| f.name() == name)
            .ok_or(GraphError::InvalidSourceParameters(
                "registered cell density field",
            ))
    }
    pub fn read(
        self,
        cell: &Cell,
        levels: Levels,
        points: ControlPoints,
        terrains: &TerrainRegistry,
    ) -> R<f32> {
        Ok(match self {
            Self::Height => cell.height,
            Self::Erosion => cell.erosion,
            Self::Weirdness => cell.weirdness,
            Self::BiomeRegion => cell.biome_region_id,
            Self::Temperature => cell.temperature,
            Self::Moisture => cell.moisture,
            Self::Gradient => cell.gradient,
            Self::HeightErosion => cell.height_erosion,
            Self::Sediment => cell.sediment,
            Self::Continent => {
                let terrain =
                    terrains
                        .get(cell.terrain)
                        .ok_or(GraphError::InvalidSourceParameters(
                            "owned continent density terrain ID",
                        ))?;
                if terrain.is_deep_ocean() {
                    let alpha = u::clamp_f32(cell.continent_edge, 0.0, points.deep_ocean);
                    let alpha = u::lerp_map(alpha, 0.0, points.deep_ocean, 0.0, 1.0);
                    u::lerp_f32(-1.05f32 + 0.05, -0.455, alpha)
                } else if terrain.is_shallow_ocean() {
                    let alpha =
                        u::clamp_f32(cell.continent_edge, points.deep_ocean, points.shallow_ocean);
                    let alpha =
                        u::lerp_map(alpha, points.deep_ocean, points.shallow_ocean, 0.0, 0.98);
                    u::lerp_f32(-0.455, -0.19, alpha)
                } else if terrain.category() == Category::Beach
                    && cell.height + cell.beach_noise < levels.water(5)
                {
                    let alpha =
                        u::clamp_f32(cell.continent_edge, points.shallow_ocean, points.beach);
                    let alpha = u::lerp_map(alpha, points.shallow_ocean, points.beach, 0.0, 1.0);
                    u::lerp_f32(-0.19, -0.11, alpha)
                } else {
                    let alpha = u::clamp_f32(cell.continent_edge, points.beach, points.inland);
                    let alpha = u::lerp_map(alpha, points.beach, points.inland, 0.0, 1.0);
                    u::lerp_f32((-0.11f32 + 0.03f32) / 2.0, 1.0, alpha)
                }
            }
        })
    }
}
/// Source truncates double*int to int, adds one with int wrapping, converts to
/// float and divides as float before widening. It is not nearest rounding.
pub fn clamp_to_nearest_unit(value: f64, resolution: i32) -> f64 {
    let scaled = ((value * resolution as f64) as i32).wrapping_add(1) as f32;
    (scaled / resolution as f32) as f64
}
pub fn noise_value(noise: &Noise, position: [i32; 3], seed: i32) -> f64 {
    noise.compute(position[0] as f32, position[2] as f32, seed) as f64
}
#[derive(Clone, Debug)]
pub struct LinearSpline {
    locations: Vec<f64>,
}
impl LinearSpline {
    /// Used source tables are nondecreasing. Native bounds resource use and
    /// rejects unsorted/nonfinite tables explicitly rather than guessing results.
    pub fn new(locations: Vec<f64>) -> R<Self> {
        if locations.is_empty()
            || locations.len() > 256
            || locations.iter().any(|v| !v.is_finite())
            || locations.windows(2).any(|p| p[0] > p[1])
        {
            return Err(GraphError::InvalidSpline);
        }
        Ok(Self { locations })
    }
    pub fn compute(&self, input: f64, mut value: impl FnMut(usize) -> f64) -> R<f64> {
        if input.is_nan() {
            return Err(GraphError::InvalidSourceParameters(
                "non-NaN linear spline input",
            ));
        }
        let count = self.locations.len();
        if input <= self.locations[0] {
            return Ok(value(0));
        }
        if input >= self.locations[count - 1] {
            return Ok(value(count - 1));
        }
        let mut low = 0;
        let mut high = count;
        while low < high {
            let mid = (low + high) / 2;
            if input < self.locations[mid] {
                high = mid;
            } else {
                low = mid + 1;
            }
        }
        let index = low - 1;
        let min = self.locations[index];
        let max = self.locations[index + 1];
        let from = value(index);
        let to = value(index + 1);
        let alpha = u::clamp_f64(u::map_f64(input, 0.0, 1.0, min, max), 0.0, 1.0);
        Ok(u::lerp_f64(from, to, alpha))
    }
    /// Legacy reported bounds narrow the child double extrema through float.
    /// These metadata values must not be treated as proven host optimization bounds.
    pub fn source_bounds(&self, values: &[(f64, f64)]) -> R<(f64, f64)> {
        if values.len() != self.locations.len() {
            return Err(GraphError::InvalidSpline);
        }
        let min = values
            .iter()
            .map(|v| v.0)
            .reduce(java_min)
            .ok_or(GraphError::InvalidSpline)?;
        let max = values
            .iter()
            .map(|v| v.1)
            .reduce(java_max)
            .ok_or(GraphError::InvalidSpline)?;
        Ok((min as f32 as f64, max as f32 as f64))
    }
}
fn java_min(a: f64, b: f64) -> f64 {
    if a.is_nan() || b.is_nan() {
        f64::NAN
    } else if a == 0.0 && b == 0.0 {
        if a.is_sign_negative() || b.is_sign_negative() {
            -0.0
        } else {
            0.0
        }
    } else if a < b {
        a
    } else {
        b
    }
}
fn java_max(a: f64, b: f64) -> f64 {
    if a.is_nan() || b.is_nan() {
        f64::NAN
    } else if a == 0.0 && b == 0.0 {
        if a.is_sign_positive() || b.is_sign_positive() {
            0.0
        } else {
            -0.0
        }
    } else if a > b {
        a
    } else {
        b
    }
}
#[derive(Clone, Debug)]
pub struct ConditionalArray {
    first_x: i32,
    first_z: i32,
    width: i32,
    values: Vec<f32>,
}
impl ConditionalArray {
    pub fn new(
        first_x: i32,
        first_z: i32,
        width: i32,
        mut sample: impl FnMut([i32; 3]) -> R<f64>,
    ) -> R<Self> {
        if !(0..=256).contains(&width) {
            return Err(GraphError::InvalidSourceParameters(
                "conditional density cache width 0..256",
            ));
        }
        let mut values = vec![0.0; (width * width) as usize];
        for x in 0..width {
            for z in 0..width {
                values[(x * width + z) as usize] =
                    sample([first_x.wrapping_add(x), 0, first_z.wrapping_add(z)])? as f32;
            }
        }
        Ok(Self {
            first_x,
            first_z,
            width,
            values,
        })
    }
    pub fn compute(&self, position: [i32; 3], fallback: impl FnOnce([i32; 3]) -> R<f64>) -> R<f64> {
        let x = position[0].wrapping_sub(self.first_x);
        let z = position[2].wrapping_sub(self.first_z);
        if x >= 0 && z >= 0 && x < self.width && z < self.width {
            Ok(self.values[(x * self.width + z) as usize] as f64)
        } else {
            fallback(position)
        }
    }
}
