//! Developer resource transformation probe. Never part of the player's runtime.
use serde_json::{Value, json};
use std::{collections::BTreeMap, error::Error};
use terraforged_native_adapter::classic_biome_data::derive;
fn main() -> Result<(), Box<dyn Error>> {
    let path = std::env::args_os().nth(1).ok_or("expected input path")?;
    let input: Value = serde_json::from_slice(&std::fs::read(path)?)?;
    let bundle = |key: &str| -> Result<BTreeMap<String, String>, serde_json::Error> {
        serde_json::from_value(input[key].clone())
    };
    let data = derive(
        &bundle("biomes")?,
        &bundle("features")?,
        &bundle("placed_features")?,
        &serde_json::from_value(input["biome_tags"].clone())?,
        &serde_json::from_value(input["feature_tags"].clone())?,
    )?;
    println!(
        "{}",
        json!({"biomes":data.biomes,"placed_features":data.placed_features,"configured_features":data.configured_features,"biome_tags":data.biome_tags,"required_custom_algorithms":data.required_custom_algorithms})
    );
    Ok(())
}
