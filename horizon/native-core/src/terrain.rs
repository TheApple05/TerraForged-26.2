//! MIT ReTerraForged ITerrain/Terrain/ConfiguredTerrain/CompositeTerrain/TerrainType.
//! Names/IDs belong to a kernel-owned registry; no process-wide mutable registry.
use crate::noise_util;
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum Category {
    None,
    DeepOcean,
    ShallowOcean,
    Coast,
    Beach,
    River,
    Lake,
    Wetland,
    Flatland,
    Lowland,
    Highland,
}
impl Category {
    pub fn dominant(self, other: Self) -> Self {
        if self > other { self } else { other }
    }
    pub fn name(self) -> &'static str {
        match self {
            Self::None => "NONE",
            Self::DeepOcean => "DEEP_OCEAN",
            Self::ShallowOcean => "SHALLOW_OCEAN",
            Self::Coast => "COAST",
            Self::Beach => "BEACH",
            Self::River => "RIVER",
            Self::Lake => "LAKE",
            Self::Wetland => "WETLAND",
            Self::Flatland => "FLATLAND",
            Self::Lowland => "LOWLAND",
            Self::Highland => "HIGHLAND",
        }
    }
    fn flags(self) -> u16 {
        match self {
            Self::None => 0,
            Self::DeepOcean => DEEP | SUBMERGED | OVERRIDE_RIVER,
            Self::ShallowOcean => SHALLOW | SUBMERGED | OVERRIDE_RIVER,
            Self::Coast | Self::Beach => COAST | OVERGROUND | OVERRIDE_RIVER,
            Self::River => RIVER | SUBMERGED,
            Self::Lake => LAKE | SUBMERGED,
            Self::Wetland => WETLAND | OVERGROUND,
            Self::Flatland => FLAT | OVERGROUND,
            Self::Lowland | Self::Highland => OVERGROUND,
        }
    }
}
const FLAT: u16 = 1 << 0;
const RIVER: u16 = 1 << 1;
const SHALLOW: u16 = 1 << 2;
const DEEP: u16 = 1 << 3;
const COAST: u16 = 1 << 4;
const SUBMERGED: u16 = 1 << 5;
const OVERGROUND: u16 = 1 << 6;
const OVERRIDE_RIVER: u16 = 1 << 7;
const OVERRIDE_COAST: u16 = 1 << 8;
const LAKE: u16 = 1 << 9;
const WETLAND: u16 = 1 << 10;
const MOUNTAIN: u16 = 1 << 11;
const VOLCANO: u16 = 1 << 12;
#[derive(Clone, Debug)]
pub struct Terrain {
    id: usize,
    name: String,
    category: Category,
    flags: u16,
    erosion: f32,
}
impl Terrain {
    pub fn id(&self) -> usize {
        self.id
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn category(&self) -> Category {
        self.category
    }
    pub fn erosion_modifier(&self) -> f32 {
        self.erosion
    }
    pub fn render_hue(&self) -> f32 {
        let hash = self
            .name
            .encode_utf16()
            .fold(0i32, |h, c| h.wrapping_mul(31).wrapping_add(c as i32));
        noise_util::val_coord_2d(hash, 0, 0)
    }
    pub fn is_flat(&self) -> bool {
        self.flags & FLAT != 0
    }
    pub fn is_river(&self) -> bool {
        self.flags & RIVER != 0
    }
    pub fn is_shallow_ocean(&self) -> bool {
        self.flags & SHALLOW != 0
    }
    pub fn is_deep_ocean(&self) -> bool {
        self.flags & DEEP != 0
    }
    pub fn is_coast(&self) -> bool {
        self.flags & COAST != 0
    }
    pub fn is_submerged(&self) -> bool {
        self.flags & SUBMERGED != 0
    }
    pub fn is_overground(&self) -> bool {
        self.flags & OVERGROUND != 0
    }
    pub fn overrides_river(&self) -> bool {
        self.flags & OVERRIDE_RIVER != 0
    }
    pub fn overrides_coast(&self) -> bool {
        self.flags & OVERRIDE_COAST != 0
    }
    pub fn is_lake(&self) -> bool {
        self.flags & LAKE != 0
    }
    pub fn is_wetland(&self) -> bool {
        self.flags & WETLAND != 0
    }
    pub fn is_mountain(&self) -> bool {
        self.flags & MOUNTAIN != 0
    }
    pub fn is_volcano(&self) -> bool {
        self.flags & VOLCANO != 0
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TerrainError {
    InvalidParent,
    InvalidName,
    RegistryLimit,
}
#[derive(Clone, Debug)]
pub struct TerrainRegistry {
    values: Vec<Terrain>,
}
impl Default for TerrainRegistry {
    fn default() -> Self {
        Self::new()
    }
}
impl TerrainRegistry {
    pub fn new() -> Self {
        use Category::*;
        let mut values = Vec::new();
        for (name, category, extra, erosion) in [
            ("none", None, 0, 1.0),
            ("deep_ocean", DeepOcean, 0, 1.0),
            ("ocean", ShallowOcean, 0, 1.0),
            ("coast", Coast, 0, 1.0),
            ("beach", Beach, 0, 1.0),
            ("river", River, 0, 1.0),
            ("lake", Lake, 0, 1.0),
            ("wetland", Wetland, OVERRIDE_RIVER, 1.0),
            ("flats", Flatland, 0, 1.0),
            ("badlands", Flatland, 0, 0.3),
            ("plateau", Lowland, 0, 1.0),
            ("hills", Lowland, 0, 1.0),
            ("mountains_1", Highland, MOUNTAIN | OVERRIDE_RIVER, 1.0),
            ("mountains_2", Highland, MOUNTAIN | OVERRIDE_RIVER, 1.0),
            ("mountains_3", Highland, MOUNTAIN | OVERRIDE_RIVER, 1.0),
            ("mountain_chain", Highland, MOUNTAIN | OVERRIDE_RIVER, 1.0),
            (
                "volcano",
                Highland,
                MOUNTAIN | OVERRIDE_RIVER | VOLCANO | OVERRIDE_COAST,
                1.0,
            ),
            (
                "volcano_pipe",
                Highland,
                MOUNTAIN | OVERRIDE_RIVER | VOLCANO | OVERRIDE_COAST,
                1.0,
            ),
        ] {
            values.push(Terrain {
                id: values.len(),
                name: name.into(),
                category,
                flags: category.flags() | extra,
                erosion,
            });
        }
        Self { values }
    }
    pub fn iter(&self) -> impl Iterator<Item = &Terrain> {
        self.values.iter()
    }
    pub fn get(&self, id: usize) -> Option<&Terrain> {
        self.values.get(id)
    }
    /// Java's get(int) returns NONE for an invalid ID.
    pub fn get_or_none(&self, id: i32) -> &Terrain {
        self.values.get(id as usize).unwrap_or(&self.values[0])
    }
    pub fn named(&self, name: &str) -> Option<&Terrain> {
        self.values
            .iter()
            .find(|t| t.name.eq_ignore_ascii_case(name))
    }
    pub fn get_or_create(
        &mut self,
        name: &str,
        parent: Option<usize>,
    ) -> Result<usize, TerrainError> {
        let Some(parent) = parent else {
            return Ok(0);
        };
        if parent == 0 {
            return Ok(0);
        }
        let parent = self
            .values
            .get(parent)
            .ok_or(TerrainError::InvalidParent)?
            .clone();
        if let Some(existing) = self.named(name) {
            return Ok(existing.id);
        }
        self.insert(name, parent.category, parent.flags, parent.erosion)
    }
    fn insert(
        &mut self,
        name: &str,
        category: Category,
        flags: u16,
        erosion: f32,
    ) -> Result<usize, TerrainError> {
        if name.is_empty() || name.len() > 256 || !name.is_ascii() {
            return Err(TerrainError::InvalidName);
        }
        if self.values.len() >= 65536 {
            return Err(TerrainError::RegistryLimit);
        }
        let id = self.values.len();
        self.values.push(Terrain {
            id,
            name: name.into(),
            category,
            flags,
            erosion,
        });
        Ok(id)
    }
    pub fn composite(&mut self, a: usize, b: usize) -> Result<usize, TerrainError> {
        let mut a = self
            .values
            .get(a)
            .ok_or(TerrainError::InvalidParent)?
            .clone();
        let mut b = self
            .values
            .get(b)
            .ok_or(TerrainError::InvalidParent)?
            .clone();
        if a.id == b.id {
            return Ok(a.id);
        }
        if a.id > b.id {
            std::mem::swap(&mut a, &mut b);
        }
        let name = format!("{}-{}", a.name, b.name);
        if let Some(existing) = self.named(&name) {
            return Ok(existing.id);
        }
        // Source CompositeTerrain.getDominant reads a.getCategory() twice, so
        // the lower-ID parent's other flags/category win. Preserve this quirk.
        let flags = (a.flags & !FLAT) | if a.is_flat() && b.is_flat() { FLAT } else { 0 };
        let erosion = if a.erosion <= b.erosion {
            a.erosion
        } else {
            b.erosion
        };
        self.insert(&name, a.category, flags, erosion)
    }
}
