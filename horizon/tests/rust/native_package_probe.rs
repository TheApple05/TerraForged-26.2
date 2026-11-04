//! Read-only acceptance of the exact prototype archive by Horizon's real readers.
use horizon_mods::{Content, Package, index::native::Bundle};
use serde_json::json;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    assert_eq!(args.len(), 4, "expected rejected original, corrected package and mismatched-version case");
    let rejected = Bundle::read(Path::new(&args[1])).err().ok_or("original unexpectedly accepted")?;
    assert!(rejected.contains("unexpected native source") || rejected.contains("identity/revision differ"));
    let native = Bundle::read(Path::new(&args[2]))?;
    let version_rejection = Bundle::read(Path::new(&args[3])).err().ok_or("mismatched version unexpectedly accepted")?;
    assert_eq!(version_rejection, "native manifest and mixin identity/revision differ");
    let package = Package::read(Path::new(&args[2]))?;
    assert_eq!(native.manifest.id, package.manifest.id);
    assert_eq!(native.manifest.version, package.manifest.version);
    assert_eq!(native.manifest.game, "26.3");
    assert!(package.wasm.is_none());
    let content = Content::read(Path::new(&args[2]), &package.manifest)?;
    assert_eq!(content.files.len(), 2);
    let preset: serde_json::Value = serde_json::from_slice(&content.file("data/terraforged/worldgen/world_preset/default.json")?)?;
    assert_eq!(preset["dimensions"]["minecraft:overworld"]["generator"]["settings"], "terraforged:overworld");
    assert_eq!(preset["dimensions"].as_object().ok_or("dimensions absent")?.len(), 3);
    let language: serde_json::Value = serde_json::from_slice(&content.file("assets/terraforged/lang/en_us.json")?)?;
    assert_eq!(language["generator.terraforged.default"], "TerraForged");
    let rust_sources = native.files.keys().filter(|n| n.ends_with(".rs")).count();
    assert!(rust_sources > 50);
    assert!(native.files.contains_key("mixins/terraforged_native/kernel/data/default-preset.json"));
    println!("{}", serde_json::to_string_pretty(&json!({
        "status": "passed-native-package-readers",
        "originalRejection": rejected,
        "mismatchedVersionRejection": version_rejection,
        "id": native.manifest.id,
        "version": native.manifest.version,
        "nativeFiles": native.files.len(),
        "rustSources": rust_sources,
        "contentFiles": content.files,
        "namespaces": content.namespaces,
        "containsWasm": false
    }))?);
    eprintln!("NATIVE_PACKAGE_READERS_OK originalRejected nativeBundle contentManifest contentFiles");
    Ok(())
}
