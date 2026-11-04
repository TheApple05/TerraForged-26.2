mod support;
use terraforged_native_adapter::{noise_registry::NoiseRegistry, preset::CellPresetInput};
use terraforged_native_core::{cell::Cell, heightmap::Heightmap};
fn output(
    stage: &str,
    seed: i64,
    x: i32,
    z: i32,
    c: &Cell,
    map: &Heightmap,
) -> Result<(), Box<dyn std::error::Error>> {
    let text = support::snapshot(c);
    let mut fields: Vec<String> = text.split(',').map(str::to_owned).collect();
    fields[1] = c.biome.name().into();
    fields[20] = map
        .terrains
        .get(c.terrain)
        .ok_or("unknown owned terrain")?
        .name()
        .into();
    println!("{stage}/{seed}/{x}/{z}\t{}", fields.join(","));
    Ok(())
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // This binary reads configuration/coordinate inputs only, never expected cells.
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 3 {
        return Err("preset, registry and queries required".into());
    }
    let preset = CellPresetInput::from_json(&std::fs::read_to_string(&args[0])?)?.kernel;
    let registry = NoiseRegistry::from_json(&std::fs::read_to_string(&args[1])?)?;
    let query: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&args[2])?)?;
    for seed in query["seeds"].as_array().unwrap() {
        let seed = seed.as_i64().unwrap();
        let map = Heightmap::new(
            &preset,
            seed as i32,
            registry.get("reterraforged:terrain/ground")?,
            registry.get("reterraforged:climate/biome_edge_shape")?,
        )?;
        for point in query["points"].as_array().unwrap() {
            let x = point[0].as_i64().unwrap() as i32;
            let z = point[1].as_i64().unwrap() as i32;
            let mut c = support::cell(false);
            map.apply_terrain(&mut c, x as f32, z as f32)?;
            output("terrain", seed, x, z, &c, &map)?;
            let rivers = map.get_rivermap(c.continent_x, c.continent_z)?;
            map.apply_rivers(&mut c, x as f32, z as f32, &rivers)?;
            output("rivers", seed, x, z, &c, &map)?;
            let mut yes = c;
            map.apply_climate(&mut yes, x as f32, z as f32, true)?;
            output("climate-true", seed, x, z, &yes, &map)?;
            let mut no = c;
            map.apply_climate(&mut no, x as f32, z as f32, false)?;
            output("climate-false", seed, x, z, &no, &map)?;
        }
    }
    Ok(())
}
