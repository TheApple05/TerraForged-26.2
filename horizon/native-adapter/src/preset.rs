//! Cell-kernel settings only. Other preset sections remain retained, unconsumed data.
use crate::noise_registry::{DecodeError, Result, distance, field_string, float, get, int, json};
use serde_json::Value;
use terraforged_native_core::preset::*;
#[derive(Clone, Debug)]
pub struct CellPresetInput {
    pub kernel: KernelPreset,
    pub original: Value,
}
fn fail(message: impl Into<String>) -> DecodeError {
    DecodeError(message.into())
}
fn boolean(value: &Value, key: &str) -> Result<bool> {
    get(value, key)?
        .as_bool()
        .ok_or_else(|| fail(format!("Expected bool field {key}")))
}
fn optional_float(value: &Value, key: &str, default: f32) -> Result<f32> {
    if value.get(key).is_none() {
        Ok(default)
    } else {
        float(value, key)
    }
}
fn optional_int(value: &Value, key: &str, default: i32) -> Result<i32> {
    if value.get(key).is_none() {
        Ok(default)
    } else {
        int(value, key)
    }
}
impl CellPresetInput {
    pub fn from_json(text: &str) -> Result<Self> {
        let original = json(text)?;
        let kernel = KernelPreset {
            world: world(get(&original, "world")?)?,
            terrain: terrain(get(&original, "terrain")?)?,
            climate: climate(get(&original, "climate")?)?,
            rivers: rivers(get(&original, "rivers")?)?,
            filters: filters(get(&original, "filters")?)?,
        };
        Ok(Self { kernel, original })
    }
}
fn world(value: &Value) -> Result<WorldSettings> {
    let c = get(value, "continent")?;
    let p = get(value, "controlPoints")?;
    let w = get(value, "properties")?;
    let continent_type = match field_string(c, "continentType")? {
        "MULTI" => ContinentType::Multi,
        "SINGLE" => ContinentType::Single,
        "MULTI_IMPROVED" => ContinentType::MultiImproved,
        "EXPERIMENTAL" => ContinentType::Experimental,
        "INFINITE" => ContinentType::Infinite,
        other => return Err(fail(format!("Unknown continent type {other}"))),
    };
    let continent_shape = distance(
        c.get("continentShape")
            .map_or(Ok("EUCLIDEAN"), |_| field_string(c, "continentShape"))?,
    )?;
    let spawn = field_string(w, "spawnType")?;
    if !matches!(spawn, "CONTINENT_CENTER" | "ISLANDS" | "WORLD_ORIGIN") {
        return Err(fail(format!("Unknown spawn type {spawn}")));
    }
    Ok(WorldSettings {
        continent: ContinentSettings {
            continent_type,
            continent_shape,
            continent_scale: int(c, "continentScale")?,
            continent_jitter: float(c, "continentJitter")?,
            continent_skipping: optional_float(c, "continentSkipping", 0.25)?,
            continent_size_variance: optional_float(c, "continentSizeVariance", 0.25)?,
            continent_noise_octaves: optional_int(c, "continentNoiseOctaves", 5)?,
            continent_noise_gain: optional_float(c, "continentNoiseGain", 0.26)?,
            continent_noise_lacunarity: optional_float(c, "continentNoiseLacunarity", 4.33)?,
        },
        control_points: ControlPointsSettings {
            mushroom_fields_inland: optional_float(p, "mushroomFieldsInland", 0.0)?,
            mushroom_fields_coast: optional_float(p, "mushroomFieldsCoast", 0.1)?,
            deep_ocean: float(p, "deepOcean")?,
            shallow_ocean: float(p, "shallowOcean")?,
            beach: float(p, "beach")?,
            coast: float(p, "coast")?,
            inland: float(p, "inland")?,
        },
        properties: WorldProperties {
            spawn_type: spawn.into(),
            world_height: int(w, "worldHeight")?,
            world_depth: optional_int(w, "worldDepth", 64)?,
            sea_level: int(w, "seaLevel")?,
            lava_level: optional_int(w, "lavaLevel", -54)?,
        },
    })
}
fn terrain_entry(value: &Value) -> Result<TerrainSettingsEntry> {
    Ok(TerrainSettingsEntry {
        weight: float(value, "weight")?,
        base_scale: float(value, "baseScale")?,
        vertical_scale: float(value, "verticalScale")?,
        horizontal_scale: float(value, "horizontalScale")?,
    })
}
fn terrain(value: &Value) -> Result<TerrainSettings> {
    let g = get(value, "general")?;
    Ok(TerrainSettings {
        general: GeneralTerrain {
            terrain_seed_offset: int(g, "terrainSeedOffset")?,
            terrain_region_size: int(g, "terrainRegionSize")?,
            global_vertical_scale: float(g, "globalVerticalScale")?,
            global_horizontal_scale: float(g, "globalHorizontalScale")?,
            fancy_mountains: boolean(g, "fancyMountains")?,
        },
        steppe: terrain_entry(get(value, "steppe")?)?,
        plains: terrain_entry(get(value, "plains")?)?,
        hills: terrain_entry(get(value, "hills")?)?,
        dales: terrain_entry(get(value, "dales")?)?,
        plateau: terrain_entry(get(value, "plateau")?)?,
        badlands: terrain_entry(get(value, "badlands")?)?,
        torridonian: terrain_entry(get(value, "torridonian")?)?,
        mountains: terrain_entry(get(value, "mountains")?)?,
        volcano: terrain_entry(get(value, "volcano")?)?,
    })
}
fn range(value: &Value) -> Result<ClimateRange> {
    Ok(ClimateRange {
        seed_offset: int(value, "seedOffset")?,
        scale: int(value, "scale")?,
        falloff: int(value, "falloff")?,
        min: float(value, "min")?,
        max: float(value, "max")?,
        bias: float(value, "bias")?,
    })
}
fn climate(value: &Value) -> Result<ClimateSettings> {
    let b = get(value, "biomeShape")?;
    let e = get(value, "biomeEdgeShape")?;
    let r#type = match field_string(e, "type")? {
        "BILLOW" => EdgeType::Billow,
        "CELL" => EdgeType::Voronoi,
        "CELL_EDGE" => EdgeType::VoronoiEdge,
        "CONST" => EdgeType::Constant,
        "CUBIC" => EdgeType::Cubic,
        "PERLIN" => EdgeType::Perlin,
        "PERLIN2" => EdgeType::Perlin2,
        "RIDGE" => EdgeType::PerlinRidge,
        "SIMPLEX" => EdgeType::Simplex,
        "SIMPLEX2" => EdgeType::Simplex2,
        "SIMPLEX_RIDGE" => EdgeType::SimplexRidge,
        "SIN" => EdgeType::Sin,
        "RAND" => EdgeType::White,
        other => return Err(fail(format!("Unknown biome edge type {other}"))),
    };
    Ok(ClimateSettings {
        temperature: range(get(value, "temperature")?)?,
        moisture: range(get(value, "moisture")?)?,
        biome_shape: BiomeShape {
            biome_size: int(b, "biomeSize")?,
            macro_noise_size: int(b, "macroNoiseSize")?,
            biome_warp_scale: int(b, "biomeWarpScale")?,
            biome_warp_strength: int(b, "biomeWarpStrength")?,
        },
        biome_edge_shape: BiomeNoise {
            r#type,
            scale: int(e, "scale")?,
            octaves: int(e, "octaves")?,
            gain: float(e, "gain")?,
            lacunarity: float(e, "lacunarity")?,
            strength: int(e, "strength")?,
        },
    })
}
fn river_profile(value: &Value) -> Result<RiverProfile> {
    Ok(RiverProfile {
        bed_depth: int(value, "bedDepth")?,
        min_bank_height: int(value, "minBankHeight")?,
        max_bank_height: int(value, "maxBankHeight")?,
        bank_width: int(value, "bankWidth")?,
        bed_width: int(value, "bedWidth")?,
        fade: float(value, "fade")?,
    })
}
fn rivers(value: &Value) -> Result<RiverSettings> {
    let l = get(value, "lakes")?;
    let w = get(value, "wetlands")?;
    Ok(RiverSettings {
        seed_offset: int(value, "seedOffset")?,
        river_count: int(value, "riverCount")?,
        main_rivers: river_profile(get(value, "mainRivers")?)?,
        branch_rivers: river_profile(get(value, "branchRivers")?)?,
        lakes: LakeSettings {
            chance: float(l, "chance")?,
            min_start_distance: float(l, "minStartDistance")?,
            max_start_distance: float(l, "maxStartDistance")?,
            depth: int(l, "depth")?,
            size_min: int(l, "sizeMin")?,
            size_max: int(l, "sizeMax")?,
            min_bank_height: int(l, "minBankHeight")?,
            max_bank_height: int(l, "maxBankHeight")?,
        },
        wetlands: WetlandSettings {
            chance: float(w, "chance")?,
            size_min: int(w, "sizeMin")?,
            size_max: int(w, "sizeMax")?,
        },
    })
}
fn filters(value: &Value) -> Result<FilterSettings> {
    let e = get(value, "erosion")?;
    let s = get(value, "smoothing")?;
    Ok(FilterSettings {
        erosion: ErosionSettings {
            droplets_per_chunk: int(e, "dropletsPerChunk")?,
            droplet_lifetime: int(e, "dropletLifetime")?,
            droplet_volume: float(e, "dropletVolume")?,
            droplet_velocity: float(e, "dropletVelocity")?,
            erosion_rate: float(e, "erosionRate")?,
            deposite_rate: float(e, "depositeRate")?,
        },
        smoothing: SmoothingSettings {
            iterations: int(s, "iterations")?,
            smoothing_radius: float(s, "smoothingRadius")?,
            smoothing_rate: float(s, "smoothingRate")?,
        },
    })
}
