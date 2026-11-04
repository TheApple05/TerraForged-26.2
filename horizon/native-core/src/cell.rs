//! Owned value state corresponding to MIT ReTerraForged Cell.java.
//! Terrain IDs belong to the owning kernel's registry; built-in NONE has ID0.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum BiomeType {
    TropicalRainforest,
    Savanna,
    Desert,
    TemperateRainforest,
    TemperateForest,
    Grassland,
    ColdSteppe,
    Steppe,
    Taiga,
    Tundra,
    Alpine,
}

#[derive(Clone, Copy, Debug)]
pub struct Cell {
    pub height: f32,
    pub height_erosion: f32,
    pub sediment: f32,
    pub gradient: f32,
    pub region_moisture: f32,
    pub region_temperature: f32,
    pub continent_id: f32,
    pub continent_edge: f32,
    pub terrain_region_id: f32,
    pub terrain_region_edge: f32,
    pub biome_region_id: f32,
    pub biome_region_edge: f32,
    pub macro_biome_id: f32,
    pub river_mask: f32,
    pub continent_x: i32,
    pub continent_z: i32,
    pub erosion_mask: bool,
    pub terrain: usize,
    pub biome: BiomeType,
    pub erosion: f32,
    pub weirdness: f32,
    pub temperature: f32,
    pub moisture: f32,
    pub beach_noise: f32,
}
impl Default for Cell {
    fn default() -> Self {
        Self {
            height: 0.0,
            height_erosion: 0.0,
            sediment: 0.0,
            gradient: 0.0,
            region_moisture: 0.5,
            region_temperature: 0.5,
            continent_id: 0.0,
            continent_edge: 0.0,
            terrain_region_id: 0.0,
            terrain_region_edge: 0.0,
            biome_region_id: 0.0,
            biome_region_edge: 1.0,
            macro_biome_id: 0.0,
            river_mask: 1.0,
            continent_x: 0,
            continent_z: 0,
            erosion_mask: false,
            terrain: 0,
            biome: BiomeType::Grassland,
            erosion: 0.0,
            weirdness: 0.0,
            temperature: 0.0,
            moisture: 0.0,
            beach_noise: 0.0,
        }
    }
}
impl Cell {
    pub fn copy_from(&mut self, other: &Self) {
        *self = *other;
    }
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}
