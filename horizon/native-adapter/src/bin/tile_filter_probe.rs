mod support;
use serde_json::Value as V;
use support::{bits, snapshot};
use terraforged_native_adapter::{noise_registry::NoiseRegistry, preset::CellPresetInput};
use terraforged_native_core::{
    heightmap::Heightmap,
    layout::{Levels, Size},
    terrain::TerrainRegistry,
    tile::TileBuffer,
    tile_erosion::Erosion,
    tile_filters_basic::{Modifier, Smoothing, Steepness},
    world_filters::WorldFilters,
};
fn i(v: &V, key: &str) -> i32 {
    v[key].as_i64().unwrap() as i32
}
fn initialize(v: &V, root: &V) -> Result<TileBuffer, Box<dyn std::error::Error>> {
    let n = i(v, "size");
    let pattern = i(v, "pattern");
    let mut tile = TileBuffer::new(
        v["origin"][0].as_i64().unwrap() as i32 >> 4,
        v["origin"][1].as_i64().unwrap() as i32 >> 4,
        0,
        0,
    )?;
    // Controlled Filterable backing, matching Java's synthetic Size.make(n,0).
    tile.size = Size::make(n, 0);
    tile.cells = vec![support::cell(true); (n * n) as usize];
    for z in 0..n {
        for x in 0..n {
            let index = (z * n + x) as usize;
            let c = &mut tile.cells[index];
            c.height = match pattern {
                0 => 0.2 + x as f32 * 0.003 + z as f32 * 0.001,
                1 => ((x * 31 + z * 17) % 257) as f32 / 256.0,
                2 => root["heights"][((x + z * 3) % 8) as usize]
                    .as_f64()
                    .unwrap() as f32,
                _ => {
                    if index % 47 == 0 {
                        f32::NAN
                    } else {
                        0.25
                    }
                }
            };
            c.terrain = ((x * 3 + z * 7) % 18) as usize;
            c.continent_edge = ((x + z) % 5) as f32 * 0.1;
            c.terrain_region_edge = ((x * 7 + z * 11) % 17) as f32 / 16.0;
            c.river_mask = root["masks"][((x + z) % 7) as usize].as_f64().unwrap() as f32;
            c.erosion_mask = index % 11 == 0;
        }
    }
    Ok(tile)
}
fn zoom(
    map: &Heightmap,
    filters: &WorldFilters,
    v: &V,
    optional: bool,
) -> Result<TileBuffer, Box<dyn std::error::Error>> {
    let mut tile = TileBuffer::new(0, 0, 0, 1)?;
    tile.populate_zoomed(
        map,
        v["center"][0].as_f64().unwrap() as f32,
        v["center"][1].as_f64().unwrap() as f32,
        v["zoom"].as_f64().unwrap() as f32,
    )?;
    filters.apply(&mut tile, optional, &map.terrains)?;
    Ok(tile)
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root: V = serde_json::from_str(&std::fs::read_to_string(
        std::env::args().nth(1).ok_or("input required")?,
    )?)?;
    let terrains = TerrainRegistry::new();
    let mut seen = std::collections::HashSet::new();
    for (case_index, v) in root["cases"].as_array().unwrap().iter().enumerate() {
        let n = i(v, "size");
        let seed = i(v, "seed");
        let lv = &v["levels"];
        let levels = Levels::new(
            lv[0].as_i64().unwrap() as i32,
            lv[1].as_i64().unwrap() as i32,
        );
        let preset = CellPresetInput::from_json(
            &root["presets"][v["preset"].as_str().unwrap()].to_string(),
        )?
        .kernel;
        let settings = &preset.filters;
        let modifier = Modifier::range(levels.ground, levels.ground(15));
        let erosion = Erosion::new(seed.wrapping_add(12768), n, settings.erosion, modifier)?;
        if seen.insert(n) {
            for index in 0..(n * n) as usize {
                let values = erosion
                    .brush(index)
                    .ok_or("missing brush")?
                    .iter()
                    .flat_map(|(node, weight)| [node.to_string(), bits(*weight)])
                    .collect::<Vec<_>>()
                    .join(",");
                println!("brush/{n}/{index}\t{values}");
            }
        }
        let initial = initialize(v, &root)?;
        for x in [-8, -1, 0, n - 1, n, n + 8] {
            for z in [-1, 0, 1, n - 1, n, n + 1] {
                println!(
                    "raw/{case_index}/{x}/{z}\t{}",
                    initial
                        .raw_index(x, z)
                        .map(|i| i.to_string())
                        .unwrap_or("absent".into())
                );
            }
        }
        for (index, c) in initial.cells.iter().take(64).enumerate() {
            let mut modifier = modifier;
            for inv in 0..3 {
                println!(
                    "modifier/{case_index}/{index}/{inv}\t{},{}",
                    bits(modifier.value(c.height)),
                    bits(modifier.modify(c, -0.125, &terrains)?)
                );
                modifier = modifier.invert();
            }
        }
        for stage in root["stages"].as_array().unwrap() {
            let name = stage.as_str().unwrap();
            let mut tile = initialize(v, &root)?;
            let smoothing = Smoothing::new(settings.smoothing, levels);
            let steepness = Steepness::new(1, 10.0, levels);
            let filters = WorldFilters::new(&preset, seed, levels)?;
            match name {
                "erosion" => {
                    erosion.apply(&mut tile, settings.erosion.droplets_per_chunk, &terrains)?
                }
                "smoothing" => {
                    smoothing.apply(&mut tile, settings.smoothing.iterations, &terrains)?
                }
                "steepness" => steepness.apply(&mut tile),
                "beach" => filters.beach(&mut tile, &terrains)?,
                "correction" => filters.corrections(&mut tile, &terrains)?,
                "optional" => filters.apply(&mut tile, true, &terrains)?,
                "required" => filters.apply(&mut tile, false, &terrains)?,
                _ => return Err("unknown stage".into()),
            }
            for (index, c) in tile.cells.iter().enumerate() {
                println!("stage/{case_index}/{name}/{index}\t{}", snapshot(c));
            }
        }
    }
    let registry = NoiseRegistry::from_json(&root["noiseRegistry"].to_string())?;
    let ground = registry.get("reterraforged:terrain/ground")?;
    let edge = registry.get("reterraforged:climate/biome_edge_shape")?;
    for (index, v) in root["zoomCases"].as_array().unwrap().iter().enumerate() {
        let preset = CellPresetInput::from_json(
            &root["presets"][v["preset"].as_str().unwrap()].to_string(),
        )?
        .kernel;
        let map = Heightmap::new(&preset, i(v, "seed"), ground.clone(), edge.clone())?;
        let filters = WorldFilters::new(&preset, i(v, "seed"), map.levels)?;
        let mut expected = vec![];
        for optional in 0..2 {
            let tile = zoom(&map, &filters, v, optional == 1)?;
            let values = tile.cells.iter().map(snapshot).collect::<Vec<_>>();
            for (c, text) in values.iter().enumerate() {
                println!("zoom/{index}/{optional}/{c}\t{text}");
            }
            expected.push(values);
        }
        for optional in (0..2).rev() {
            let tile = zoom(&map, &filters, v, optional == 1)?;
            assert_eq!(
                expected[optional],
                tile.cells.iter().map(snapshot).collect::<Vec<_>>()
            );
        }
        std::thread::scope(|scope| {
            for optional in 0..2 {
                let (map, filters, expected) = (&map, &filters, &expected);
                scope.spawn(move || {
                    let tile = zoom(map, filters, v, optional == 1).unwrap();
                    assert_eq!(
                        expected[optional],
                        tile.cells.iter().map(snapshot).collect::<Vec<_>>()
                    );
                });
            }
        });
    }
    for (factor, border) in [(-1, 0), (5, 0), (0, -1), (0, 5)] {
        assert!(TileBuffer::new(0, 0, factor, border).is_err());
    }
    let preset = CellPresetInput::from_json(&root["presets"]["default"].to_string())?.kernel;
    let levels = Levels::new(256, 63);
    for radius in [-1.0, f32::NAN, 65.0] {
        let mut p = preset.clone();
        p.filters.smoothing.smoothing_radius = radius;
        assert!(WorldFilters::new(&p, 0, levels).is_err());
    }
    assert!(Erosion::new(0, 15, preset.filters.erosion, Modifier::range(0.0, 1.0)).is_err());
    let mut tile = TileBuffer::new(0, 0, 0, 0)?;
    assert!(
        tile.populate_zoomed(&Heightmap::new(&preset, 0, ground, edge)?, 0.0, 0.0, 0.0)
            .is_err()
    );
    eprintln!(
        "NATIVE_TILE_FILTERS_OK originalBrushes flatIndexWrapping sequentialUpdates nestedInversion NaNClassification zoomOptional reverseOrder parallelTiles boundedInputs"
    );
    Ok(())
}
