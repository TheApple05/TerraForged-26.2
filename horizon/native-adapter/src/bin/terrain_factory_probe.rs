mod support;
use support::{bits, cell, snapshot};
use terraforged_native_core::{
    graph::{GraphError, Noise as N},
    layout::Levels,
    noise_util::round_java,
    populator::{Populator as P, TerrainPopulator as T},
    preset::TerrainSettingsEntry as Settings,
    seed::Seed,
    terrain_factories as f,
};
fn float(v: &serde_json::Value, key: &str) -> f32 {
    v[key].as_f64().unwrap() as f32
}
fn settings(v: &serde_json::Value) -> Settings {
    Settings {
        weight: float(v, "weight"),
        base_scale: float(v, "baseScale"),
        vertical_scale: float(v, "verticalScale"),
        horizontal_scale: float(v, "horizontalScale"),
    }
}
fn build(
    factory: usize,
    root: i32,
    seed: &mut Seed,
    ground: N,
    s: &Settings,
    border: &Settings,
    vertical: f32,
    levels: &Levels,
) -> Result<(P, Option<T>), GraphError> {
    let t = match factory {
        0 => f::make_steppe(seed, ground, s)?,
        1 => f::make_plains(seed, ground, s, vertical)?,
        2 => f::make_plateau(seed, ground, s, vertical)?,
        3 => f::make_hills1(seed, ground, s, vertical)?,
        4 => f::make_hills2(seed, ground, s, vertical)?,
        5 => f::make_dales(seed, ground, s)?,
        6 => f::make_badlands(seed, ground, s)?,
        7 => f::make_torridonian(seed, ground, s)?,
        8 => f::make_mountains1(seed, ground, s, vertical, false)?,
        9 => f::make_mountain_chain(seed, ground, s, vertical, false)?,
        10 => f::make_mountains2(seed, ground, s, vertical, false)?,
        11 => f::make_mountains3(seed, ground, s, vertical, false)?,
        12 => f::make_mountains1(seed, ground, s, vertical, true)?,
        13 => f::make_mountain_chain(seed, ground, s, vertical, true)?,
        14 => f::make_mountains2(seed, ground, s, vertical, true)?,
        15 => f::make_mountains3(seed, ground, s, vertical, true)?,
        16 => f::make_border(seed, ground, s, border, vertical)?,
        17 => return Ok((f::make_deep_ocean(root, levels.water)?, None)),
        18 => return Ok((f::make_shallow_ocean(levels)?, None)),
        19 => return Ok((f::make_coast(levels)?, None)),
        _ => unreachable!(),
    };
    Ok((P::terrain(t.clone())?, Some(t)))
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(
        std::env::args().nth(1).ok_or("input required")?,
    )?)?;
    let coords: Vec<f32> = root["coordinates"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_f64().unwrap() as f32)
        .collect();
    let cases: std::collections::BTreeMap<String, serde_json::Value> =
        serde_json::from_value(root["cases"].clone())?;
    for (name, config) in &cases {
        for root_seed in root["seeds"].as_array().unwrap() {
            for factory in 0..20 {
                let root_seed = root_seed.as_i64().unwrap() as i32;
                let mut seed = Seed::new(root_seed);
                let s = settings(&config["entry"]);
                let border = settings(&config["borderScaling"]);
                let vertical = float(config, "globalVerticalScale");
                let levels = Levels::new(
                    config["height"].as_i64().unwrap() as i32,
                    config["sea"].as_i64().unwrap() as i32,
                );
                let (p, terrain) = build(
                    factory,
                    root_seed,
                    &mut seed,
                    N::constant(levels.ground)?,
                    &s,
                    &border,
                    vertical,
                    &levels,
                )?;
                let prefix = format!("{name}/{root_seed}/{factory}");
                println!("cursor/{prefix}\t{}", seed.get());
                if let Some(t) = &terrain {
                    let mut fields = vec![
                        t.terrain.to_string(),
                        bits(t.base_scale),
                        bits(t.height_scale),
                        bits(t.weight),
                    ];
                    for n in [&t.base, &t.height, &t.erosion, &t.weirdness] {
                        fields.extend([n.kind().into(), bits(n.min_value()), bits(n.max_value())]);
                    }
                    println!("meta/{prefix}\t{}", fields.join(","));
                }
                for (i, &x) in coords.iter().enumerate() {
                    for (j, &z) in coords.iter().enumerate() {
                        let mut c = cell(true);
                        p.apply(&mut c, x, z)?;
                        println!("cell/{prefix}/{i}/{j}\t{}", snapshot(&c));
                        if let Some(t) = &terrain {
                            println!(
                                "noise/{prefix}/{i}/{j}\t{},{},{},{}",
                                bits(t.base.compute(x, z, 0)),
                                bits(t.height.compute(x, z, 0)),
                                bits(t.erosion.compute(x, z, 0)),
                                bits(t.weirdness.compute(x, z, 0))
                            );
                        }
                    }
                }
                let mut c = cell(true);
                p.apply(&mut c, 17.0, -17.0)?;
                let expected = snapshot(&c);
                for &x in coords.iter().rev() {
                    p.apply(&mut cell(true), x, -x)?;
                }
                p.apply(&mut c, 17.0, -17.0)?;
                assert_eq!(snapshot(&c), expected);
                std::thread::scope(|s| {
                    let handles: Vec<_> = (0..4)
                        .map(|_| {
                            s.spawn(|| {
                                let mut c = cell(true);
                                p.apply(&mut c, 17.0, -17.0).unwrap();
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
    }
    for v in root["roundInputs"].as_array().unwrap() {
        let key = v.as_str().unwrap();
        println!(
            "round/{key}\t{}",
            round_java(f32::from_bits(u32::from_str_radix(key, 16)?))
        );
    }
    eprintln!(
        "NATIVE_TERRAIN_FACTORY_GUARDS_OK originalCursors ownedGraphs replayOrder parallelReaders"
    );
    Ok(())
}
