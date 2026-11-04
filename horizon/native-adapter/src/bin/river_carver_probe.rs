mod support;
use serde_json::Value as V;
use support::{bits, snapshot};
use terraforged_native_core::{
    bounds::{Bounds, BoundsBuilder},
    cell::Cell,
    graph::CurveFunction,
    java_random::JavaRandom,
    lake::Lake,
    layout::Levels,
    noise_util,
    preset::LakeSettings,
    river::River,
    river_carver::{CarverSettings, RiverCarver},
    river_config::{LakeConfig, RiverConfig},
    river_warp::RiverWarp,
    terrain::TerrainRegistry,
    wetland::Wetland,
};
fn f(v: &V, k: &str) -> f32 {
    v[k].as_f64().unwrap() as f32
}
fn i(v: &V, k: &str) -> i32 {
    v[k].as_i64().unwrap() as i32
}
fn a(v: &V, k: usize) -> f32 {
    v[k].as_f64().unwrap() as f32
}
fn cell(v: &V) -> Cell {
    let mut c = support::cell(true);
    c.erosion_mask = false;
    c.height = f(v, "height");
    c.river_mask = f(v, "mask");
    c.terrain = i(v, "terrain") as usize;
    c.continent_edge = f(v, "edge");
    c
}
fn config(v: &V, l: Levels) -> RiverConfig {
    RiverConfig::builder(l)
        .order(i(v, "order"))
        .main(v["main"].as_bool().unwrap())
        .bed_width(i(v, "bedWidth"))
        .bank_width(i(v, "bankWidth"))
        .bed_depth(i(v, "bedDepth"))
        .bank_height(i(v, "bankMin"), i(v, "bankMax"))
        .length(i(v, "length"))
        .fade(f(v, "fade"))
        .build()
}
fn config_row(key: &str, c: RiverConfig) {
    println!(
        "{key}\t{},{},{},{},{},{},{},{},{},{}",
        c.bank_width,
        bits(c.bed_height),
        c.bed_width,
        bits(c.fade),
        c.length,
        c.length2,
        c.main,
        bits(c.max_bank_height),
        bits(c.min_bank_height),
        c.order
    );
}
fn bounds(key: &str, b: Bounds) {
    println!(
        "{key}\t{},{},{},{}",
        bits(b.min_x),
        bits(b.min_z),
        bits(b.max_x),
        bits(b.max_z)
    );
}
fn carve(
    prefix: &str,
    c: &RiverCarver,
    levels: Levels,
    root: &V,
    terrains: &TerrainRegistry,
) -> Result<(), Box<dyn std::error::Error>> {
    for value in root["tBits"].as_array().unwrap() {
        let key = value.as_str().unwrap();
        let t = f32::from_bits(u32::from_str_radix(key, 16)?);
        config_row(
            &format!("fork/{prefix}/{key}"),
            c.create_fork_config(t, levels),
        );
        for (o, offset) in root["offsets"].as_array().unwrap().iter().enumerate() {
            let distance = offset.as_f64().unwrap() as f32;
            let x = c.river.x1 + c.river.dx * t + c.river.norm_x * distance;
            let z = c.river.z1 + c.river.dz * t + c.river.norm_z * distance;
            for (v, input) in root["cellCases"].as_array().unwrap().iter().enumerate() {
                let mut cell = cell(input);
                c.carve(&mut cell, x + 3.0, z - 5.0, t + 0.01, x, z, t, terrains)?;
                println!("carve/{prefix}/{key}/{o}/{v}\t{}", snapshot(&cell));
            }
        }
    }
    Ok(())
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root: V = serde_json::from_str(&std::fs::read_to_string(
        std::env::args().nth(1).ok_or("input required")?,
    )?)?;
    let terrains = TerrainRegistry::new();
    for (l, lv) in root["levels"].as_array().unwrap().iter().enumerate() {
        let levels = Levels::new(
            lv[0].as_i64().unwrap() as i32,
            lv[1].as_i64().unwrap() as i32,
        );
        let gs = root["geometries"].as_array().unwrap();
        let cs = root["profiles"].as_array().unwrap();
        for (g, coords) in gs.iter().enumerate() {
            let river = River::new(a(coords, 0), a(coords, 1), a(coords, 2), a(coords, 3));
            for (c, profile) in cs.iter().enumerate() {
                for (s, v) in root["carverSettings"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .enumerate()
                {
                    let settings = CarverSettings {
                        valley_size: f(v, "valleySize"),
                        fade_in: f(v, "fadeIn"),
                        connecting: v["connecting"].as_bool().unwrap(),
                        valley_curve: CurveFunction::scurve(a(&v["curve"], 0), a(&v["curve"], 1)),
                    };
                    let carver = RiverCarver::new(
                        river.clone(),
                        RiverWarp::none(),
                        config(profile, levels),
                        settings,
                        levels,
                    );
                    carve(
                        &format!("explicit/{l}/{g}/{c}/{s}"),
                        &carver,
                        levels,
                        &root,
                        &terrains,
                    )?;
                }
            }
        }
        for seed in root["randomSeeds"].as_array().unwrap() {
            let seed = seed.as_i64().unwrap();
            let mut random = JavaRandom::new(seed);
            let carver = RiverCarver::create(
                0.0,
                0.0,
                1000.0,
                250.0,
                config(&cs[0], levels),
                levels,
                &mut random,
            );
            println!("create-rng/{l}/{seed}\t{}", random.next_i32());
            carve(
                &format!("random/{l}/{seed}"),
                &carver,
                levels,
                &root,
                &terrains,
            )?;
        }
        let grid = root["grid"].as_array().unwrap();
        for (c, center) in root["lakeCenters"].as_array().unwrap().iter().enumerate() {
            let cx = a(center, 0);
            let cz = a(center, 1);
            for (s, shape) in root["lakeShapes"].as_array().unwrap().iter().enumerate() {
                for (cfg, v) in root["lakeCases"].as_array().unwrap().iter().enumerate() {
                    let settings = LakeSettings {
                        chance: f(v, "chance"),
                        min_start_distance: f(v, "minStartDistance"),
                        max_start_distance: f(v, "maxStartDistance"),
                        depth: i(v, "depth"),
                        size_min: i(v, "sizeMin"),
                        size_max: i(v, "sizeMax"),
                        min_bank_height: i(v, "minBankHeight"),
                        max_bank_height: i(v, "maxBankHeight"),
                    };
                    let lake = Lake::new(
                        (cx, cz),
                        a(shape, 0),
                        a(shape, 1),
                        &LakeConfig::from_settings(&settings, levels),
                    );
                    let prefix = format!("{l}/{c}/{s}/{cfg}");
                    let mut builder = BoundsBuilder::new();
                    lake.record_bounds(&mut builder);
                    let b = builder.build();
                    bounds(&format!("lake-bounds/{prefix}"), b);
                    for (x, gx) in grid.iter().enumerate() {
                        for (z, gz) in grid.iter().enumerate() {
                            let px = cx + gx.as_f64().unwrap() as f32;
                            let pz = cz + gz.as_f64().unwrap() as f32;
                            println!(
                                "lake-geometry/{prefix}/{x}/{z}\t{},{}",
                                lake.overlaps(px, pz, 900.0),
                                b.contains(px, pz)
                            );
                            for (v, input) in
                                root["cellCases"].as_array().unwrap().iter().enumerate()
                            {
                                let mut cell = cell(input);
                                lake.apply(&mut cell, px, pz);
                                println!("lake/{prefix}/{x}/{z}/{v}\t{}", snapshot(&cell));
                            }
                        }
                    }
                }
            }
        }
        for seed in root["randomSeeds"].as_array().unwrap() {
            let seed = seed.as_i64().unwrap();
            for (g, coords) in gs.iter().enumerate() {
                for (radius, value) in root["wetlandRadii"].as_array().unwrap().iter().enumerate() {
                    let cx = a(coords, 0);
                    let cz = a(coords, 1);
                    let wet = Wetland::new(
                        seed as i32,
                        (cx, cz),
                        (a(coords, 2), a(coords, 3)),
                        value.as_f64().unwrap() as f32,
                        levels,
                    )?;
                    let prefix = format!("{l}/{seed}/{g}/{radius}");
                    let mut builder = BoundsBuilder::new();
                    wet.record_bounds(&mut builder);
                    bounds(&format!("wetland-bounds/{prefix}"), builder.build());
                    for (x, gx) in grid.iter().enumerate() {
                        for (z, gz) in grid.iter().enumerate() {
                            let px = cx + gx.as_f64().unwrap() as f32;
                            let pz = cz + gz.as_f64().unwrap() as f32;
                            for (v, input) in
                                root["cellCases"].as_array().unwrap().iter().enumerate()
                            {
                                let mut cell = cell(input);
                                wet.apply(&mut cell, px + 3.0, pz - 5.0, px, pz);
                                println!("wetland/{prefix}/{x}/{z}/{v}\t{}", snapshot(&cell));
                            }
                        }
                    }
                }
            }
        }
    }
    for seed in root["randomSeeds"].as_array().unwrap() {
        let seed = seed.as_i64().unwrap();
        let mut random = JavaRandom::new(seed);
        for index in 0..64 {
            let curve = RiverCarver::valley_type(&mut random);
            let mut values: Vec<String> = root["curveValues"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| bits(curve.apply(v.as_f64().unwrap() as f32)))
                .collect();
            values.push(random.next_i32().to_string());
            println!("valley-rng/{seed}/{index}\t{}", values.join(","));
        }
    }
    bounds("bounds-empty", BoundsBuilder::new().build());
    bounds("bounds-none", Bounds::NONE);
    let mut negative = BoundsBuilder::new();
    negative.record(-1000.0, -2000.0);
    negative.record(-5.0, -2.0);
    bounds("bounds-negative", negative.build());
    for key in root["doubleRoundBits"].as_array().unwrap() {
        let key = key.as_str().unwrap();
        println!(
            "round-double/{key}\t{}",
            noise_util::round_java_f64(f64::from_bits(u64::from_str_radix(key, 16)?))
        );
    }
    eprintln!(
        "NATIVE_RIVER_CARVERS_OK exactMutations originalBounds forkRounding randomDraws noNetworks"
    );
    Ok(())
}
