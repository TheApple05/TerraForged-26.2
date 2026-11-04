use horizon_core::position::ChunkPos;
use horizon_world::{
    registry::BlockRegistry,
    worldgen::generator::{GeneratorData, OverworldGenerator, ProtoChunk, terraforged_world},
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
    time::Instant,
};
fn snapshot(chunk: &ProtoChunk, registry: &BlockRegistry) -> Value {
    let mut hash = Sha256::new();
    let mut states = BTreeMap::new();
    let mut solid = 0usize;
    let mut water = 0usize;
    for y in -64..320 {
        for z in 0..16 {
            for x in 0..16 {
                let state = registry
                    .state(chunk.block(x, y, z))
                    .expect("registered generated state");
                hash.update(state.name.as_bytes());
                let properties: BTreeMap<_, _> = state.properties.iter().collect();
                hash.update(serde_json::to_string(&properties).unwrap().as_bytes());
                hash.update(b"\n");
                *states.entry(state.name.clone()).or_insert(0usize) += 1;
                if state.name != "minecraft:air"
                    && state.name != "minecraft:cave_air"
                    && state.name != "minecraft:void_air"
                    && state.name != "minecraft:water"
                    && state.name != "minecraft:lava"
                {
                    solid += 1;
                }
                if state.name == "minecraft:water" {
                    water += 1;
                }
            }
        }
    }
    let pos = chunk.position();
    let mut biomes = BTreeSet::new();
    let mut heights = Vec::new();
    for z in 0..16 {
        for x in 0..16 {
            heights.push(chunk.surface_height(x, z));
        }
    }
    for y in (-64..320).step_by(4) {
        for z in 0..4 {
            for x in 0..4 {
                biomes.insert(
                    chunk
                        .biome_at(pos.x * 16 + x * 4, y, pos.z * 16 + z * 4)
                        .to_owned(),
                );
            }
        }
    }
    assert!(
        solid > 0 && solid < 16 * 16 * 384,
        "nonempty terrain and sky"
    );
    assert!(*heights.iter().min().unwrap() >= -64 && *heights.iter().max().unwrap() <= 320);
    json!({"position":[pos.x,pos.z],"blockSha256":format!("{:x}",hash.finalize()),"solidBlocks":solid,"waterBlocks":water,"blockCounts":states,"biomes":biomes,"surfaceMin":heights.iter().min(),"surfaceMax":heights.iter().max(),"decorated":chunk.is_decorated()})
}
fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 2 {
        return Err("configuration and own prepared resources required".into());
    }
    let input: Value = serde_json::from_str(&fs::read_to_string(&args[0])?)?;
    let resources = Path::new(&args[1]);
    let seed = 8675309i64;
    let registry = Box::leak(Box::new(BlockRegistry::from_facts_json(fs::File::open(
        resources.join("facts.json"),
    )?)?));
    let data = GeneratorData::load(&resources.join("worldgen"), registry)?;
    let opened = terraforged_world::open(
        data.clone(),
        seed,
        &input["preset"].to_string(),
        &input["noiseRegistry"].to_string(),
        false,
    )?;
    let native = opened.generator;
    let kernel = opened.kernel;
    assert!(kernel.upgrade().is_some());
    assert_eq!(native.level(), (-64, 384));
    assert_eq!(native.sea_level(), 63);
    let mut worker = native.worker();
    let mut chunks = Vec::new();
    let started = Instant::now();
    for (x, z) in [(0, 0), (-1, -1), (64, -64), (1024, -2048)] {
        let chunk = worker.generate_proto(ChunkPos::new(x, z))?;
        assert!(!chunk.is_decorated());
        chunks.push(snapshot(&chunk, registry));
    }
    let generation_seconds = started.elapsed().as_secs_f64();
    let baseline = OverworldGenerator::without_structures(data.clone(), seed)?;
    let mut base_worker = baseline.worker();
    let vanilla = snapshot(&base_worker.generate_proto(ChunkPos::new(0, 0))?, registry);
    assert_ne!(
        chunks[0]["blockSha256"], vanilla["blockSha256"],
        "native world must use the procedural terrain"
    );
    drop(base_worker);
    drop(baseline);
    let first = chunks[0]["blockSha256"].clone();
    native.release_caches();
    assert_eq!(kernel.upgrade().unwrap().cache_counts()?, (0, 0));
    worker.release();
    let rebuilt = snapshot(&worker.generate_proto(ChunkPos::new(0, 0))?, registry);
    assert_eq!(first, rebuilt["blockSha256"]);
    let mut neighborhood = Vec::new();
    for z in -1..=1 {
        for x in -1..=1 {
            neighborhood.push(worker.generate_proto(ChunkPos::new(x, z))?);
        }
    }
    let mut neighborhood: [ProtoChunk; 9] = neighborhood
        .try_into()
        .map_err(|_| "nine neighbors required")?;
    let decoration = worker.decorate(&mut neighborhood)?;
    assert!(neighborhood[4].is_decorated());
    let center = snapshot(&neighborhood[4], registry);
    assert!(
        kernel.upgrade().unwrap().cache_counts()?.0 > 0,
        "native samples must populate the owned tile cache"
    );
    let unsupported = native.unsupported_features();
    let census = native.cache_census();
    drop(neighborhood);
    worker.release();
    drop(worker);
    native.release_caches();
    assert_eq!(kernel.upgrade().unwrap().cache_counts()?, (0, 0));
    drop(native);
    assert!(
        kernel.upgrade().is_none(),
        "world retains native kernel after workers/generator drop"
    );
    println!(
        "{}",
        serde_json::to_string_pretty(
            &json!({"status":"passed-native-procedural-terrain-prototype","worldSeed":seed,"chunks":chunks,"vanillaOrigin":vanilla,"coldReloadOrigin":rebuilt,"decoratedOrigin":center,"decoration":{"placed":decoration.placed,"unsupported":decoration.unsupported,"failed":decoration.failed,"skipped":decoration.skipped},"unsupportedFeatures":unsupported,"hostCacheCensus":{"biomeChunks":census.biome_chunks,"biomeBytes":census.biome_bytes,"carverEntries":census.carver_entries,"startChunks":census.start_chunks,"startLists":census.start_lists},"generationSeconds":generation_seconds,"clearedCaches":true,"releasedKernelOwner":true,"notCovered":["17 native classic biome variants","Source carver settings","Independent whole-chunk Java parity","Structures/spawn/save/client/launcher/modpack acceptance"]})
        )?
    );
    eprintln!(
        "NATIVE_PROCEDURAL_CHUNKS_OK distantCoordinates realNoiseAquifersMaterialsCarvers actualDecoration coldRegeneration releasedKernelOwner"
    );
    Ok(())
}
fn main() {
    if let Err(e) = run() {
        eprintln!("{e}");
        std::process::exit(1)
    }
}
