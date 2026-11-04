mod support;
use support::{bits, cell, snapshot};
use terraforged_native_adapter::{noise_registry::NoiseRegistry, preset::CellPresetInput};
use terraforged_native_core::{
    graph::Noise as N, layout::Levels, noises as n, populator::Populator as P,
    region::RegionConfig, seed::Seed, terrain::TerrainRegistry, terrain_provider, volcano::Volcano,
};
fn floats(root: &serde_json::Value, key: &str) -> Vec<f32> {
    root[key]
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
    let noises = NoiseRegistry::from_json(&root["noiseRegistry"].to_string())?;
    let ground = N::holder(noises.get("reterraforged:terrain/ground")?)?;
    let coords = floats(&root, "coordinates");
    let ids = floats(&root, "identities");
    let heights = floats(&root, "modifyHeights");
    let masks = floats(&root, "modifyMasks");
    let mut observed = std::collections::BTreeSet::new();
    for (name, value) in &cases {
        for seed_value in root["seeds"].as_array().unwrap() {
            let preset = CellPresetInput::from_json(&value.to_string())?.kernel;
            let seed_value = seed_value.as_i64().unwrap() as i32;
            let seed = Seed::new(seed_value);
            let props = &preset.world.properties;
            let levels = Levels::new(props.terrain_scaler(), props.sea_level);
            let settings = &preset.terrain;
            let warp_seed = seed_value.wrapping_add(8934);
            let region = RegionConfig {
                seed: seed_value.wrapping_add(789124),
                scale: settings.general.terrain_region_size,
                warp_x: n::simplex(warp_seed, 400, 1)?,
                warp_z: n::simplex(warp_seed.wrapping_add(1), 400, 1)?,
                warp_strength: 200.0,
            };
            let mut registry = TerrainRegistry::new();
            let nodes = terrain_provider::generate(
                &seed,
                settings,
                &region,
                &levels,
                ground.clone(),
                &mut registry,
            )?;
            let prefix = format!("{name}/{seed_value}");
            println!("provider/{prefix}\t{},{}", seed.get(), nodes.len());
            for (index, p) in nodes.iter().enumerate() {
                let mut fields = vec![p.source_kind().into(), bits(p.weight().unwrap())];
                if let Some(t) = p.terrain_data() {
                    fields.extend([
                        t.terrain.to_string(),
                        registry.get(t.terrain).unwrap().name().into(),
                        bits(t.base_scale),
                        bits(t.height_scale),
                    ]);
                    for noise in [&t.base, &t.height, &t.erosion, &t.weirdness] {
                        fields.extend([
                            noise.kind().into(),
                            bits(noise.min_value()),
                            bits(noise.max_value()),
                        ]);
                    }
                }
                println!("node/{prefix}/{index}\t{}", fields.join(","));
                for (i, &x) in coords.iter().enumerate() {
                    for (j, &z) in coords.iter().enumerate() {
                        let mut c = cell(true);
                        p.apply(&mut c, x, z)?;
                        println!("node-cell/{prefix}/{index}/{i}/{j}\t{}", snapshot(&c));
                    }
                }
            }
            let selector = P::selector(nodes)?;
            for (id, &identity) in ids.iter().enumerate() {
                for (i, &x) in coords.iter().enumerate() {
                    for (j, &z) in coords.iter().enumerate() {
                        let mut c = cell(true);
                        c.terrain_region_id = identity;
                        selector.apply(&mut c, x, z)?;
                        println!("selector/{prefix}/{id}/{i}/{j}\t{}", snapshot(&c));
                    }
                }
            }
            let mut volcano_seed = Seed::new(seed_value);
            let volcano =
                Volcano::new(&mut volcano_seed, &region, &levels, settings.volcano.weight)?;
            println!("volcano-cursor/{prefix}\t{}", volcano_seed.get());
            let grid = &root["volcanoGrid"];
            let min = grid["min"].as_i64().unwrap() as i32;
            let max = grid["max"].as_i64().unwrap() as i32;
            let step = grid["step"].as_u64().unwrap() as usize;
            for z in (min..=max).step_by(step) {
                for x in (min..=max).step_by(step) {
                    let mut c = cell(true);
                    volcano.apply(&mut c, x as f32, z as f32);
                    observed.insert(c.terrain);
                    println!("volcano/{prefix}/{x}/{z}\t{}", snapshot(&c));
                }
            }
            for (h, &height) in heights.iter().enumerate() {
                for (m, &mask) in masks.iter().enumerate() {
                    for initial in [0, 16, 17] {
                        let mut c = cell(true);
                        c.terrain = initial;
                        c.height = height;
                        c.river_mask = mask;
                        Volcano::modify_type(&mut c, &levels);
                        println!("modify/{prefix}/{h}/{m}/{initial}\t{}", snapshot(&c));
                    }
                }
            }
            let mut c = cell(true);
            c.terrain_region_id = 0.5;
            selector.apply(&mut c, 17.0, -17.0)?;
            let expected = snapshot(&c);
            for &x in coords.iter().rev() {
                let mut query = cell(true);
                query.terrain_region_id = 0.5;
                selector.apply(&mut query, x, -x)?;
            }
            let mut c = cell(true);
            c.terrain_region_id = 0.5;
            selector.apply(&mut c, 17.0, -17.0)?;
            assert_eq!(snapshot(&c), expected);
            std::thread::scope(|s| {
                let handles: Vec<_> = (0..4)
                    .map(|_| {
                        s.spawn(|| {
                            let mut c = cell(true);
                            c.terrain_region_id = 0.5;
                            selector.apply(&mut c, 17.0, -17.0).unwrap();
                            snapshot(&c)
                        })
                    })
                    .collect();
                for h in handles {
                    assert_eq!(h.join().unwrap(), expected);
                }
            });
        }
    }
    assert!([0, 16, 17].into_iter().all(|id| observed.contains(&id)));
    eprintln!(
        "NATIVE_TERRAIN_PROVIDER_GUARDS_OK ownedRegistry originalOrder volcanoBranches replayOrder parallelReaders"
    );
    Ok(())
}
