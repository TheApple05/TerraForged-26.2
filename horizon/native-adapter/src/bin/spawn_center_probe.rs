use serde_json::Value;
use std::{error::Error, fs};
use terraforged_native_adapter::{noise_registry::NoiseRegistry, preset::CellPresetInput, spawn};
use terraforged_native_core::heightmap::Heightmap;

fn main() -> Result<(), Box<dyn Error>> {
    let input: Value = serde_json::from_str(&fs::read_to_string(
        std::env::args_os()
            .nth(1)
            .ok_or("configuration JSON required")?,
    )?)?;
    let preset = CellPresetInput::from_json(&input["preset"].to_string())?;
    let registry = NoiseRegistry::from_json(&input["noiseRegistry"].to_string())?;
    for seed in [
        0_i64,
        1,
        -1,
        8675309,
        2147483647,
        -2147483648,
        4294967296,
        4294967297,
    ] {
        let map = Heightmap::new(
            &preset.kernel,
            seed as i32,
            registry.get("reterraforged:terrain/ground")?,
            registry.get("reterraforged:climate/biome_edge_shape")?,
        )?;
        for mode in ["CONTINENT_CENTER", "ISLANDS", "WORLD_ORIGIN"] {
            let mut configuration = preset.original.clone();
            configuration["world"]["properties"]["spawnType"] = Value::String(mode.into());
            let [x, z] = spawn::search_center(&configuration, &map)?;
            println!("{seed}/{mode}\t{x},{z}");
        }
        for invalid in [
            Value::Null,
            Value::Bool(false),
            Value::String("UNKNOWN".into()),
        ] {
            let mut configuration = preset.original.clone();
            configuration["world"]["properties"]["spawnType"] = invalid;
            assert!(spawn::search_center(&configuration, &map).is_err());
        }
    }
    eprintln!(
        "NATIVE_SPAWN_CENTERS_OK independentConfiguredCenters invalidModesRejected longSeedWrapping"
    );
    Ok(())
}
