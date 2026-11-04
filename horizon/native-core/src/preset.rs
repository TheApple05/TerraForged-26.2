// DTO shapes derived from ReTerraForged MIT-licensed preset settings classes.
// Pure cell-kernel data; JSON adapters and construction algorithms are separate.

use crate::worley::DistanceFunction;

#[derive(Clone, Debug)]
pub struct KernelPreset {
    pub world: WorldSettings,
    pub terrain: TerrainSettings,
    pub climate: ClimateSettings,
    pub rivers: RiverSettings,
    pub filters: FilterSettings,
}

#[derive(Clone, Debug)]
pub struct WorldSettings {
    pub continent: ContinentSettings,
    pub control_points: ControlPointsSettings,
    pub properties: WorldProperties,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContinentType {
    Multi,
    Single,
    MultiImproved,
    Experimental,
    Infinite,
}
impl ContinentType {
    pub fn serialized_name(self) -> &'static str {
        match self {
            Self::Multi => "MULTI",
            Self::Single => "SINGLE",
            Self::MultiImproved => "MULTI_IMPROVED",
            Self::Experimental => "EXPERIMENTAL",
            Self::Infinite => "INFINITE",
        }
    }
}

#[derive(Clone, Debug)]
pub struct ContinentSettings {
    pub continent_type: ContinentType,
    pub continent_shape: DistanceFunction,
    pub continent_scale: i32,
    pub continent_jitter: f32,
    pub continent_skipping: f32,
    pub continent_size_variance: f32,
    pub continent_noise_octaves: i32,
    pub continent_noise_gain: f32,
    pub continent_noise_lacunarity: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct ControlPointsSettings {
    pub mushroom_fields_inland: f32,
    pub mushroom_fields_coast: f32,
    pub deep_ocean: f32,
    pub shallow_ocean: f32,
    pub beach: f32,
    pub coast: f32,
    pub inland: f32,
}

#[derive(Clone, Debug)]
pub struct WorldProperties {
    pub spawn_type: String,
    pub world_height: i32,
    pub world_depth: i32,
    pub sea_level: i32,
    pub lava_level: i32,
}

#[derive(Clone, Debug)]
pub struct TerrainSettings {
    pub general: GeneralTerrain,
    pub steppe: TerrainSettingsEntry,
    pub plains: TerrainSettingsEntry,
    pub hills: TerrainSettingsEntry,
    pub dales: TerrainSettingsEntry,
    pub plateau: TerrainSettingsEntry,
    pub badlands: TerrainSettingsEntry,
    pub torridonian: TerrainSettingsEntry,
    pub mountains: TerrainSettingsEntry,
    pub volcano: TerrainSettingsEntry,
}

#[derive(Clone, Copy, Debug)]
pub struct GeneralTerrain {
    pub terrain_seed_offset: i32,
    pub terrain_region_size: i32,
    pub global_vertical_scale: f32,
    pub global_horizontal_scale: f32,
    pub fancy_mountains: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct TerrainSettingsEntry {
    pub weight: f32,
    pub base_scale: f32,
    pub vertical_scale: f32,
    pub horizontal_scale: f32,
}

#[derive(Clone, Debug)]
pub struct ClimateSettings {
    pub temperature: ClimateRange,
    pub moisture: ClimateRange,
    pub biome_shape: BiomeShape,
    pub biome_edge_shape: BiomeNoise,
}

#[derive(Clone, Copy, Debug)]
pub struct ClimateRange {
    pub seed_offset: i32,
    pub scale: i32,
    pub falloff: i32,
    pub min: f32,
    pub max: f32,
    pub bias: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct BiomeShape {
    pub biome_size: i32,
    pub macro_noise_size: i32,
    pub biome_warp_scale: i32,
    pub biome_warp_strength: i32,
}

#[derive(Clone, Copy, Debug)]
pub struct BiomeNoise {
    pub r#type: EdgeType,
    pub scale: i32,
    pub octaves: i32,
    pub gain: f32,
    pub lacunarity: f32,
    pub strength: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EdgeType {
    Billow,
    Voronoi,
    VoronoiEdge,
    Constant,
    Cubic,
    Perlin,
    Perlin2,
    PerlinRidge,
    Simplex,
    Simplex2,
    SimplexRidge,
    Sin,
    White,
}
impl EdgeType {
    pub fn serialized_name(self) -> &'static str {
        match self {
            Self::Billow => "BILLOW",
            Self::Voronoi => "CELL",
            Self::VoronoiEdge => "CELL_EDGE",
            Self::Constant => "CONST",
            Self::Cubic => "CUBIC",
            Self::Perlin => "PERLIN",
            Self::Perlin2 => "PERLIN2",
            Self::PerlinRidge => "RIDGE",
            Self::Simplex => "SIMPLEX",
            Self::Simplex2 => "SIMPLEX2",
            Self::SimplexRidge => "SIMPLEX_RIDGE",
            Self::Sin => "SIN",
            Self::White => "RAND",
        }
    }
}

#[derive(Clone, Debug)]
pub struct RiverSettings {
    pub seed_offset: i32,
    pub river_count: i32,
    pub main_rivers: RiverProfile,
    pub branch_rivers: RiverProfile,
    pub lakes: LakeSettings,
    pub wetlands: WetlandSettings,
}

#[derive(Clone, Copy, Debug)]
pub struct RiverProfile {
    pub bed_depth: i32,
    pub min_bank_height: i32,
    pub max_bank_height: i32,
    pub bank_width: i32,
    pub bed_width: i32,
    pub fade: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct LakeSettings {
    pub chance: f32,
    pub min_start_distance: f32,
    pub max_start_distance: f32,
    pub depth: i32,
    pub size_min: i32,
    pub size_max: i32,
    pub min_bank_height: i32,
    pub max_bank_height: i32,
}

#[derive(Clone, Copy, Debug)]
pub struct WetlandSettings {
    pub chance: f32,
    pub size_min: i32,
    pub size_max: i32,
}

#[derive(Clone, Debug)]
pub struct FilterSettings {
    pub erosion: ErosionSettings,
    pub smoothing: SmoothingSettings,
}

#[derive(Clone, Copy, Debug)]
pub struct ErosionSettings {
    pub droplets_per_chunk: i32,
    pub droplet_lifetime: i32,
    pub droplet_volume: f32,
    pub droplet_velocity: f32,
    pub erosion_rate: f32,
    // Misspelled to match source field name.
    pub deposite_rate: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct SmoothingSettings {
    pub iterations: i32,
    pub smoothing_radius: f32,
    pub smoothing_rate: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct ControlPoints {
    pub mushroom_fields_inland: f32,
    pub mushroom_fields_coast: f32,
    pub deep_ocean: f32,
    pub shallow_ocean: f32,
    pub beach: f32,
    pub coast: f32,
    pub coast_marker: f32,
    pub inland: f32,
}
impl ControlPointsSettings {
    pub fn validate(&self) -> bool {
        self.inland <= 1.0
            && self.inland > self.coast
            && self.coast > self.beach
            && self.beach > self.shallow_ocean
            && self.shallow_ocean > self.deep_ocean
            && self.mushroom_fields_coast >= self.mushroom_fields_inland
            && self.mushroom_fields_inland >= 0.0
    }
    pub fn make(&self) -> ControlPoints {
        let p = if self.validate() {
            *self
        } else {
            Self {
                mushroom_fields_inland: 0.0,
                mushroom_fields_coast: 0.1,
                deep_ocean: 0.1,
                shallow_ocean: 0.25,
                beach: 0.327,
                coast: 0.448,
                inland: 0.502,
            }
        };
        ControlPoints {
            mushroom_fields_inland: p.mushroom_fields_inland,
            mushroom_fields_coast: p.mushroom_fields_coast,
            deep_ocean: p.deep_ocean,
            shallow_ocean: p.shallow_ocean,
            beach: p.beach,
            coast: p.coast,
            coast_marker: p.coast + (p.inland - p.coast) / 2.0,
            inland: p.inland,
        }
    }
}
impl WorldProperties {
    pub fn terrain_scaler(&self) -> i32 {
        self.world_height.min(256)
    }
}
impl ClimateRange {
    pub fn get_min(&self) -> f32 {
        crate::noise_util::clamp_f32(crate::graph::java_min(self.min, self.max), 0.0, 1.0)
    }
    pub fn get_max(&self) -> f32 {
        crate::noise_util::clamp_f32(
            crate::graph::java_max(self.min, self.max),
            self.get_min(),
            1.0,
        )
    }
    pub fn get_bias(&self) -> f32 {
        crate::noise_util::clamp_f32(self.bias, -1.0, 1.0)
    }
    pub fn apply(
        &self,
        input: crate::graph::Noise,
    ) -> Result<crate::graph::Noise, crate::graph::GraphError> {
        use crate::graph::Noise;
        let min = self.get_min();
        let max = self.get_max();
        let bias = self.get_bias() / 2.0;
        Noise::clamp(
            Noise::add(input, Noise::constant(bias)?)?,
            Noise::constant(min)?,
            Noise::constant(max)?,
        )
    }
}
impl BiomeNoise {
    pub fn build(&self, seed: i32) -> Result<crate::graph::Noise, crate::graph::GraphError> {
        use crate::{graph::Noise, noises as n};
        let raw = match self.r#type {
            EdgeType::Billow => {
                n::billow_with(seed, self.scale, self.octaves, self.lacunarity, self.gain)?
            }
            EdgeType::Voronoi => n::worley(seed, self.scale)?,
            EdgeType::VoronoiEdge => n::worley_edge(seed, self.scale)?,
            EdgeType::Constant => Noise::constant(1.0)?,
            EdgeType::Cubic => {
                n::cubic_with(seed, self.scale, self.octaves, self.lacunarity, self.gain)?
            }
            EdgeType::Perlin => {
                n::perlin_with(seed, self.scale, self.octaves, self.lacunarity, self.gain)?
            }
            EdgeType::Perlin2 => {
                n::perlin2_with(seed, self.scale, self.octaves, self.lacunarity, self.gain)?
            }
            EdgeType::PerlinRidge => {
                n::perlin_ridge_with(seed, self.scale, self.octaves, self.lacunarity, self.gain)?
            }
            EdgeType::Simplex => {
                n::simplex_with(seed, self.scale, self.octaves, self.lacunarity, self.gain)?
            }
            // Intentional source compatibility alias: this is Simplex2, not SimplexRidge.
            EdgeType::Simplex2 | EdgeType::SimplexRidge => {
                n::simplex2_with(seed, self.scale, self.octaves, self.lacunarity, self.gain)?
            }
            EdgeType::Sin => n::sin(seed, 1.0, Noise::constant(0.0)?)?,
            EdgeType::White => n::white(seed, 1)?,
        };
        Noise::add(raw, Noise::constant(-0.5)?)
    }
}
