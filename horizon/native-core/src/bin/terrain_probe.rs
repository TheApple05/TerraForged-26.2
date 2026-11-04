use terraforged_native_core::terrain::{Category, Terrain, TerrainRegistry};
fn bits(f: f32) -> String {
    format!("{:08x}", f.to_bits())
}
fn snapshot(key: &str, t: &Terrain) {
    println!(
        "{key}\t{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
        t.id(),
        t.name(),
        t.category().name(),
        bits(t.erosion_modifier()),
        bits(t.render_hue()),
        t.is_flat(),
        t.is_river(),
        t.is_shallow_ocean(),
        t.is_deep_ocean(),
        t.is_coast(),
        t.is_submerged(),
        t.is_overground(),
        t.overrides_river(),
        t.overrides_coast(),
        t.is_lake(),
        t.is_wetland(),
        t.is_mountain(),
        t.is_volcano()
    );
}
fn main() {
    let mut registry = TerrainRegistry::new();
    for terrain in registry.iter() {
        snapshot(&format!("builtin/{}", terrain.id()), terrain);
    }
    for (key, name, parent) in [
        ("plain", "qa_plain", Some(8)),
        ("case-collision", "QA_PLAIN", Some(12)),
        ("mountain", "qa_mountain", Some(12)),
        ("badlands", "qa_badlands", Some(9)),
        ("volcano", "qa_volcano", Some(16)),
        ("none", "qa_none", Some(0)),
        ("null", "qa_null", Option::None),
        ("existing-builtin", "FLATS", Some(16)),
        ("shadow", "flats-hills", Some(16)),
    ] {
        let id = registry.get_or_create(name, parent).unwrap();
        snapshot(&format!("alias/{key}"), registry.get(id).unwrap());
    }
    let count = registry.iter().count();
    for i in 0..count {
        for j in 0..count {
            let id = registry.composite(i, j).unwrap();
            snapshot(&format!("composite/{i}/{j}"), registry.get(id).unwrap());
        }
    }
    let a = registry.composite(8, 12).unwrap();
    let b = registry.composite(9, 16).unwrap();
    snapshot("nested/a", registry.get(a).unwrap());
    snapshot("nested/b", registry.get(b).unwrap());
    let mix = registry.composite(a, b).unwrap();
    snapshot("nested/mix", registry.get(mix).unwrap());
    let alias = registry
        .get_or_create("qa_inherited_mix", Some(mix))
        .unwrap();
    snapshot("nested/alias", registry.get(alias).unwrap());
    for id in [i32::MIN, -1, 0, 8, 17, 18, 100, i32::MAX] {
        snapshot(&format!("get/{id}"), registry.get_or_none(id));
    }
    for name in [
        "qa_plain",
        "QA_PLAIN",
        "FLATS-HILLS",
        "qa_inherited_mix",
        "not_present",
    ] {
        println!(
            "find/{name}\t{}",
            registry
                .named(name)
                .map_or_else(|| "missing".into(), |t| t.id().to_string())
        );
    }
    use Category::*;
    let categories = [
        None,
        DeepOcean,
        ShallowOcean,
        Coast,
        Beach,
        River,
        Lake,
        Wetland,
        Flatland,
        Lowland,
        Highland,
    ];
    for (i, x) in categories.into_iter().enumerate() {
        for (j, z) in categories.into_iter().enumerate() {
            println!("category/{i}/{j}\t{}", x.dominant(z).name());
        }
    }
    println!(
        "final-count\t{}",
        registry
            .iter()
            .find(|t| t.name() == "qa_inherited_mix")
            .unwrap()
            .id()
    );
    assert!(registry.get_or_create("qa_bad", Some(usize::MAX)).is_err());
    assert!(registry.get_or_create("", Some(8)).is_err());
    assert!(registry.get_or_create("nonascii_α", Some(8)).is_err());
    assert!(registry.composite(usize::MAX, 8).is_err());
    eprintln!("NATIVE_TERRAIN_GUARDS_OK ownedIds asciiNames invalidParents");
}
