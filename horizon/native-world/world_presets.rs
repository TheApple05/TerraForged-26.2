// Native preset metadata only. Generator owners stay in the constructed world.
use super::{
    Tag,
    custom_presets::{self, CustomPreset, RegisteredPreset},
};
use std::sync::OnceLock;
fn string(value: &str) -> Tag {
    Tag::String(value.into())
}
fn overworld() -> Result<Tag, String> {
    Ok(Tag::compound([
        ("type", string("minecraft:overworld")),
        (
            "generator",
            Tag::compound([
                ("type", string("minecraft:noise")),
                ("settings", string("terraforged:overworld")),
                (
                    "biome_source",
                    Tag::compound([
                        ("type", string("minecraft:multi_noise")),
                        ("preset", string("minecraft:overworld")),
                    ]),
                ),
            ]),
        ),
    ]))
}
fn recognizes(generator: &Tag) -> bool {
    generator.get("type") == Some(&string("minecraft:noise"))
        && generator.get("settings") == Some(&string("terraforged:overworld"))
}
pub fn register() {
    static REGISTERED: OnceLock<Result<RegisteredPreset, String>> = OnceLock::new();
    if let Err(error) = REGISTERED.get_or_init(|| {
        custom_presets::register(CustomPreset {
            id: "terraforged:default",
            label: "generator.terraforged.default",
            overworld,
            recognizes,
        })
    }) {
        panic!("TerraForged preset registration failed: {error}");
    }
}
