//! Deliberate 26.2 revival policy from ClassicBiomeSource, not recovered Engine code.
//! Adapted from this project's MIT Java implementation; see licenses/TerraForged-MIT.txt.
use crate::{
    cell::Cell,
    terrain::{Category, Terrain},
};

pub const VARIANTS: [&str; 17] = [
    "reterraforged:bryce",
    "reterraforged:cold_steppe",
    "reterraforged:cold_marshland",
    "reterraforged:fir_forest",
    "reterraforged:flower_plains",
    "reterraforged:frozen_lake",
    "reterraforged:frozen_marsh",
    "reterraforged:lake",
    "reterraforged:marshland",
    "reterraforged:savanna_scrub",
    "reterraforged:shattered_savanna_scrub",
    "reterraforged:snowy_fir_forest",
    "reterraforged:snowy_taiga_scrub",
    "reterraforged:steppe",
    "reterraforged:stone_forest",
    "reterraforged:taiga_scrub",
    "reterraforged:warm_beach",
];

pub fn region_choice(region_id: f32) -> f32 {
    if !region_id.is_finite() {
        return 1.0;
    }
    let mut hash = region_id.to_bits() ^ 0x632b_e59b;
    hash ^= hash >> 16;
    hash = hash.wrapping_mul(0x7feb_352d);
    hash ^= hash >> 15;
    hash = hash.wrapping_mul(0x846c_a68b);
    hash ^= hash >> 16;
    (hash >> 8) as f32 * (1.0 / 16_777_216.0)
}

pub fn eligible(parent: &str) -> bool {
    matches!(
        parent,
        "minecraft:badlands"
            | "minecraft:eroded_badlands"
            | "minecraft:old_growth_spruce_taiga"
            | "minecraft:taiga"
            | "minecraft:snowy_taiga"
            | "minecraft:flower_forest"
            | "minecraft:swamp"
            | "minecraft:savanna"
            | "minecraft:savanna_plateau"
            | "minecraft:windswept_savanna"
            | "minecraft:jungle"
            | "minecraft:plains"
            | "minecraft:sunflower_plains"
            | "minecraft:snowy_plains"
            | "minecraft:beach"
            | "minecraft:snowy_beach"
            | "minecraft:river"
            | "minecraft:frozen_river"
    )
}

/// Parent selection and its accurate cell lookup happen before this policy.
/// The wrapper retains parent cave biomes for quart Y < 0.
pub fn select<'a>(parent: &'a str, cell: &Cell, terrain: &Terrain) -> &'a str {
    if !eligible(parent)
        || !cell.biome_region_id.is_finite()
        || !cell.region_temperature.is_finite()
    {
        return parent;
    }
    let region = region_choice(cell.biome_region_id);
    let temperature = cell.region_temperature;
    // Source uses exact TerrainType.LAKE identity, not the inherited LAKE flag.
    if terrain.id() == 6 {
        return if temperature < 0.25 {
            VARIANTS[5]
        } else {
            VARIANTS[7]
        };
    }
    if terrain.is_wetland() {
        return if temperature < 0.2 {
            VARIANTS[6]
        } else if temperature < 0.4 {
            VARIANTS[2]
        } else {
            VARIANTS[8]
        };
    }
    if terrain.category() == Category::Beach && temperature > 0.75 {
        return VARIANTS[16];
    }
    match parent {
        "minecraft:badlands" | "minecraft:eroded_badlands" if region < 0.35 => VARIANTS[0],
        "minecraft:old_growth_spruce_taiga" if region < 0.35 => VARIANTS[1],
        "minecraft:taiga" if region < 0.25 => VARIANTS[3],
        "minecraft:taiga" if region < 0.55 => VARIANTS[15],
        "minecraft:snowy_taiga" if region < 0.25 => VARIANTS[11],
        "minecraft:snowy_taiga" if region < 0.55 => VARIANTS[12],
        "minecraft:flower_forest" if region < 0.35 => VARIANTS[4],
        "minecraft:swamp" if region < 0.35 => VARIANTS[8],
        "minecraft:savanna" if region < 0.35 => VARIANTS[9],
        "minecraft:savanna_plateau" if region < 0.35 => VARIANTS[13],
        "minecraft:windswept_savanna" if region < 0.35 => VARIANTS[10],
        "minecraft:jungle" if region < 1.0 / 6.0 => VARIANTS[14],
        _ => parent,
    }
}
