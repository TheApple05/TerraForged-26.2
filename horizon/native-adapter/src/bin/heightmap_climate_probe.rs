mod support;
use support::{cell, snapshot};
use terraforged_native_adapter::{noise_registry::NoiseRegistry, preset::CellPresetInput};
use terraforged_native_core::{heightmap::Heightmap, preset::ContinentType};
fn floats(v: &serde_json::Value, key: &str) -> Vec<f32> {
    v[key]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_f64().unwrap() as f32)
        .collect()
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(
        std::env::args().nth(1).ok_or("input required")?,
    )?)?;
    let cases: std::collections::BTreeMap<String, serde_json::Value> =
        serde_json::from_value(root["presets"].clone())?;
    let registry = NoiseRegistry::from_json(&root["noiseRegistry"].to_string())?;
    let ground = registry.get("reterraforged:terrain/ground")?;
    let edge = registry.get("reterraforged:climate/biome_edge_shape")?;
    let coords = floats(&root, "coordinates");
    let synthetic = &root["climateInputs"];
    let heights = floats(synthetic, "heights");
    let masks = floats(synthetic, "riverMasks");
    let macros = floats(synthetic, "macroBiomeIds");
    for (name, value) in &cases {
        for world_seed in root["worldSeeds"].as_array().unwrap() {
            let world_seed = world_seed.as_i64().unwrap();
            let seed = world_seed as i32;
            let preset = CellPresetInput::from_json(&value.to_string())?.kernel;
            let map = Heightmap::new(&preset, seed, ground.clone(), edge.clone())?;
            let prefix = format!("{name}/{world_seed}");
            println!(
                "context/{prefix}\t{seed},{},{}",
                map.seed_cursor_after, map.climate.random_seed
            );
            for (i, &x) in coords.iter().enumerate() {
                for (j, &z) in coords.iter().enumerate() {
                    for dirty in 0..2 {
                        let mut terrain = cell(dirty == 1);
                        map.apply_terrain(&mut terrain, x, z)?;
                        let key = format!("{prefix}/{i}/{j}/{dirty}");
                        println!("terrain/{key}\t{}", snapshot(&terrain));
                        let mut yes = terrain;
                        map.apply_climate(&mut yes, x, z, true)?;
                        println!("climate-true/{key}\t{}", snapshot(&yes));
                        let mut no = terrain;
                        map.apply_climate(&mut no, x, z, false)?;
                        println!("climate-false/{key}\t{}", snapshot(&no));
                    }
                }
            }
            for (t, terrain) in synthetic["terrainIds"]
                .as_array()
                .unwrap()
                .iter()
                .enumerate()
            {
                for (h, &height) in heights.iter().enumerate() {
                    for (r, &mask) in masks.iter().enumerate() {
                        for (b, &macro_id) in macros.iter().enumerate() {
                            for (p, pos) in synthetic["positions"]
                                .as_array()
                                .unwrap()
                                .iter()
                                .enumerate()
                            {
                                for enabled in 0..2 {
                                    let mut c = cell(true);
                                    c.terrain = terrain.as_u64().unwrap() as usize;
                                    c.height = height;
                                    c.river_mask = mask;
                                    c.macro_biome_id = macro_id;
                                    map.apply_climate(
                                        &mut c,
                                        pos[0].as_f64().unwrap() as f32,
                                        pos[1].as_f64().unwrap() as f32,
                                        enabled == 1,
                                    )?;
                                    println!(
                                        "synthetic/{prefix}/{t}/{h}/{r}/{b}/{p}/{enabled}\t{}",
                                        snapshot(&c)
                                    );
                                }
                            }
                        }
                    }
                }
            }
            let sample = || {
                let mut c = cell(false);
                map.apply_terrain(&mut c, 17.0, -17.0).unwrap();
                map.apply_climate(&mut c, 17.0, -17.0, true).unwrap();
                snapshot(&c)
            };
            let expected = sample();
            for &x in coords.iter().rev() {
                let mut c = cell(false);
                map.apply_terrain(&mut c, x, -x)?;
                map.apply_climate(&mut c, x, -x, true)?;
            }
            assert_eq!(sample(), expected);
            std::thread::scope(|s| {
                let handles: Vec<_> = (0..4).map(|_| s.spawn(&sample)).collect();
                for h in handles {
                    assert_eq!(h.join().unwrap(), expected);
                }
            });
            let mut c = cell(false);
            assert!(map.apply_terrain(&mut c, f32::NAN, 0.0).is_err());
            let mut c = cell(false);
            c.terrain = usize::MAX;
            assert!(map.apply_climate(&mut c, 0.0, 0.0, false).is_err());
        }
    }
    let mut p = CellPresetInput::from_json(&cases.values().next().unwrap().to_string())?.kernel;
    p.world.continent.continent_type = ContinentType::Single;
    assert!(Heightmap::new(&p, 0, ground.clone(), edge.clone()).is_err());
    p.world.continent.continent_type = ContinentType::MultiImproved;
    p.terrain.general.global_horizontal_scale = 0.0;
    assert!(Heightmap::new(&p, 0, ground.clone(), edge.clone()).is_err());
    p.terrain.general.global_horizontal_scale = 1.0;
    p.terrain.steppe.horizontal_scale = 0.0059999996;
    assert!(Heightmap::new(&p, 0, ground, edge).is_err());
    eprintln!(
        "NATIVE_HEIGHTMAP_CLIMATE_GUARDS_OK supportedContinent derivedScales finiteQueries ownedTerrain reverseOrder parallelReaders noRiverExecution"
    );
    Ok(())
}
