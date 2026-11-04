use terraforged_native_adapter::preset::CellPresetInput;
use terraforged_native_core::{
    cell::Cell,
    continent::{AdvancedContinent, bisector_distance_sq, distance_to_line_sq},
    noise_util as u,
    seed::Seed,
};
const SEEDS: [i32; 6] = [0, -1, i32::MIN, i32::MAX, 8675309, 2147483645];
const COORDS: [f32; 23] = [
    -30_000_000.0,
    -60_000.0,
    -16_384.0,
    -12_000.0,
    -4096.0,
    -3000.0,
    -1024.0,
    -257.0,
    -17.0,
    -1.0,
    -0.0,
    0.0,
    0.25,
    1.0,
    17.0,
    257.0,
    1024.0,
    3000.0,
    4096.0,
    12_000.0,
    16_384.0,
    60_000.0,
    30_000_000.0,
];
fn bits(f: f32) -> String {
    if f.is_nan() {
        "NaN".into()
    } else {
        format!("{:08x}", f.to_bits())
    }
}
fn snapshot(c: &Cell) -> String {
    [
        bits(c.beach_noise),
        (c.biome as u8).to_string(),
        bits(c.biome_region_edge),
        bits(c.biome_region_id),
        bits(c.continent_edge),
        bits(c.continent_id),
        c.continent_x.to_string(),
        c.continent_z.to_string(),
        bits(c.erosion),
        c.erosion_mask.to_string(),
        bits(c.gradient),
        bits(c.height),
        bits(c.height_erosion),
        bits(c.macro_biome_id),
        bits(c.moisture),
        bits(c.region_moisture),
        bits(c.region_temperature),
        bits(c.river_mask),
        bits(c.sediment),
        bits(c.temperature),
        c.terrain.to_string(),
        bits(c.terrain_region_edge),
        bits(c.terrain_region_id),
        bits(c.weirdness),
    ]
    .join(",")
}
fn cell(dirty: bool) -> Cell {
    if !dirty {
        return Cell::default();
    }
    Cell {
        height: -3.25,
        height_erosion: -3.25,
        sediment: -3.25,
        gradient: -3.25,
        region_moisture: -3.25,
        region_temperature: -3.25,
        continent_id: -3.25,
        continent_edge: -3.25,
        terrain_region_id: -3.25,
        terrain_region_edge: -3.25,
        biome_region_id: -3.25,
        biome_region_edge: -3.25,
        macro_biome_id: -3.25,
        river_mask: -3.25,
        continent_x: -123456,
        continent_z: -123456,
        erosion_mask: true,
        erosion: -3.25,
        weirdness: -3.25,
        temperature: -3.25,
        moisture: -3.25,
        beach_noise: -3.25,
        ..Cell::default()
    }
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let text = std::fs::read_to_string(std::env::args().nth(1).ok_or("input required")?)?;
    let cases: std::collections::BTreeMap<String, serde_json::Value> = serde_json::from_str(&text)?;
    for (name, value) in &cases {
        for root in SEEDS {
            let preset = CellPresetInput::from_json(&value.to_string())?;
            let mut seed = Seed::new(root);
            let continent = AdvancedContinent::new(&mut seed, &preset.kernel.world)?;
            let prefix = format!("{name}/{root}");
            println!("cursor/{prefix}\t{}", seed.get());
            for (i, x) in COORDS.into_iter().enumerate() {
                for (j, z) in COORDS.into_iter().enumerate() {
                    for dirty in 0..2 {
                        let mut c = cell(dirty == 1);
                        continent.apply(&mut c, x, z)?;
                        println!("cell/{prefix}/{i}/{j}/{dirty}\t{}", snapshot(&c));
                    }
                }
            }
            for point in 0..3 {
                let (x, z) = match point {
                    0 => (0.0, 0.0),
                    1 => (-60000.0, 60000.0),
                    _ => (60000.0, -60000.0),
                };
                let (cx, cz) = continent.nearest_center(x, z)?;
                println!(
                    "query/{prefix}/{point}\t{cx},{cz},{}",
                    bits(continent.edge_value(x, z)?)
                );
                for dir in 0..8 {
                    let angle = dir as f32 * (u::PI2 / 8.0);
                    let dx = u::sin(angle);
                    let dz = u::cos(angle);
                    println!(
                        "search/{prefix}/{point}/{dir}\t{},{}",
                        bits(continent.distance_to_edge(cx, cz, dx, dz)?),
                        bits(continent.distance_to_ocean(cx, cz, dx, dz)?)
                    );
                }
            }
            // Query order and immutable shared ownership must not change results.
            let forward: Vec<_> = COORDS
                .into_iter()
                .map(|x| continent.edge_value(x, -x).map(bits))
                .collect::<Result<_, _>>()?;
            for (i, x) in COORDS.into_iter().enumerate().rev() {
                assert_eq!(forward[i], bits(continent.edge_value(x, -x)?));
            }
            std::thread::scope(|s| {
                let handles: Vec<_> = (0..4)
                    .map(|_| {
                        s.spawn(|| {
                            COORDS
                                .into_iter()
                                .map(|x| continent.edge_value(x, -x).map(bits))
                                .collect::<Result<Vec<_>, _>>()
                                .unwrap()
                        })
                    })
                    .collect();
                for h in handles {
                    assert_eq!(h.join().unwrap(), forward);
                }
            });
        }
    }
    let values = [-100.0, -1.0, -0.0, 0.0, 0.25, 1.0, 100.0];
    for (i, x) in values.into_iter().enumerate() {
        for (j, z) in values.into_iter().enumerate() {
            println!(
                "geometry/{i}/{j}\t{},{}",
                bits(bisector_distance_sq(x, z, -1.0, 2.0, 3.0, -4.0)),
                bits(distance_to_line_sq(x, z, -1.0, 2.0, 3.0, -4.0))
            );
        }
    }
    let p = CellPresetInput::from_json(&cases.values().next().unwrap().to_string())?;
    let c = AdvancedContinent::new(&mut Seed::new(0), &p.kernel.world)?;
    for x in [f32::NAN, f32::INFINITY, f32::MAX] {
        assert!(c.apply(&mut Cell::default(), x, 0.0).is_err());
    }
    let mut bad = p.kernel.world.clone();
    bad.continent.continent_scale = 0;
    assert!(AdvancedContinent::new(&mut Seed::new(0), &bad).is_err());
    eprintln!("NATIVE_CONTINENT_GUARDS_OK finiteQueries scaleBounds queryOrder parallelOwnership");
    Ok(())
}
