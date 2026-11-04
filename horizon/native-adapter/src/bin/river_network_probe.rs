mod support;
use serde_json::Value as V;
use support::{bits, snapshot};
use terraforged_native_adapter::{noise_registry::NoiseRegistry, preset::CellPresetInput};
use terraforged_native_core::{
    bounds::BoundsBuilder, heightmap::Heightmap, java_random::JavaRandom,
    river_generator::RiverGenerator, river_network::Network, rivermap::Rivermap,
};
fn tree(prefix: &str, n: &Network, root: &V) -> String {
    let c = &n.river_carver;
    let r = &c.river;
    let cfg = &c.config;
    let mut lines = vec![format!(
        "node/{prefix}\t{},{},{}",
        n.children.len(),
        n.lakes.len(),
        n.wetlands.len()
    )];
    lines.push(format!(
        "river/{prefix}\t{}",
        [
            r.dx,
            r.dz,
            r.length,
            r.length2,
            r.max_x,
            r.max_z,
            r.min_x,
            r.min_z,
            r.ndx,
            r.ndz,
            r.norm_x,
            r.norm_z,
            r.x1,
            r.x2,
            r.z1,
            r.z2,
            r.angle()
        ]
        .into_iter()
        .map(bits)
        .collect::<Vec<_>>()
        .join(",")
    ));
    let b = n.bounds;
    lines.push(format!(
        "bounds/{prefix}\t{}",
        [b.min_x, b.min_z, b.max_x, b.max_z]
            .into_iter()
            .map(bits)
            .collect::<Vec<_>>()
            .join(",")
    ));
    lines.push(format!(
        "config/{prefix}\t{},{},{},{},{},{},{},{},{},{}",
        cfg.bank_width,
        bits(cfg.bed_height),
        cfg.bed_width,
        bits(cfg.fade),
        cfg.length,
        cfg.length2,
        cfg.main,
        bits(cfg.max_bank_height),
        bits(cfg.min_bank_height),
        cfg.order
    ));
    let (seed, parameters) = c.warp.parameters();
    lines.push(format!(
        "warp/{prefix}\t{seed},{}",
        parameters
            .into_iter()
            .map(bits)
            .collect::<Vec<_>>()
            .join(",")
    ));
    lines.push(format!(
        "curve/{prefix}\t{}",
        root["curveValues"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| bits(c.valley_curve.apply(v.as_f64().unwrap() as f32)))
            .collect::<Vec<_>>()
            .join(",")
    ));
    for (i, lake) in n.lakes.iter().enumerate() {
        let mut builder = BoundsBuilder::new();
        lake.record_bounds(&mut builder);
        let b = builder.build();
        lines.push(format!(
            "lake/{prefix}/{i}\t{}",
            [b.min_x, b.min_z, b.max_x, b.max_z]
                .into_iter()
                .map(bits)
                .collect::<Vec<_>>()
                .join(",")
        ));
    }
    for (i, wetland) in n.wetlands.iter().enumerate() {
        let mut builder = BoundsBuilder::new();
        wetland.record_bounds(&mut builder);
        let b = builder.build();
        lines.push(format!(
            "wetland/{prefix}/{i}\t{}",
            [b.min_x, b.min_z, b.max_x, b.max_z]
                .into_iter()
                .map(bits)
                .collect::<Vec<_>>()
                .join(",")
        ));
    }
    for (i, child) in n.children.iter().enumerate() {
        lines.push(tree(&format!("{prefix}/{i}"), child, root));
    }
    lines.join("\n")
}
fn sample(
    key: &str,
    map: &Rivermap,
    x: f32,
    z: f32,
    root: &V,
    heightmap: &Heightmap,
) -> Result<(), Box<dyn std::error::Error>> {
    for (i, v) in root["cellCases"].as_array().unwrap().iter().enumerate() {
        let mut c = support::cell(true);
        c.erosion_mask = false;
        c.height = v["height"].as_f64().unwrap() as f32;
        c.river_mask = v["mask"].as_f64().unwrap() as f32;
        c.terrain = v["terrain"].as_u64().unwrap() as usize;
        c.continent_edge = v["edge"].as_f64().unwrap() as f32;
        map.apply(&mut c, x, z, &heightmap.terrains)?;
        println!("map/{key}/{i}\t{}", snapshot(&c));
    }
    Ok(())
}
fn lines(
    key: &str,
    n: &Network,
    map: &Rivermap,
    root: &V,
    heightmap: &Heightmap,
) -> Result<(), Box<dyn std::error::Error>> {
    let r = &n.river_carver.river;
    for (t, value) in root["lineT"].as_array().unwrap().iter().enumerate() {
        let alpha = value.as_f64().unwrap() as f32;
        for (d, offset) in root["lineOffsets"].as_array().unwrap().iter().enumerate() {
            let distance = offset.as_f64().unwrap() as f32;
            let x = r.x1 + r.dx * alpha + r.norm_x * distance;
            let z = r.z1 + r.dz * alpha + r.norm_z * distance;
            sample(&format!("line/{key}/{t}/{d}"), map, x, z, root, heightmap)?;
        }
    }
    for (i, child) in n.children.iter().enumerate() {
        lines(&format!("{key}/{i}"), child, map, root, heightmap)?;
    }
    Ok(())
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root: V = serde_json::from_str(&std::fs::read_to_string(
        std::env::args().nth(1).ok_or("input required")?,
    )?)?;
    let cases: std::collections::BTreeMap<String, V> =
        serde_json::from_value(root["presets"].clone())?;
    let registry = NoiseRegistry::from_json(&root["noiseRegistry"].to_string())?;
    let ground = registry.get("reterraforged:terrain/ground")?;
    let edge = registry.get("reterraforged:climate/biome_edge_shape")?;
    for (name, value) in &cases {
        for seed in root["worldSeeds"].as_array().unwrap() {
            let world_seed = seed.as_i64().unwrap();
            let seed = world_seed as i32;
            let preset = CellPresetInput::from_json(&value.to_string())?.kernel;
            let map = Heightmap::new(&preset, seed, ground.clone(), edge.clone())?;
            let generator = RiverGenerator::new(map.continent.clone(), &preset, map.levels, seed)?;
            let context_key = format!("{name}/{world_seed}");
            for (center_index, query) in
                root["centerQueries"].as_array().unwrap().iter().enumerate()
            {
                let (cx, cz) = map.continent.nearest_center(
                    query[0].as_f64().unwrap() as f32,
                    query[1].as_f64().unwrap() as f32,
                )?;
                let key = format!("{context_key}/{center_index}");
                let cached = map.get_rivermap(cx, cz)?;
                println!("map-info/{key}\t{cx},{cz},{}", cached.networks.len());
                let expected = cached
                    .networks
                    .iter()
                    .enumerate()
                    .map(|(i, n)| tree(&format!("{key}/{i}"), n, &root))
                    .collect::<Vec<_>>()
                    .join("\n");
                if !expected.is_empty() {
                    println!("{expected}");
                }
                let id = (((cx as u32 as u64) << 32) | (cz as u32 as u64)) as i64;
                let mut random = JavaRandom::new(
                    id.wrapping_add(seed.wrapping_add(preset.rivers.seed_offset) as i64),
                );
                let replay = generator.generate_with_random(cx, cz, id, &mut random)?;
                let replay = replay
                    .networks
                    .iter()
                    .enumerate()
                    .map(|(i, n)| tree(&format!("{key}/{i}"), n, &root))
                    .collect::<Vec<_>>()
                    .join("\n");
                assert_eq!(
                    expected, replay,
                    "cached construction and caller-owned random path"
                );
                println!("rng/{key}\t{}", random.next_i32());
                let grid = root["grid"].as_array().unwrap();
                for (x, gx) in grid.iter().enumerate() {
                    for (z, gz) in grid.iter().enumerate() {
                        sample(
                            &format!("grid/{key}/{x}/{z}"),
                            &cached,
                            cx as f32 + gx.as_f64().unwrap() as f32,
                            cz as f32 + gz.as_f64().unwrap() as f32,
                            &root,
                            &map,
                        )?;
                    }
                }
                for (n, network) in cached.networks.iter().enumerate() {
                    lines(&format!("{key}/{n}"), network, &cached, &root, &map)?;
                }
                assert!(std::sync::Arc::ptr_eq(&cached, &map.get_rivermap(cx, cz)?));
            }
            let coords = root["fullCoordinates"].as_array().unwrap();
            let mut observations = vec![];
            for (x, vx) in coords.iter().enumerate() {
                for (z, vz) in coords.iter().enumerate() {
                    for climate in 0..2 {
                        let px = vx.as_f64().unwrap() as f32;
                        let pz = vz.as_f64().unwrap() as f32;
                        let mut c = support::cell(false);
                        map.apply(&mut c, px, pz, climate == 1)?;
                        let text = snapshot(&c);
                        println!("full/{context_key}/{x}/{z}/{climate}\t{text}");
                        observations.push((px, pz, climate == 1, text));
                    }
                }
            }
            for (x, z, climate, expected) in observations.iter().rev() {
                let mut c = support::cell(false);
                map.apply(&mut c, *x, *z, *climate)?;
                assert_eq!(expected, &snapshot(&c));
            }
            std::thread::scope(|scope| {
                for chunk in observations.chunks(observations.len().div_ceil(4)) {
                    let map = &map;
                    scope.spawn(move || {
                        for (x, z, climate, expected) in chunk {
                            let mut c = support::cell(false);
                            map.apply(&mut c, *x, *z, *climate).unwrap();
                            assert_eq!(expected, &snapshot(&c));
                        }
                    });
                }
            });
            assert!(map.river_cache_len()? <= 32);
            map.clear_river_cache()?;
            assert_eq!(map.river_cache_len()?, 0);
            assert!(
                map.apply(&mut support::cell(false), f32::NAN, 0.0, true)
                    .is_err()
            );
        }
    }
    // Invalid cache/count settings reject before allocating large construction lists.
    let mut preset = CellPresetInput::from_json(&cases["default"].to_string())?.kernel;
    let map = Heightmap::new(&preset, 0, ground.clone(), edge.clone())?;
    for count in [-1, 257] {
        preset.rivers.river_count = count;
        assert!(RiverGenerator::new(map.continent.clone(), &preset, map.levels, 0).is_err());
    }
    let generator = RiverGenerator::new(
        map.continent.clone(),
        &CellPresetInput::from_json(&cases["default"].to_string())?.kernel,
        map.levels,
        0,
    )?;
    assert!(terraforged_native_core::river_cache::RiverCache::new(generator.clone(), 0).is_err());
    assert!(terraforged_native_core::river_cache::RiverCache::new(generator, 257).is_err());
    let empty = CellPresetInput::from_json(&cases["empty"].to_string())?.kernel;
    let cache = terraforged_native_core::river_cache::RiverCache::new(
        RiverGenerator::new(map.continent.clone(), &empty, map.levels, 0)?,
        2,
    )?;
    let first = cache.get(0, 0)?;
    let weak_first = std::sync::Arc::downgrade(&first);
    let second = cache.get(1, 1)?;
    let weak_second = std::sync::Arc::downgrade(&second);
    drop(second);
    assert!(std::sync::Arc::ptr_eq(&first, &cache.get(0, 0)?));
    cache.get(2, 2)?;
    assert!(weak_second.upgrade().is_none());
    assert_eq!(cache.len()?, 2);
    cache.clear()?;
    assert_eq!(cache.len()?, 0);
    drop(first);
    assert!(weak_first.upgrade().is_none());
    let shared = std::thread::scope(|scope| {
        let readers: Vec<_> = (0..4)
            .map(|_| {
                let cache = &cache;
                scope.spawn(move || cache.get(-5, 7).unwrap())
            })
            .collect();
        readers
            .into_iter()
            .map(|reader| reader.join().unwrap())
            .collect::<Vec<_>>()
    });
    assert!(shared.iter().all(|m| std::sync::Arc::ptr_eq(m, &shared[0])));
    eprintln!(
        "NATIVE_RIVER_NETWORKS_OK originalTree randomCheckpoints cachedReplay reverseOrder parallelReaders boundedOwnership lruRelease concurrentIdentity completeUnfilteredCells"
    );
    Ok(())
}
