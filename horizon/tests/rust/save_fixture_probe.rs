//! Build/read a scratch native saved stem with Horizon's actual save API.
//! This is QA setup; it does not claim the create-world UI offers the preset.
use horizon_client_lib::{
    biome::Biomes,
    save::{self, Level, WorldGen},
};
use horizon_core::position::ChunkPos;
use horizon_world::{
    DATA_VERSION,
    nbt::{Limits, Tag},
    region::RegionFile,
    registry::BlockRegistry,
    worldgen::generator::{GeneratorData, OverworldGenerator},
};
use serde_json::{Value, json};
use std::{fs, path::Path};

fn to_tag(value: &Value) -> Result<Tag, Box<dyn std::error::Error>> {
    Ok(match value {
        Value::String(value) => Tag::String(value.as_str().into()),
        Value::Object(value) => Tag::Compound(
            value
                .iter()
                .map(|(k, v)| Ok((k.as_str().into(), to_tag(v)?)))
                .collect::<Result<_, Box<dyn std::error::Error>>>()?,
        ),
        _ => return Err("QA preset requires only string/compound fields".into()),
    })
}
fn native_stem(settings: &Tag) -> Result<&Tag, Box<dyn std::error::Error>> {
    Ok(settings
        .get("dimensions")
        .and_then(|d| d.get("minecraft:overworld"))
        .ok_or("saved overworld stem missing")?)
}
fn fingerprint(names: &[String]) -> String {
    let mut hash = 14_695_981_039_346_656_037_u64;
    for name in names {
        for byte in name.bytes().chain([0]) {
            hash = (hash ^ u64::from(byte)).wrapping_mul(1_099_511_628_211);
        }
    }
    format!("{hash:016x}")
}
fn generated_biomes(
    generator: &OverworldGenerator,
    pos: ChunkPos,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut worker = generator.worker();
    let chunk = worker.generate_proto(pos)?;
    let mut names = Vec::new();
    for y in (-64..320).step_by(4) {
        for z in (0..16).step_by(4) {
            for x in (0..16).step_by(4) {
                names.push(chunk.biome_at(x, y, z).to_owned());
            }
        }
    }
    Ok(names)
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    assert_eq!(
        args.len(),
        5,
        "expected init/check, fresh QA world path, own preset JSON and own prepared resources"
    );
    let world = Path::new(&args[2]);
    if args[1] == "init" {
        if world.exists() {
            return Err("Refusing to replace an existing world".into());
        }
        let mut level = Level {
            name: "TerraForged native QA".into(),
            seed: 8675309,
            game_mode: 3,
            spawn: Some([8.5, 130., 8.5]),
            position: Some([8.5, 130., 8.5]),
            world_spawn: Some([8, 100, 8]),
            rotation: [135., 65.],
            health: 20.,
            food: 20,
            saturation: 5.,
            dimension: "minecraft:overworld".into(),
            difficulty: "peaceful".into(),
            allow_commands: Some(true),
            time: 6000,
            day_time: 6000,
            ..Level::default()
        };
        level.write(world)?;
        save::write_world_gen_settings(
            world,
            8675309,
            WorldGen {
                structures: false,
                ..WorldGen::default()
            },
        )?;
        // Explicit QA fixture setup after the normal save writer. The real
        // game must subsequently read, save and reopen this stem unchanged.
        let preset: Value = serde_json::from_str(&fs::read_to_string(&args[3])?)?;
        let mut settings =
            save::world_gen_settings(world).ok_or("normal settings were not saved")?;
        let Tag::Compound(fields) = &mut settings else {
            return Err("settings compound expected".into());
        };
        fields.insert("dimensions".into(), to_tag(&preset["dimensions"])?);
        save::write_dat(
            &world.join("data/minecraft/world_gen_settings.dat"),
            Tag::compound([("data", settings), ("DataVersion", Tag::Int(DATA_VERSION))]),
        )?;
    } else if args[1] != "check" && args[1] != "check_all" {
        return Err("expected init or check".into());
    }
    let settings = save::world_gen_settings(world)
        .ok_or("real save reader could not reload generator settings")?;
    let stem = native_stem(&settings)?;
    assert_eq!(
        stem.get("generator").and_then(|g| g.get("settings")),
        Some(&Tag::String("terraforged:overworld".into()))
    );
    assert_eq!(settings.get("seed"), Some(&Tag::Long(8675309)));
    let level = Level::read(world).ok_or("real level reader could not reload level")?;
    assert_eq!(level.seed, 8675309);
    let mut regions = Vec::new();
    let path = world.join("dimensions/minecraft/overworld/region");
    if path.is_dir() {
        for file in fs::read_dir(path)? {
            let file = file?;
            if file.path().extension().is_some_and(|x| x == "mca") {
                regions.push(file.file_name().to_string_lossy().into_owned());
            }
        }
    }
    regions.sort();
    let resources = Path::new(&args[4]);
    let registry = Box::leak(Box::new(BlockRegistry::from_facts_json(fs::File::open(
        resources.join("facts.json"),
    )?)?));
    let data = GeneratorData::load(&resources.join("worldgen"), registry)?;
    let preset: Value = serde_json::from_str(&fs::read_to_string(&args[3])?)?;
    let selected = OverworldGenerator::for_world_with_settings(
        data.clone(),
        8675309,
        "minecraft:normal",
        "minecraft:overworld",
        false,
        Some(&preset["dimensions"]["minecraft:overworld"]),
    )?;
    let vanilla_generator = OverworldGenerator::for_world_with_settings(
        data,
        8675309,
        "minecraft:normal",
        "minecraft:overworld",
        false,
        None,
    )?;
    let mut checks = Vec::new();
    for pos in [ChunkPos::new(0, 0), ChunkPos::new(1024, -2048)] {
        let native = generated_biomes(&selected, pos)?;
        let vanilla = generated_biomes(&vanilla_generator, pos)?;
        assert_ne!(
            native, vanilla,
            "chosen chunk must distinguish native generation from vanilla"
        );
        let mut stored_hash = None;
        if args[1] == "check_all" || (args[1] == "check" && pos == ChunkPos::new(0, 0)) {
            let region_pos = pos.region();
            let mut region = RegionFile::open(world.join(format!(
                "dimensions/minecraft/overworld/region/r.{}.{}.mca",
                region_pos.x, region_pos.z
            )))?;
            let root = region
                .read(pos)?
                .ok_or("target chunk was not saved")?
                .nbt(Limits::saved())?;
            let biomes = Biomes::load(resources)?;
            let chunk = save::decode(&root, registry, &biomes)
                .ok_or("target was not saved as a decodable FULL chunk")?;
            let mut stored = Vec::new();
            for y in (-64..320).step_by(4) {
                for z in (0..16).step_by(4) {
                    for x in (0..16).step_by(4) {
                        stored.push(biomes.get(chunk.biome(x, y, z)).name.clone());
                    }
                }
            }
            assert_eq!(
                stored, native,
                "game-saved biomes must match the native factory, not a vanilla fallback"
            );
            stored_hash = Some(fingerprint(&stored));
        }
        checks.push(json!({"chunk":[pos.x,pos.z],"quartBiomes":native.len(),"nativeBiomeFingerprint":fingerprint(&native),"vanillaBiomeFingerprint":fingerprint(&vanilla),"storedBiomeFingerprint":stored_hash}));
    }
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "status":"passed-native-save-fixture-reader", "action":args[1],"seed":8675309,
            "overworldSettings":"terraforged:overworld","regionFiles":regions,
            "levelName":level.name,"position":level.position,
        "qaSpawnSupplied":true,"structuresEnabled":false,
        "chunkChecks":checks,
            "notCovered":["UI world creation","Natural spawn","Structures","Full save/client acceptance without subsequent game runs"]
        }))?
    );
    Ok(())
}
