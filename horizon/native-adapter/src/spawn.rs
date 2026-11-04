//! Source configured search center. Safe player-height placement belongs to the host.
use crate::noise_registry::{DecodeError, Result};
use serde_json::Value;
use terraforged_native_core::heightmap::Heightmap;

pub fn search_center(preset: &Value, heightmap: &Heightmap) -> Result<[i32; 2]> {
    match preset["world"]["properties"]["spawnType"].as_str() {
        Some("CONTINENT_CENTER") => {
            let (x, z) = heightmap.continent.nearest_center(0.0, 0.0)?;
            Ok([x, z])
        }
        // ISLANDS is also origin in the Java source; its TODO is not a new search.
        Some("WORLD_ORIGIN" | "ISLANDS") => Ok([0, 0]),
        _ => Err(DecodeError(
            "Missing or unsupported TerraForged world.properties.spawnType".into(),
        )),
    }
}
