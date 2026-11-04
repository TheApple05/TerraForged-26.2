//! Native production generator integration probe: all snapshot blocks and quart biomes.
use horizon_core::position::ChunkPos;
use horizon_world::{
    registry::BlockRegistry,
    worldgen::{
        generator::{BiomeSearch, GeneratorData, OverworldGenerator},
        overrides::WorldgenOverrides,
    },
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, fs, path::Path};

fn digest_state(hash: &mut Sha256, state: &horizon_world::chunk::BlockState) {
    let properties: BTreeMap<&str, &str> = state
        .properties
        .iter()
        .map(|(k, v)| (k.as_str(), v.as_str()))
        .collect();
    hash.update(state.name.as_bytes());
    hash.update(b":");
    hash.update(serde_json::to_string(&properties).unwrap().as_bytes());
    hash.update(b"\n");
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 4 {
        return Err("prepared_probe RESOURCES SNAPSHOT BIOME_DIRECTORY".into());
    }
    let resources = Path::new(&args[1]);
    let snapshot = Path::new(&args[2]);
    let biome_dir = Path::new(&args[3]);
    let manifest: Value =
        serde_json::from_slice(&fs::read(snapshot.join("terraforged-prepared.json"))?)?;
    let seed = manifest["seed"].as_i64().ok_or("seed")?;
    let registry = Box::leak(Box::new(BlockRegistry::from_facts_json(fs::File::open(
        resources.join("facts.json"),
    )?)?));
    let mut overrides = WorldgenOverrides::default();
    let mut imported_biomes = Vec::new();
    if biome_dir.exists() {
        for entry in fs::read_dir(biome_dir)? {
            let entry = entry?;
            if entry.path().extension().is_some_and(|v| v == "json") {
                let id = format!(
                    "reterraforged:{}",
                    entry.path().file_stem().unwrap().to_str().unwrap()
                );
                overrides.set("biomes", &id, fs::read_to_string(entry.path())?);
                imported_biomes.push(id);
            }
        }
    }
    let data = GeneratorData::load_with(&resources.join("worldgen"), registry, &overrides)?;
    // The subprocess environment chooses the prepared source. This is the real
    // OverworldGenerator constructor call that the native bundle hooks.
    let generator = OverworldGenerator::new(data.clone(), seed)?;
    assert!(
        generator
            .unsupported_features()
            .iter()
            .any(|v| v.0 == "terraforged:prepared"),
        "actual constructor injection did not install the prepared pipeline"
    );
    assert!(
        OverworldGenerator::new(data.clone(), seed.wrapping_add(1)).is_err(),
        "wrong seed accepted"
    );
    // Other dimensions keep their real pipelines even when the mod is enabled.
    let nether = OverworldGenerator::for_dimension(
        data.clone(),
        seed,
        "minecraft:normal",
        "minecraft:the_nether",
    )?;
    assert!(
        !nether
            .unsupported_features()
            .iter()
            .any(|v| v.0 == "terraforged:prepared")
    );
    let end =
        OverworldGenerator::for_dimension(data, seed, "minecraft:normal", "minecraft:the_end")?;
    assert!(
        !end.unsupported_features()
            .iter()
            .any(|v| v.0 == "terraforged:prepared")
    );
    let mut worker = generator.worker();
    let mut chunks = BTreeMap::new();
    let (min, height) = generator.level();
    let spawn = generator.spawn_position()?;
    let bounded_biomes = worker.noise_biomes(&[
        [spawn[0] >> 2, min >> 2, spawn[1] >> 2],
        [spawn[0] >> 2, (min + height - 1) >> 2, spawn[1] >> 2],
        [spawn[0] >> 2, i32::MIN, spawn[1] >> 2],
        [spawn[0] >> 2, i32::MAX, spawn[1] >> 2],
    ]);
    assert_eq!(bounded_biomes[0], bounded_biomes[2]);
    assert_eq!(bounded_biomes[1], bounded_biomes[3]);
    let source_biome = worker.noise_biomes(&[[spawn[0] >> 2, (min + 1) >> 2, spawn[1] >> 2]])[0]
        .clone()
        .ok_or("spawn biome")?;
    assert!(
        worker
            .find_closest_biome(&BiomeSearch {
                allowed: &|name| name == source_biome,
                origin: [spawn[0], min + 1, spawn[1]],
                radius: 0,
                horizontal: 4,
                vertical: 4,
                min_y: min,
                max_y: min + height - 1
            })
            .is_some()
    );
    for key in manifest["chunks"].as_object().ok_or("chunks")?.keys() {
        let (x, z) = key.split_once(',').ok_or("coordinate")?;
        let (cx, cz) = (x.parse::<i32>()?, z.parse::<i32>()?);
        let proto = worker.generate_proto(ChunkPos::new(cx, cz))?;
        assert!(
            proto.is_decorated(),
            "completed Java features must not decorate twice"
        );
        let (mut block_hash, mut biome_hash) = (Sha256::new(), Sha256::new());
        for y in min..min + height {
            for z in 0..16 {
                for x in 0..16 {
                    let state = registry
                        .state(proto.block(x, y, z))
                        .ok_or("unknown generated state")?;
                    digest_state(&mut block_hash, state);
                }
            }
        }
        let mut quarts = Vec::new();
        for y in min >> 2..(min + height) >> 2 {
            for z in 0..4 {
                for x in 0..4 {
                    let name = proto.biome_at(cx * 16 + x * 4, y * 4, cz * 16 + z * 4);
                    biome_hash.update(name.as_bytes());
                    biome_hash.update(b"\n");
                    quarts.push([cx * 4 + x, y, cz * 4 + z]);
                }
            }
        }
        let noise = worker.noise_biomes(&quarts);
        let mut index = 0;
        for y in min >> 2..(min + height) >> 2 {
            for z in 0..4 {
                for x in 0..4 {
                    assert_eq!(
                        noise[index].as_deref(),
                        Some(proto.biome_at(cx * 16 + x * 4, y * 4, cz * 16 + z * 4))
                    );
                    index += 1;
                }
            }
        }
        for z in 0..16 {
            for x in 0..16 {
                let literal_top = (min..min + height)
                    .rev()
                    .find(|&y| !proto.data().block_states().get(proto.block(x, y, z)).air)
                    .map_or(min, |y| y + 1);
                assert_eq!(
                    proto.surface_height(x, z),
                    literal_top,
                    "native heightmap differs from dense blocks"
                );
                let wx = cx * 16 + x;
                let wz = cz * 16 + z;
                let (_, spans) = worker.surface_column(wx, wz);
                let lod = worker.lod_grid(wx, wz, 1, 1, false)?;
                assert_eq!(
                    spans, lod[0].1,
                    "LOD material spans differ from full column"
                );
                for &(from, to, state) in &spans {
                    for y in from..to {
                        assert_eq!(proto.block(x, y, z), state);
                    }
                }
            }
        }
        chunks.insert(key.clone(), json!({"blockSha256": format!("{:x}", block_hash.finalize()),
            "quartBiomeSha256": format!("{:x}", biome_hash.finalize()), "blockCount": height as usize * 256,
            "quartBiomeCount": height as usize * 4, "surfaceColumnsChecked": 256}));
    }
    // Coverage failure is explicit. Never generate a different landscape outside the snapshot.
    assert!(
        worker
            .generate_proto(ChunkPos::new(1_000_000, 1_000_000))
            .is_err()
    );
    assert!(
        worker
            .lod_grid(16_000_000, 16_000_000, 1, 1, false)
            .is_err()
    );
    let before = generator.cache_census();
    generator.release_caches();
    assert_eq!(generator.cache_census().biome_chunks, 0);
    let first = manifest["chunks"]
        .as_object()
        .unwrap()
        .keys()
        .next()
        .unwrap();
    let (cx, cz) = first.split_once(',').unwrap();
    let reloaded = worker.generate_proto(ChunkPos::new(cx.parse()?, cz.parse()?))?;
    let mut reload_hash = Sha256::new();
    for y in min..min + height {
        for z in 0..16 {
            for x in 0..16 {
                digest_state(
                    &mut reload_hash,
                    registry.state(reloaded.block(x, y, z)).unwrap(),
                );
            }
        }
    }
    assert_eq!(
        format!("{:x}", reload_hash.finalize()),
        chunks[first]["blockSha256"].as_str().unwrap()
    );
    println!(
        "{}",
        serde_json::to_string_pretty(
            &json!({"schema": 1, "status": "passed-native-terrain-probe", "seed": seed,
        "constructorInjectionObserved": true, "wrongSeedRejected": true, "netherAndEndPreserved": true,
        "missingCoverageRejected": true, "biomeLocateInsideCoverage": true, "cacheCleared": true,
        "extremeQuartYClampsWithoutOverflow": true, "snapshotDiskReloadMatches": true, "cacheBytesBeforeClear": before.biome_bytes,
        "importedBiomeIds": imported_biomes, "chunks": chunks,
        "notTested": ["Rendered Horizon client", "Horizon level.dat save/reload", "Entities and block entities", "Live infinite native terrain generation"]})
        )?
    );
    Ok(())
}
