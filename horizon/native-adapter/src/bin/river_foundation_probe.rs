use terraforged_native_core::{
    java_random::JavaRandom,
    layout::Levels,
    preset::{LakeSettings, WetlandSettings},
    river::River,
    river_config::{LakeConfig, LakeConfigBuilder, RiverConfig, Variance, WetlandConfig},
    river_domain::GenWarp,
    river_warp::RiverWarp,
};
fn bits(f: f32) -> String {
    if f.is_nan() {
        "NaN".into()
    } else {
        format!("{:08x}", f.to_bits())
    }
}
fn pair(p: (f32, f32)) -> String {
    format!("{},{}", bits(p.0), bits(p.1))
}
fn f(v: &serde_json::Value, key: &str) -> f32 {
    v[key].as_f64().unwrap() as f32
}
fn i(v: &serde_json::Value, key: &str) -> i32 {
    v[key].as_i64().unwrap() as i32
}
fn array(v: &serde_json::Value, key: &str) -> Vec<f32> {
    v[key]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_f64().unwrap() as f32)
        .collect()
}
fn config(key: &str, c: &RiverConfig) {
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
fn geometry(key: &str, r: &River) {
    println!(
        "{key}\t{}",
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
    );
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(
        std::env::args().nth(1).ok_or("input required")?,
    )?)?;
    for h in root["heights"].as_array().unwrap() {
        for sea in root["seas"].as_array().unwrap() {
            let height = h.as_i64().unwrap() as i32;
            let water = sea.as_i64().unwrap() as i32;
            let levels = Levels::new(height, water);
            let prefix = format!("{height}/{water}");
            config(
                &format!("config-default/{prefix}"),
                &RiverConfig::builder(levels).build(),
            );
            for (index, v) in root["builderCases"].as_array().unwrap().iter().enumerate() {
                let c = RiverConfig::builder(levels)
                    .order(i(v, "order"))
                    .main(v["main"].as_bool().unwrap())
                    .bed_width(i(v, "bedWidth"))
                    .bank_width(i(v, "bankWidth"))
                    .bed_depth(i(v, "bedDepth"))
                    .bank_height(i(v, "bankMin"), i(v, "bankMax"))
                    .length(i(v, "length"))
                    .fade(f(v, "fade"))
                    .build();
                config(&format!("config/{prefix}/{index}"), &c);
                for width in root["connectBits"].as_array().unwrap() {
                    let b = width.as_str().unwrap();
                    let width = f32::from_bits(u32::from_str_radix(b, 16)?);
                    config(
                        &format!("fork/{prefix}/{index}/{b}"),
                        &c.create_fork_width(width, levels),
                    );
                }
                config(
                    &format!("fork-dimensions/{prefix}/{index}"),
                    &c.create_fork_dimensions(i32::MIN, -1, i32::MAX, levels),
                );
            }
            for (index, v) in root["lakeCases"].as_array().unwrap().iter().enumerate() {
                let s = LakeSettings {
                    chance: f(v, "chance"),
                    min_start_distance: f(v, "minStartDistance"),
                    max_start_distance: f(v, "maxStartDistance"),
                    depth: i(v, "depth"),
                    size_min: i(v, "sizeMin"),
                    size_max: i(v, "sizeMax"),
                    min_bank_height: i(v, "minBankHeight"),
                    max_bank_height: i(v, "maxBankHeight"),
                };
                let c = LakeConfig::from_settings(&s, levels);
                println!(
                    "lake/{prefix}/{index}\t{}",
                    [
                        c.bank_max,
                        c.bank_min,
                        c.chance,
                        c.depth,
                        c.distance_max,
                        c.distance_min,
                        c.size_max,
                        c.size_min,
                        c.size_range
                    ]
                    .into_iter()
                    .map(bits)
                    .collect::<Vec<_>>()
                    .join(",")
                );
            }
        }
    }
    let b = LakeConfigBuilder::new();
    println!(
        "lake-builder/default\t{}",
        [
            b.bank_max,
            b.bank_min,
            b.chance,
            b.depth,
            b.distance_max,
            b.distance_min,
            b.size_max,
            b.size_min
        ]
        .into_iter()
        .map(bits)
        .collect::<Vec<_>>()
        .join(",")
    );
    for (index, v) in root["wetlandCases"].as_array().unwrap().iter().enumerate() {
        let w = WetlandConfig::from_settings(&WetlandSettings {
            chance: f(v, "chance"),
            size_min: i(v, "sizeMin"),
            size_max: i(v, "sizeMax"),
        });
        println!(
            "wetland/{index}\t{},{},{},{},{}",
            w.skip_size,
            bits(w.length.min),
            bits(w.length.range),
            bits(w.width.min),
            bits(w.width.range)
        );
    }
    for (index, v) in root["variances"].as_array().unwrap().iter().enumerate() {
        let a = Variance::of(v[0].as_f64().unwrap() as f32, v[1].as_f64().unwrap() as f32);
        println!(
            "variance-meta/{index}\t{},{},{},{}",
            bits(a.min),
            bits(a.range),
            bits(Variance::min(a.min).range),
            bits(Variance::range(a.range).min)
        );
        for (j, value) in array(&root, "varianceValues").into_iter().enumerate() {
            println!(
                "variance/{index}/{j}\t{},{}",
                bits(a.apply(value)),
                bits(a.apply_scaled(value, 2.5))
            );
        }
        for seed in root["randomSeeds"].as_array().unwrap() {
            let seed = seed.as_i64().unwrap();
            let mut random = JavaRandom::new(seed);
            for n in 0..32 {
                println!(
                    "variance-rng/{index}/{seed}/{n}\t{},{},{}",
                    bits(a.next(&mut random)),
                    bits(a.next_scaled(&mut random, 2.5)),
                    random.next_i32()
                );
            }
        }
    }
    let mut rivers = Vec::new();
    let radii = array(&root, "radii");
    let coords = array(&root, "coordinates");
    for (g, a) in root["geometries"].as_array().unwrap().iter().enumerate() {
        let p: Vec<f32> = a
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_f64().unwrap() as f32)
            .collect();
        for &radius in &radii {
            rivers.push(River::with_radius(p[0], p[1], p[2], p[3], radius));
        }
        geometry(
            &format!("nodes/{g}"),
            &River::from_nodes((p[0] as i32, p[1] as i32), (p[2] as i32, p[3] as i32)),
        );
    }
    for (index, r) in rivers.iter().enumerate() {
        geometry(&format!("river/{index}"), r);
        for d in root["shorten"].as_array().unwrap() {
            let d = d.as_i64().unwrap() as i32;
            geometry(&format!("shorten/{index}/{d}"), &r.shorten(d));
        }
        for (x, &px) in coords.iter().enumerate() {
            for (z, &pz) in coords.iter().enumerate() {
                println!(
                    "bounds/{index}/{x}/{z}\t{},{},{}",
                    r.contains(px, pz),
                    r.overlaps_box(px, pz, px + 1.0, pz + 1.0),
                    r.overlaps_circle_bounds((px, pz), 275.0)
                );
            }
        }
        for (j, other) in rivers.iter().enumerate() {
            println!(
                "pair/{index}/{j}\t{},{}",
                r.intersects(other),
                r.overlaps(other)
            );
            for (e, extend) in array(&root, "extend").into_iter().enumerate() {
                println!(
                    "extend/{index}/{j}/{e}\t{}",
                    r.intersects_extended(other, extend)
                );
            }
        }
    }
    let mut warps = vec![
        RiverWarp::none(),
        RiverWarp::new(0, 0.0, 0.0, 0.0, 0.0),
        RiverWarp::new(12345, 0.1, 0.9, 0.001, 150.0),
        RiverWarp::new(i32::MAX, 1.0, 0.0, -0.25, -100.0),
    ];
    for seed in root["randomSeeds"].as_array().unwrap() {
        let seed = seed.as_i64().unwrap();
        let mut random = JavaRandom::new(seed);
        warps.push(RiverWarp::create_with_fade(0.1, &mut random));
        warps.push(
            warps
                .last()
                .unwrap()
                .create_child(0.2, 0.8, 0.5, &mut random),
        );
        warps.push(RiverWarp::none().create_child(0.0, 1.0, 0.0, &mut random));
        println!("warp-rng/{seed}\t{}", random.next_i32());
    }
    for (w, warp) in warps.iter().enumerate() {
        let (seed, params) = warp.parameters();
        println!(
            "warp/{w}\t{seed},{}",
            params.into_iter().map(bits).collect::<Vec<_>>().join(",")
        );
        for (index, r) in rivers.iter().enumerate() {
            for t in root["tBits"].as_array().unwrap() {
                let key = t.as_str().unwrap();
                let t = f32::from_bits(u32::from_str_radix(key, 16)?);
                let pos = r.pos(t);
                println!(
                    "warp-sample/{w}/{index}/{key}\t{},{},{},{}",
                    warp.test(t),
                    pair(pos),
                    pair(warp.offset(pos.0, pos.1, t, r)),
                    pair(r.pos_warped(t, warp))
                );
            }
        }
    }
    let empty = GenWarp::empty()?;
    for seed in root["randomSeeds"].as_array().unwrap() {
        for scale in root["genScales"].as_array().unwrap() {
            let seed = seed.as_i64().unwrap();
            let scale = scale.as_i64().unwrap() as i32;
            let gen_warp = GenWarp::new(seed as i32, scale)?;
            for (index, &x) in coords.iter().enumerate() {
                for (j, &z) in coords.iter().enumerate() {
                    let mut fields = Vec::new();
                    for d in [&gen_warp.lake, &gen_warp.river, &empty.lake, &empty.river] {
                        fields.extend([
                            bits(d.get_x(x, z, 0)),
                            bits(d.get_z(x, z, 0)),
                            bits(d.offset_x(x, z, 0)),
                            bits(d.offset_z(x, z, 0)),
                        ]);
                    }
                    println!("gen/{seed}/{scale}/{index}/{j}\t{}", fields.join(","));
                }
            }
        }
    }
    assert!(!warps[0].test(0.5));
    assert!(warps[1].test(0.5));
    eprintln!(
        "NATIVE_RIVER_FOUNDATIONS_OK sentinelIdentity exactRandomDraws wrappingConversions noCarvingOrNetworks"
    );
    Ok(())
}
