#[allow(dead_code)] // Shared QA helper also provides constructors used by other probes.
mod support;
use std::sync::Arc;
use terraforged_native_adapter::{noise_registry::NoiseRegistry, preset::CellPresetInput};
use terraforged_native_core::{
    cell::Cell,
    kernel_sampler::{KernelSampler, SamplerOptions},
    tile::TileBuffer,
};
fn named(c: &Cell, sampler: &KernelSampler) -> Result<String, Box<dyn std::error::Error>> {
    let text = support::snapshot(c);
    let mut fields: Vec<String> = text.split(',').map(str::to_owned).collect();
    fields[1] = c.biome.name().into();
    fields[20] = sampler
        .heightmap()
        .terrains
        .get(c.terrain)
        .ok_or("owned sampler terrain")?
        .name()
        .into();
    Ok(fields.join(","))
}
fn values(tile: &TileBuffer) -> Vec<String> {
    tile.cells.iter().map(support::snapshot).collect()
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Only configuration/coordinate/layout inputs. No Java expected cells/snapshots.
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 3 {
        return Err("preset, registry and queries required".into());
    }
    let preset = CellPresetInput::from_json(&std::fs::read_to_string(&args[0])?)?.kernel;
    let registry = NoiseRegistry::from_json(&std::fs::read_to_string(&args[1])?)?;
    let queries: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&args[2])?)?;
    let ground = registry.get("reterraforged:terrain/ground")?;
    let edge = registry.get("reterraforged:climate/biome_edge_shape")?;
    for seed in queries["seeds"].as_array().unwrap() {
        let seed = seed.as_i64().unwrap();
        let mut options = SamplerOptions::for_preset(&preset);
        options.cache_capacity = 2;
        options.max_in_flight = 2;
        let sampler =
            KernelSampler::new(&preset, seed as i32, ground.clone(), edge.clone(), options)?;
        for layout in queries["layouts"].as_array().unwrap() {
            let tx = layout["tileX"].as_i64().unwrap() as i32;
            let tz = layout["tileZ"].as_i64().unwrap() as i32;
            let size = layout["size"].as_i64().unwrap() as i32;
            assert_eq!(size, 16 << options.tile_factor);
            for dz in 0..size {
                for dx in 0..size {
                    let x = tx * size + dx;
                    let z = tz * size + dz;
                    let c = sampler.sample(x, z)?;
                    println!(
                        "filtered-tile/{seed}/{tx}/{tz}/{x}/{z}\t{}",
                        named(&c, &sampler)?
                    );
                }
            }
        }
        let first = sampler.tile(-1, -1)?;
        let expected = values(&first);
        let weak = Arc::downgrade(&first);
        drop(first);
        sampler.tile(0, 0)?;
        sampler.tile(2, -2)?;
        assert!(weak.upgrade().is_none());
        let reloaded = sampler.tile(-1, -1)?;
        assert_eq!(expected, values(&reloaded));
        sampler.clear_tiles()?;
        assert_eq!(sampler.cache_counts()?, (0, 0));
        let cases = std::thread::scope(|scope| {
            let workers: Vec<_> = [(-1, -1), (0, 0), (2, -2), (123, -456)]
                .into_iter()
                .map(|(x, z)| {
                    let sampler = &sampler;
                    scope.spawn(move || (x, z, sampler.tile(x, z).unwrap()))
                })
                .collect();
            workers
                .into_iter()
                .map(|t| t.join().unwrap())
                .collect::<Vec<_>>()
        });
        for (x, z, tile) in &cases {
            let mut direct = TileBuffer::new(*x, *z, options.tile_factor, options.border_chunks)?;
            direct.populate(sampler.heightmap())?;
            let filters = terraforged_native_core::world_filters::WorldFilters::new(
                &preset,
                seed as i32,
                sampler.heightmap().levels,
            )?;
            filters.apply(&mut direct, true, &sampler.heightmap().terrains)?;
            assert_eq!(values(tile), values(&direct));
            for (dx, dz) in [(0, 0), (127, 0), (0, 127), (127, 127), (63, 64)] {
                let wx = (*x << 7) + dx;
                let wz = (*z << 7) + dz;
                assert_eq!(
                    support::snapshot(&sampler.sample(wx, wz)?),
                    support::snapshot(tile.lookup(wx, wz))
                );
            }
        }
        assert!(sampler.cache_counts()?.0 <= 2);
        assert_eq!(sampler.cache_counts()?.1, 0);
        sampler.release_caches()?;
        assert_eq!(sampler.cache_counts()?, (0, 0));
        assert_eq!(sampler.heightmap().river_cache_len()?, 0);
        assert_eq!(expected, values(&reloaded));
        drop(cases);
        drop(reloaded);
    }
    for options in [
        SamplerOptions {
            tile_factor: 5,
            ..SamplerOptions::for_preset(&preset)
        },
        SamplerOptions {
            border_chunks: -1,
            ..SamplerOptions::for_preset(&preset)
        },
        SamplerOptions {
            cache_capacity: 0,
            ..SamplerOptions::for_preset(&preset)
        },
        SamplerOptions {
            max_in_flight: 0,
            ..SamplerOptions::for_preset(&preset)
        },
    ] {
        assert!(KernelSampler::new(&preset, 0, ground.clone(), edge.clone(), options).is_err());
    }
    eprintln!(
        "NATIVE_KERNEL_SAMPLER_OK goldenCoreColumns negativeAddressing evictedReload reverseOrder parallelTiles uncapturedCoordinates retainedArcs boundedCaches releasedCaches"
    );
    Ok(())
}
