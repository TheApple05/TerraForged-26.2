use serde_json::Value as V;
use terraforged_native_adapter::{noise_registry::NoiseRegistry, preset::CellPresetInput};
use terraforged_native_core::{
    cell::Cell,
    density_primitives::{
        CellField, ConditionalArray, LinearSpline, clamp_to_nearest_unit, noise_value,
    },
    graph::GraphError,
    layout::Levels,
    terrain::TerrainRegistry,
};
fn f(v: f32) -> String {
    if v.is_nan() {
        "NaN".into()
    } else {
        format!("{:08x}", v.to_bits())
    }
}
fn d(v: f64) -> String {
    if v.is_nan() {
        "NaN".into()
    } else {
        format!("{:016x}", v.to_bits())
    }
}
fn f32_input(v: &V) -> f32 {
    f32::from_bits(u32::from_str_radix(v.as_str().unwrap(), 16).unwrap())
}
fn f64_input(v: &V) -> f64 {
    f64::from_bits(u64::from_str_radix(v.as_str().unwrap(), 16).unwrap())
}
fn pos(v: &V) -> [i32; 3] {
    [
        v[0].as_i64().unwrap() as i32,
        v[1].as_i64().unwrap() as i32,
        v[2].as_i64().unwrap() as i32,
    ]
}
fn coordinate(base: f64, p: [i32; 3]) -> f64 {
    base + p[0] as f64 * 0.1 + p[1] as f64 * 0.01 + p[2] as f64 * -0.05
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root: V = serde_json::from_str(&std::fs::read_to_string(
        std::env::args().nth(1).ok_or("input required")?,
    )?)?;
    let terrains = TerrainRegistry::new();
    let presets: std::collections::BTreeMap<String, V> =
        serde_json::from_value(root["presets"].clone())?;
    for (name, value) in &presets {
        let p = CellPresetInput::from_json(&value.to_string())?.kernel;
        let l = Levels::new(
            p.world.properties.terrain_scaler(),
            p.world.properties.sea_level,
        );
        let cp = p.world.control_points.make();
        println!(
            "context/{name}\t{},{},{},{},{},{},{},{}",
            l.world_height,
            f(l.water),
            f(l.ground),
            f(cp.deep_ocean),
            f(cp.shallow_ocean),
            f(cp.beach),
            f(cp.coast),
            f(cp.inland)
        );
        for terrain in 0..18 {
            for h in root["heightBits"].as_array().unwrap() {
                for e in root["edgeBits"].as_array().unwrap() {
                    for (b, beach) in root["beachNoises"].as_array().unwrap().iter().enumerate() {
                        let c = Cell {
                            height: f32_input(h),
                            continent_edge: f32_input(e),
                            beach_noise: beach.as_f64().unwrap() as f32,
                            terrain,
                            erosion: -3.25,
                            weirdness: -0.0,
                            biome_region_id: 0.125,
                            temperature: 1.25,
                            moisture: -0.25,
                            gradient: 0.75,
                            height_erosion: -0.1,
                            sediment: 0.3,
                            ..Cell::default()
                        };
                        let values = CellField::ALL
                            .into_iter()
                            .map(|field| field.read(&c, l, cp, &terrains).map(f))
                            .collect::<Result<Vec<_>, _>>()?;
                        println!(
                            "field/{name}/{terrain}/{}/{}/{b}\t{}",
                            h.as_str().unwrap(),
                            e.as_str().unwrap(),
                            values.join(",")
                        );
                    }
                }
            }
        }
    }
    for resolution in root["resolutions"].as_array().unwrap() {
        let resolution = resolution.as_i64().unwrap() as i32;
        for value in root["doubleBits"].as_array().unwrap() {
            let result = d(clamp_to_nearest_unit(f64_input(value), resolution));
            println!(
                "clamp/{resolution}/{}\t{result},{result},{result}",
                value.as_str().unwrap()
            );
        }
    }
    for (index, s) in root["splineCases"].as_array().unwrap().iter().enumerate() {
        let spline = LinearSpline::new(
            s["locationBits"]
                .as_array()
                .unwrap()
                .iter()
                .map(f64_input)
                .collect(),
        )?;
        let values: Vec<_> = s["valueBits"]
            .as_array()
            .unwrap()
            .iter()
            .map(f64_input)
            .collect();
        let bounds: Vec<_> = s["boundsBits"]
            .as_array()
            .unwrap()
            .iter()
            .map(|p| (f64_input(&p[0]), f64_input(&p[1])))
            .collect();
        let (min, max) = spline.source_bounds(&bounds)?;
        for value in root["doubleBits"].as_array().unwrap() {
            for (p, position) in root["positions"].as_array().unwrap().iter().enumerate() {
                let mut trace = vec![-1];
                let position = pos(position);
                let result = spline.compute(f64_input(value), |index| {
                    trace.push(index as i32);
                    coordinate(values[index], position)
                });
                println!(
                    "spline/{index}/{}/{p}\t{},{},{},{}",
                    value.as_str().unwrap(),
                    result.map(d).unwrap_or("rejected".into()),
                    d(min),
                    d(max),
                    trace
                        .iter()
                        .map(i32::to_string)
                        .collect::<Vec<_>>()
                        .join(":")
                );
            }
        }
    }
    for (origin_index, origin) in root["cacheOrigins"].as_array().unwrap().iter().enumerate() {
        let ox = origin[0].as_i64().unwrap() as i32;
        let oz = origin[1].as_i64().unwrap() as i32;
        for width in root["cacheWidths"].as_array().unwrap() {
            let width = width.as_i64().unwrap() as i32;
            let prefix = format!("{origin_index}/{width}");
            let mut calls = vec![];
            let cache = ConditionalArray::new(ox, oz, width, |position| {
                let value = coordinate(0.125, position);
                calls.push(format!(
                    "{},{},{},{}",
                    position[0],
                    position[1],
                    position[2],
                    d(value)
                ));
                Ok(value)
            })?;
            for (index, call) in calls.iter().enumerate() {
                println!("array-init/{prefix}/{index}\t{call}");
            }
            println!("array-bounds/{prefix}\t{},{}", d(-1e9), d(1e9));
            let offsets = [-1, 0, 1, width - 1, width, width + 1];
            for (x, dx) in offsets.iter().enumerate() {
                for (z, dz) in offsets.iter().enumerate() {
                    for (y, height) in root["cacheYs"].as_array().unwrap().iter().enumerate() {
                        let position = [
                            ox.wrapping_add(*dx),
                            height.as_i64().unwrap() as i32,
                            oz.wrapping_add(*dz),
                        ];
                        let mut called = 0;
                        let value = cache.compute(position, |position| {
                            called += 1;
                            Ok(coordinate(0.125, position))
                        })?;
                        println!("array/{prefix}/{x}/{z}/{y}\t{},{}", d(value), called);
                    }
                }
            }
        }
    }
    let registry = NoiseRegistry::from_json(&root["noiseRegistry"].to_string())?;
    let names: std::collections::BTreeMap<String, V> =
        serde_json::from_value(root["noiseRegistry"].clone())?;
    for name in names.keys() {
        let noise = registry.get(name)?;
        for seed in root["noiseSeeds"].as_array().unwrap() {
            let seed = seed.as_i64().unwrap() as i32;
            for (p, position) in root["positions"].as_array().unwrap().iter().enumerate() {
                println!(
                    "noise/{name}/{seed}/{p}\t{},{},{}",
                    d(noise_value(&noise, pos(position), seed)),
                    d(noise.min_value() as f64),
                    d(noise.max_value() as f64)
                );
            }
        }
    }
    for field in CellField::ALL {
        assert_eq!(CellField::parse(field.name())?, field);
    }
    assert!(CellField::parse("HEIGHT").is_err());
    for locations in [vec![], vec![1.0, 0.0], vec![f64::NAN], vec![0.0; 257]] {
        assert!(LinearSpline::new(locations).is_err());
    }
    for width in [-1, 257] {
        assert!(ConditionalArray::new(0, 0, width, |_| Ok(0.0)).is_err());
    }
    let mut count = 0;
    assert!(matches!(
        ConditionalArray::new(0, 0, 4, |_| {
            count += 1;
            if count == 5 {
                Err(GraphError::NodeBudget)
            } else {
                Ok(0.0)
            }
        }),
        Err(GraphError::NodeBudget)
    ));
    assert_eq!(count, 5);
    let p = CellPresetInput::from_json(&presets["default"].to_string())?.kernel;
    assert!(
        CellField::Continent
            .read(
                &Cell {
                    terrain: usize::MAX,
                    ..Cell::default()
                },
                Levels::new(256, 63),
                p.world.control_points.make(),
                &terrains
            )
            .is_err()
    );
    eprintln!(
        "NATIVE_DENSITY_PRIMITIVES_OK originalFields doubleFloatNarrowing splineCallbacks nestedCacheCoordinates ownedTables explicitUnsupportedInputs noHostBinding"
    );
    Ok(())
}
