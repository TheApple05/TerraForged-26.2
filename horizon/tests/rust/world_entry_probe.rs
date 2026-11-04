use horizon_core::density::terraforged_runtime::kernel;
use horizon_core::position::ChunkPos;
use horizon_world::{
    registry::BlockRegistry,
    worldgen::{
        generator::{GeneratorData, OverworldGenerator, terraforged_registry, terraforged_world},
        overrides::WorldgenOverrides,
    },
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, fs, path::Path, sync::Arc};
fn digest(
    generator: &OverworldGenerator,
    x: i32,
    z: i32,
    registry: &BlockRegistry,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut worker = generator.worker();
    let chunk = worker.generate_proto(ChunkPos::new(x, z))?;
    let mut hash = Sha256::new();
    for y in -64..320 {
        for z in 0..16 {
            for x in 0..16 {
                let state = registry
                    .state(chunk.block(x, y, z))
                    .ok_or("registered state")?;
                let properties: BTreeMap<_, _> = state.properties.iter().collect();
                hash.update(state.name.as_bytes());
                hash.update(serde_json::to_string(&properties)?.as_bytes());
                hash.update(b"\n");
            }
        }
    }
    Ok(format!("{:x}", hash.finalize()))
}
fn open(
    data: Arc<GeneratorData>,
    preset: &str,
    saved: Option<&Value>,
) -> Result<OverworldGenerator, Box<dyn std::error::Error>> {
    Ok(OverworldGenerator::for_world_with_settings(
        data,
        8675309,
        preset,
        "minecraft:overworld",
        false,
        saved,
    )?)
}
fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 2 {
        return Err("own prepared resources and native preset JSON required".into());
    }
    let root = Path::new(&args[0]);
    let preset = fs::read_to_string(&args[1])?;
    let preset_value: Value = serde_json::from_str(&preset)?;
    let registry = Box::leak(Box::new(BlockRegistry::from_facts_json(fs::File::open(
        root.join("facts.json"),
    )?)?));
    let mut overrides = WorldgenOverrides::default();
    overrides.set("world_presets", "terraforged:default", preset);
    let data = GeneratorData::load_with(&root.join("worldgen"), registry, &overrides)?;
    assert!(terraforged_registry::active().is_none());
    let vanilla = open(data.clone(), "minecraft:normal", None)?;
    let vanilla_origin = digest(&vanilla, 0, 0, registry)?;
    drop(vanilla);
    let direct = terraforged_world::open(
        data.clone(),
        8675309,
        kernel::builtin_preset_json(),
        kernel::builtin_noises_json(),
        false,
    )?;
    let native_origin = digest(&direct.generator, 0, 0, registry)?;
    let native_far = digest(&direct.generator, 1024, -2048, registry)?;
    assert_ne!(native_origin, vanilla_origin);
    drop(direct);
    assert!(terraforged_registry::active().is_none());
    let selected = open(data.clone(), "terraforged:default", None)?;
    assert_eq!(digest(&selected, 0, 0, registry)?, native_origin);
    assert_eq!(digest(&selected, 1024, -2048, registry)?, native_far);
    drop(selected);
    let native_stem = preset_value["dimensions"]["minecraft:overworld"].clone();
    let saved_native = open(data.clone(), "minecraft:normal", Some(&native_stem))?;
    assert_eq!(digest(&saved_native, 0, 0, registry)?, native_origin);
    drop(saved_native);
    let mut vanilla_stem = native_stem.clone();
    vanilla_stem["generator"]["settings"] = json!("minecraft:overworld");
    let saved_vanilla = open(data.clone(), "terraforged:default", Some(&vanilla_stem))?;
    assert_eq!(digest(&saved_vanilla, 0, 0, registry)?, vanilla_origin);
    drop(saved_vanilla);
    for dimension in ["minecraft:the_nether", "minecraft:the_end"] {
        let generator = OverworldGenerator::for_world_with_settings(
            data.clone(),
            8675309,
            "terraforged:default",
            dimension,
            false,
            None,
        )?;
        let baseline = OverworldGenerator::for_world_with_settings(
            data.clone(),
            8675309,
            "minecraft:normal",
            dimension,
            false,
            None,
        )?;
        assert_eq!(generator.level(), baseline.level());
        assert_eq!(generator.sea_level(), baseline.sea_level());
    }
    // A dimension override takes precedence over a saved stem, as in the host.
    overrides.set("dimensions", "minecraft:overworld", native_stem.to_string());
    let dimension_data = GeneratorData::load_with(&root.join("worldgen"), registry, &overrides)?;
    let dimension_native = open(dimension_data, "minecraft:normal", Some(&vanilla_stem))?;
    assert_eq!(digest(&dimension_native, 0, 0, registry)?, native_origin);
    drop(dimension_native);
    overrides.set(
        "dimensions",
        "minecraft:overworld",
        vanilla_stem.to_string(),
    );
    let dimension_data = GeneratorData::load_with(&root.join("worldgen"), registry, &overrides)?;
    let dimension_vanilla = open(dimension_data, "terraforged:default", Some(&native_stem))?;
    assert_eq!(digest(&dimension_vanilla, 0, 0, registry)?, vanilla_origin);
    drop(dimension_vanilla);
    let vanilla_after = open(data, "minecraft:normal", None)?;
    assert_eq!(digest(&vanilla_after, 0, 0, registry)?, vanilla_origin);
    assert!(terraforged_registry::active().is_none());
    println!(
        "{}",
        serde_json::to_string_pretty(
            &json!({"status":"passed-native-world-entry-prototype","nativeOrigin":native_origin,"nativeFar":native_far,"vanillaOrigin":vanilla_origin,"selectedPresetMatchesFactory":true,"nativeSavedStemRecognized":true,"savedVanillaWinsOverUiPreset":true,"dimensionOverridePrecedence":true,"netherEndConstructorPropertiesPreserved":true,"vanillaUnchanged":true,"notCovered":["Actual save-file round trip","Player launcher installation","17 classic native biomes/features","Rendered native client"]})
        )?
    );
    eprintln!(
        "NATIVE_WORLD_ENTRY_OK actualConstructor selectedPreset savedStem dimensionPrecedence distantCoordinates vanillaPreserved netherEndConstructorProperties"
    );
    Ok(())
}
fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1)
    }
}
