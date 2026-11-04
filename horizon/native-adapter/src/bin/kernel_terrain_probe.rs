mod support;
use terraforged_native_adapter::{noise_registry::NoiseRegistry, preset::CellPresetInput};
use terraforged_native_core::heightmap::Heightmap;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Only configuration and coordinate inputs are read. This binary never opens
    // the Java expected-cell captures or a prepared terrain/snapshot package.
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 3 {
        return Err("preset, noise registry and query JSON required".into());
    }
    let preset = CellPresetInput::from_json(&std::fs::read_to_string(&args[0])?)?.kernel;
    let registry = NoiseRegistry::from_json(&std::fs::read_to_string(&args[1])?)?;
    let query: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&args[2])?)?;
    for seed in query["seeds"].as_array().unwrap() {
        let world_seed = seed.as_i64().unwrap();
        let map = Heightmap::new(
            &preset,
            world_seed as i32,
            registry.get("reterraforged:terrain/ground")?,
            registry.get("reterraforged:climate/biome_edge_shape")?,
        )?;
        for point in query["points"].as_array().unwrap() {
            let x = point[0].as_i64().unwrap() as i32;
            let z = point[1].as_i64().unwrap() as i32;
            let mut c = support::cell(false);
            map.apply_terrain(&mut c, x as f32, z as f32)?;
            let text = support::snapshot(&c);
            let mut fields: Vec<String> = text.split(',').map(str::to_owned).collect();
            fields[1] = c.biome.name().into();
            fields[20] = map
                .terrains
                .get(c.terrain)
                .ok_or("unknown owned terrain")?
                .name()
                .into();
            println!("terrain/{world_seed}/{x}/{z}\t{}", fields.join(","));
        }
    }
    Ok(())
}
