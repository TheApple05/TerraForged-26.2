use std::{sync::Arc, time::Instant};
use terraforged_native_adapter::{noise_registry::NoiseRegistry, preset::CellPresetInput};
use terraforged_native_core::{
    cell::Cell,
    kernel_sampler::{KernelSampler, SamplerOptions},
};
fn checksum(c: &Cell) -> u64 {
    [
        c.height.to_bits(),
        c.height_erosion.to_bits(),
        c.sediment.to_bits(),
        c.gradient.to_bits(),
        c.region_moisture.to_bits(),
        c.region_temperature.to_bits(),
        c.continent_id.to_bits(),
        c.continent_edge.to_bits(),
        c.terrain_region_id.to_bits(),
        c.terrain_region_edge.to_bits(),
        c.biome_region_id.to_bits(),
        c.biome_region_edge.to_bits(),
        c.macro_biome_id.to_bits(),
        c.river_mask.to_bits(),
        c.continent_x as u32,
        c.continent_z as u32,
        c.erosion_mask as u32,
        c.terrain as u32,
        c.biome as u32,
        c.erosion.to_bits(),
        c.weirdness.to_bits(),
        c.temperature.to_bits(),
        c.moisture.to_bits(),
        c.beach_noise.to_bits(),
    ]
    .into_iter()
    .fold(0xcbf29ce484222325, |state, v| {
        (state ^ v as u64).wrapping_mul(0x100000001b3)
    })
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 3 {
        return Err("preset, registry and workload required".into());
    }
    let preset = CellPresetInput::from_json(&std::fs::read_to_string(&args[0])?)?.kernel;
    let registry = NoiseRegistry::from_json(&std::fs::read_to_string(&args[1])?)?;
    let w: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&args[2])?)?;
    let seed = w["worldSeed"].as_i64().unwrap() as i32;
    let options = SamplerOptions {
        tile_factor: w["tileFactor"].as_i64().unwrap() as i32,
        border_chunks: w["borderChunks"].as_i64().unwrap() as i32,
        cache_capacity: w["cacheCapacity"].as_u64().unwrap() as usize,
        max_in_flight: w["maxInFlight"].as_u64().unwrap() as usize,
    };
    let tiles: Vec<(i32, i32)> = w["tiles"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| (p[0].as_i64().unwrap() as i32, p[1].as_i64().unwrap() as i32))
        .collect();
    let columns = w["warmColumns"].as_u64().unwrap() as usize;
    let trials = w["trials"].as_u64().unwrap() as usize;
    if tiles.len() != 4 || trials != 3 || columns != 65536 || options.tile_factor != 3 {
        return Err("predefined benchmark workload required".into());
    }
    let mut observations = vec![];
    let mut baseline = None;
    for trial in 0..trials {
        for mode in ["serial", "parallel"] {
            let sampler = KernelSampler::new(
                &preset,
                seed,
                registry.get("reterraforged:terrain/ground")?,
                registry.get("reterraforged:climate/biome_edge_shape")?,
                options,
            )?;
            let start = Instant::now();
            let values = if mode == "serial" {
                tiles
                    .iter()
                    .map(|(x, z)| sampler.tile(*x, *z))
                    .collect::<Result<Vec<_>, _>>()?
            } else {
                std::thread::scope(|scope| {
                    let workers: Vec<_> = tiles
                        .iter()
                        .map(|(x, z)| {
                            let sampler = &sampler;
                            scope.spawn(move || sampler.tile(*x, *z))
                        })
                        .collect();
                    workers
                        .into_iter()
                        .map(|t| t.join().expect("native tile benchmark thread"))
                        .collect::<Result<Vec<_>, _>>()
                })?
            };
            let cold = start.elapsed().as_secs_f64();
            let tile_checksums: Vec<_> = values
                .iter()
                .map(|t| {
                    t.cells
                        .iter()
                        .fold(0u64, |state, c| state.rotate_left(7) ^ checksum(c))
                })
                .collect();
            let start = Instant::now();
            let mut warm_checksum = 0u64;
            for index in 0..columns {
                let (tx, tz) = tiles[index % tiles.len()];
                let local = index / tiles.len();
                let x = (tx << 7) + (local as i32 & 127);
                let z = (tz << 7) + ((local as i32 >> 7) & 127);
                warm_checksum = warm_checksum.rotate_left(7)
                    ^ checksum(&std::hint::black_box(sampler.sample(x, z)?));
            }
            let warm = start.elapsed().as_secs_f64();
            if let Some((expected, expected_warm)) = &baseline {
                assert_eq!(&tile_checksums, expected);
                assert_eq!(&warm_checksum, expected_warm);
            } else {
                baseline = Some((tile_checksums.clone(), warm_checksum));
            }
            assert!(values.iter().all(|v| Arc::strong_count(v) >= 2));
            assert_eq!(sampler.cache_counts()?.1, 0);
            observations.push(serde_json::json!({"trial":trial,"mode":mode,"coldTiles":tiles.len(),"coldSeconds":cold,
            "coreColumnsGenerated":4*128*128,"backingColumnsGenerated":4*160*160,"warmColumns":columns,"warmSecondsIncludingChecksum":warm,
            "tileChecksums":tile_checksums.iter().map(|v|format!("{v:016x}")).collect::<Vec<_>>(),"warmChecksum":format!("{warm_checksum:016x}"),
            "cachedTiles":sampler.cache_counts()?.0,"cachedRiverMaps":sampler.heightmap().river_cache_len()?}));
            sampler.release_caches()?;
            assert_eq!(sampler.cache_counts()?, (0, 0));
        }
    }
    println!(
        "{}",
        serde_json::to_string(
            &serde_json::json!({"status":"measured-native-kernel-only","workload":w,"observations":observations,"allChecksumsAgree":true,
        "scope":"Optimized procedural cell/tile kernel. Warm time includes 24-field checksum. Cold parallel time includes thread startup. No native density, chunks, game client or FPS."})
        )?
    );
    Ok(())
}
