//! Runtime biome resources derived from installed parent definitions.
//! Modern adaptation of TerraForged BiomeBuilders (c) 2021 TerraForged, MIT.
//! The policy and additions come from this project's MIT ClassicBiomes.java.
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
type Bundle = BTreeMap<String, String>;

pub struct ClassicBiomeData {
    pub biomes: Bundle,
    pub placed_features: Bundle,
    pub configured_features: Bundle,
    pub biome_tags: BTreeMap<String, Vec<String>>,
    /// These algorithms must be supplied by the native decoration integration.
    pub required_custom_algorithms: [&'static str; 2],
}
#[derive(Clone, Copy)]
struct Spec {
    id: &'static str,
    parent: &'static str,
    tag_parent: &'static str,
    remove: u8,
    add: u8,
    climate: [Option<f32>; 2],
}
const fn v(
    id: &'static str,
    parent: &'static str,
    tag_parent: &'static str,
    remove: u8,
    add: u8,
    climate: [Option<f32>; 2],
) -> Spec {
    Spec {
        id,
        parent,
        tag_parent,
        remove,
        add,
        climate,
    }
}
// remove: tree=1, forest rock=2, mushrooms=4; add: dead bush=1, grass=2, ferns=4.
const SPECS: [Spec; 17] = [
    v("bryce", "badlands", "badlands", 1, 1, [None, None]),
    v(
        "cold_steppe",
        "old_growth_spruce_taiga",
        "old_growth_spruce_taiga",
        7,
        7,
        [Some(0.2), Some(0.025)],
    ),
    v(
        "cold_marshland",
        "snowy_taiga",
        "snowy_taiga",
        1,
        7,
        [Some(0.2), Some(0.5)],
    ),
    v("fir_forest", "taiga", "taiga", 0, 7, [None, None]),
    v(
        "flower_plains",
        "flower_forest",
        "flower_forest",
        0,
        0,
        [None, None],
    ),
    v(
        "frozen_lake",
        "frozen_river",
        "frozen_river",
        1,
        0,
        [None, None],
    ),
    v(
        "frozen_marsh",
        "snowy_taiga",
        "snowy_taiga",
        1,
        0,
        [Some(0.14999), Some(0.15)],
    ),
    v("lake", "river", "river", 1, 0, [None, None]),
    v("marshland", "swamp", "swamp", 1, 7, [Some(0.7), None]),
    v("savanna_scrub", "savanna", "savanna", 1, 3, [None, None]),
    v(
        "shattered_savanna_scrub",
        "windswept_savanna",
        "windswept_savanna",
        0,
        3,
        [None, None],
    ),
    v(
        "snowy_fir_forest",
        "snowy_taiga",
        "snowy_taiga",
        0,
        4,
        [None, None],
    ),
    v(
        "snowy_taiga_scrub",
        "snowy_taiga",
        "snowy_taiga",
        1,
        4,
        [None, None],
    ),
    v(
        "steppe",
        "old_growth_spruce_taiga",
        "windswept_savanna",
        7,
        3,
        [Some(1.2), Some(0.05)],
    ),
    v("stone_forest", "jungle", "jungle", 0, 0, [None, None]),
    v("taiga_scrub", "taiga", "taiga", 5, 7, [None, None]),
    v("warm_beach", "warm_ocean", "beach", 1, 0, [Some(1.0), None]),
];
const ALIASES: [(&str, &str); 5] = [
    ("classic_dead_bush", "patch_dead_bush_2"),
    ("classic_grass_plain", "patch_grass_plain"),
    ("classic_grass_normal", "patch_grass_normal"),
    ("classic_grass_savanna", "patch_grass_savanna"),
    ("classic_ferns", "patch_large_fern"),
];
fn parse(bundle: &Bundle, name: &str) -> Result<Value, String> {
    serde_json::from_str(
        bundle
            .get(name)
            .ok_or_else(|| format!("Missing installed parent resource {name}"))?,
    )
    .map_err(|e| format!("{name}: {e}"))
}

pub fn derive(
    parents: &Bundle,
    configured: &Bundle,
    placed: &Bundle,
    resolved_tags: &BTreeMap<String, Vec<String>>,
    feature_tags: &BTreeMap<String, Vec<String>>,
) -> Result<ClassicBiomeData, String> {
    let mut result = ClassicBiomeData {
        biomes: Bundle::new(),
        placed_features: Bundle::new(),
        configured_features: Bundle::new(),
        biome_tags: BTreeMap::new(),
        required_custom_algorithms: [
            "reterraforged:classic_pillar",
            "reterraforged:swamp_surface",
        ],
    };
    for (alias, source) in ALIASES {
        let mut value = parse(placed, &format!("minecraft:{source}"))?;
        value["placement"]
            .as_array_mut()
            .ok_or("Alias placement list missing")?
            .push(json!({"type":"minecraft:count", "count":1}));
        result
            .placed_features
            .insert(format!("reterraforged:{alias}"), value.to_string());
    }
    for (id, kind, config, placement) in [
        (
            "bryce_surface",
            "classic_pillar",
            json!({"stone_forest":false}),
            json!([]),
        ),
        (
            "stone_forest_surface",
            "classic_pillar",
            json!({"stone_forest":true}),
            json!([]),
        ),
        (
            "classic_marsh_surface",
            "swamp_surface",
            json!({
                "clay_material":{"id":"minecraft:clay"},
                "gravel_material":{"id":"minecraft:gravel"},
                "dirt_material":{"id":"minecraft:dirt"}
            }),
            json!([{"type":"minecraft:count", "count":1}]),
        ),
    ] {
        let name = format!("reterraforged:{id}");
        result.configured_features.insert(name.clone(), {
            let mut definition = config;
            definition["type"] = json!(format!("reterraforged:{kind}"));
            definition.to_string()
        });
        result.placed_features.insert(
            name.clone(),
            json!({"feature":name,"placement":placement}).to_string(),
        );
    }
    let mut classifier = TreeClassifier {
        configured,
        placed,
        feature_tags,
        memo: BTreeMap::new(),
        active: BTreeSet::new(),
        remaining: 1_000_000,
    };
    for spec in SPECS {
        let name = format!("reterraforged:{}", spec.id);
        let mut biome = parse(parents, &format!("minecraft:{}", spec.parent))?;
        let steps = biome["features"]
            .as_array_mut()
            .ok_or("Parent features absent")?;
        if steps.len() != 11 {
            return Err("Classic biome definitions require the 26.3 eleven-step schema".into());
        }
        for step in steps.iter_mut() {
            let features = step.as_array().ok_or("Parent feature step is not a list")?;
            let mut retained = Vec::new();
            for feature in features {
                let id = feature.as_str().unwrap_or("");
                let path = id.split_once(':').map_or(id, |(_, path)| path);
                let old_modifier = matches!(
                    id,
                    "reterraforged:erode"
                        | "reterraforged:decorate_snow"
                        | "reterraforged:swamp_surface"
                );
                let rock = spec.remove & 2 != 0 && path == "forest_rock";
                let mushroom = spec.remove & 4 != 0
                    && matches!(
                        path,
                        "brown_mushroom_taiga"
                            | "red_mushroom_taiga"
                            | "brown_mushroom_old_growth"
                            | "red_mushroom_old_growth"
                    );
                if old_modifier || rock || mushroom {
                    continue;
                }
                if !(spec.remove & 1 != 0 && classifier.placed(feature, 0)?) {
                    retained.push(feature.clone());
                }
            }
            *step = Value::Array(retained);
        }
        let plants = steps[9].as_array_mut().ok_or("Vegetation step absent")?;
        let mut aliases = Vec::new();
        if spec.add & 1 != 0 {
            aliases.push("classic_dead_bush");
        }
        if spec.add & 2 != 0 {
            aliases.extend([
                "classic_grass_plain",
                "classic_grass_normal",
                "classic_grass_savanna",
            ]);
        }
        if spec.add & 4 != 0 {
            aliases.push("classic_ferns");
        }
        plants.extend(
            aliases
                .into_iter()
                .map(|name| json!(format!("reterraforged:{name}"))),
        );
        let raw = steps[0]
            .as_array_mut()
            .ok_or("Raw generation step absent")?;
        match spec.id {
            "bryce" => raw.push(json!("reterraforged:bryce_surface")),
            "stone_forest" => raw.push(json!("reterraforged:stone_forest_surface")),
            "marshland" => raw.push(json!("reterraforged:classic_marsh_surface")),
            _ => {}
        }
        if let Some(value) = spec.climate[0] {
            biome["temperature"] = json!(value);
        }
        if let Some(value) = spec.climate[1] {
            biome["downfall"] = json!(value);
        }
        if spec.id == "steppe" {
            let ambience = parse(parents, "minecraft:savanna")?;
            biome["attributes"] = ambience["attributes"].clone();
            biome["effects"] = ambience["effects"].clone();
            biome["has_precipitation"] = json!(false);
        }
        for (tag, members) in resolved_tags {
            if members.contains(&format!("minecraft:{}", spec.tag_parent)) {
                result
                    .biome_tags
                    .entry(tag.clone())
                    .or_default()
                    .push(name.clone());
            }
        }
        result
            .biome_tags
            .entry("reterraforged:classic_variants".into())
            .or_default()
            .push(name.clone());
        result.biomes.insert(name, biome.to_string());
    }
    Ok(result)
}

struct TreeClassifier<'a> {
    configured: &'a Bundle,
    placed: &'a Bundle,
    feature_tags: &'a BTreeMap<String, Vec<String>>,
    memo: BTreeMap<(bool, String), bool>,
    active: BTreeSet<(bool, String)>,
    remaining: usize,
}
impl TreeClassifier<'_> {
    fn named(&mut self, name: &str, placed: bool, depth: usize) -> Result<bool, String> {
        self.visit(depth)?;
        let (tag, id) = name
            .strip_prefix('#')
            .map_or((false, name), |id| (true, id));
        let id = if id.contains(':') {
            id.to_owned()
        } else {
            format!("minecraft:{id}")
        };
        let name = if tag { format!("#{id}") } else { id };
        let key = (placed, name.clone());
        if let Some(value) = self.memo.get(&key) {
            return Ok(*value);
        }
        if !self.active.insert(key.clone()) {
            return Err(format!("Cyclic parent feature {name}"));
        }
        let tree = if let Some(tag) = name.strip_prefix('#') {
            if !placed {
                return Err(format!("Configured feature is a placed-feature tag {name}"));
            }
            let entries = self
                .feature_tags
                .get(tag)
                .ok_or_else(|| format!("Missing placed-feature tag {name}"))?
                .clone();
            let mut tree = false;
            // Validate every member, even if an earlier member is a tree.
            for entry in entries {
                tree |= self.named(&entry, true, depth + 1)?;
            }
            tree
        } else {
            let value = parse(if placed { self.placed } else { self.configured }, &name)?;
            if placed {
                self.placed(&value, depth)?
            } else {
                self.configured(&value, depth)?
            }
        };
        self.active.remove(&key);
        self.memo.insert(key, tree);
        Ok(tree)
    }
    fn visit(&mut self, depth: usize) -> Result<(), String> {
        if depth > 128 || self.remaining == 0 {
            return Err("Parent feature traversal budget exceeded".into());
        }
        self.remaining -= 1;
        Ok(())
    }
    fn placed(&mut self, value: &Value, depth: usize) -> Result<bool, String> {
        self.visit(depth)?;
        if let Some(name) = value.as_str() {
            return self.named(name, true, depth + 1);
        }
        if value.get("type").is_some() {
            return self.configured(value, depth + 1);
        }
        self.configured(
            value
                .get("feature")
                .ok_or("Parent placed feature has no feature")?,
            depth + 1,
        )
    }
    fn configured(&mut self, value: &Value, depth: usize) -> Result<bool, String> {
        self.visit(depth)?;
        if let Some(name) = value.as_str() {
            return self.named(name, false, depth + 1);
        }
        if !value["type"].is_string() {
            return Err("Configured parent feature has no type".into());
        }
        let config = value.get("config").unwrap_or(value);
        if value["type"] == "minecraft:tree" {
            return Ok(true);
        }
        if value["type"] == "reterraforged:template"
            && config["templates"].as_array().is_some_and(|templates| {
                !templates.is_empty()
                    && templates.iter().all(|id| {
                        id.as_str().is_some_and(|s| {
                            s.split_once(':')
                                .map_or(s, |(_, p)| p)
                                .starts_with("structures/trees/")
                        })
                    })
            })
        {
            return Ok(true);
        }
        self.children_fields(config, depth + 1)
    }
    fn children_fields(&mut self, value: &Value, depth: usize) -> Result<bool, String> {
        self.visit(depth)?;
        let mut found = false;
        for key in [
            "feature",
            "features",
            "default",
            "vegetation_feature",
            "true_feature",
            "false_feature",
            "entries",
            "data",
        ] {
            if let Some(child) = value.get(key) {
                found |= self.children(child, depth + 1)?;
            }
        }
        Ok(found)
    }
    fn children(&mut self, value: &Value, depth: usize) -> Result<bool, String> {
        self.visit(depth)?;
        match value {
            Value::Array(values) => {
                let mut found = false;
                for value in values {
                    found |= self.children(value, depth + 1)?;
                }
                return Ok(found);
            }
            Value::String(_) => return self.placed(value, depth + 1),
            Value::Object(object) => {
                if object.contains_key("type")
                    || (object.contains_key("feature") && object.contains_key("placement"))
                {
                    return self.placed(value, depth + 1);
                }
                return self.children_fields(value, depth + 1);
            }
            _ => {}
        }
        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixtures() -> (Bundle, Bundle, Bundle, BTreeMap<String, Vec<String>>) {
        let parents = ["badlands", "old_growth_spruce_taiga", "snowy_taiga", "taiga", "flower_forest", "frozen_river", "river", "swamp", "savanna", "windswept_savanna", "jungle", "warm_ocean"].into_iter().map(|name| {
            let mut steps = vec![json!([]); 11];
            steps[9] = json!(["example:selector", "minecraft:forest_rock", "minecraft:brown_mushroom_taiga", "example:plant"]);
            (format!("minecraft:{name}"), json!({"temperature":0.4,"downfall":0.6,"has_precipitation":true,"features":steps,"attributes":{"test:parent":name},"effects":{"water_color":123},"custom_pack_value":42}).to_string())
        }).collect();
        let configured = Bundle::from([
            ("example:tree_config".into(), json!({"type":"minecraft:tree"}).to_string()),
            ("example:plant_config".into(), json!({"type":"minecraft:simple_block","state":"minecraft:stone"}).to_string()),
            ("example:selector_config".into(), json!({"type":"minecraft:random_selector","features":[{"chance":0.5,"feature":"example:tree"}],"default":"example:plant"}).to_string()),
        ]);
        let mut placed = Bundle::new();
        for (id, feature) in [
            ("example:tree", "example:tree_config"),
            ("example:plant", "example:plant_config"),
            ("example:selector", "example:selector_config"),
            ("minecraft:forest_rock", "example:plant_config"),
            ("minecraft:brown_mushroom_taiga", "example:plant_config"),
        ] {
            placed.insert(
                id.into(),
                json!({"feature":feature,"placement":[]}).to_string(),
            );
        }
        for source in [
            "patch_dead_bush_2",
            "patch_grass_plain",
            "patch_grass_normal",
            "patch_grass_savanna",
            "patch_large_fern",
        ] {
            placed.insert(format!("minecraft:{source}"),json!({"feature":"example:plant_config","placement":[{"type":"minecraft:rarity_filter","chance":12}]}).to_string());
        }
        let tags = BTreeMap::from([
            ("example:taiga".into(), vec!["minecraft:taiga".into()]),
            ("example:beach".into(), vec!["minecraft:beach".into()]),
            ("example:ocean".into(), vec!["minecraft:warm_ocean".into()]),
        ]);
        (parents, configured, placed, tags)
    }
    #[test]
    fn parent_properties_aliases_climate_and_tags_are_preserved_deliberately() {
        let (parents, configured, placed, tags) = fixtures();
        let before = (parents.clone(), placed.clone());
        let data = derive(&parents, &configured, &placed, &tags, &BTreeMap::new()).unwrap();
        assert_eq!(
            data.biomes
                .keys()
                .map(String::as_str)
                .collect::<BTreeSet<_>>(),
            terraforged_native_core::classic_biomes::VARIANTS
                .into_iter()
                .collect()
        );
        assert_eq!(data.biomes.len(), 17);
        let fir: Value = serde_json::from_str(&data.biomes["reterraforged:fir_forest"]).unwrap();
        assert_eq!(fir["custom_pack_value"], 42);
        assert_eq!(fir["features"][9][0], "example:selector");
        assert_eq!(fir["attributes"]["test:parent"], "taiga");
        let steppe: Value = serde_json::from_str(&data.biomes["reterraforged:steppe"]).unwrap();
        assert_eq!(steppe["attributes"]["test:parent"], "savanna");
        assert_eq!(steppe["has_precipitation"], false);
        assert_eq!(steppe["temperature"].as_f64().unwrap() as f32, 1.2_f32);
        assert_eq!(steppe["downfall"].as_f64().unwrap() as f32, 0.05_f32);
        assert_eq!(
            data.biome_tags["example:taiga"],
            vec!["reterraforged:fir_forest", "reterraforged:taiga_scrub"]
        );
        assert_eq!(
            data.biome_tags["example:beach"],
            vec!["reterraforged:warm_beach"]
        );
        assert!(!data.biome_tags.contains_key("example:ocean"));
        let alias: Value =
            serde_json::from_str(&data.placed_features["reterraforged:classic_dead_bush"]).unwrap();
        assert_eq!(alias["feature"], "example:plant_config");
        assert_eq!(
            alias["placement"],
            json!([{"type":"minecraft:rarity_filter","chance":12},{"type":"minecraft:count","count":1}])
        );
        assert_eq!((parents, placed), before);
    }
    #[test]
    fn nested_tree_removal_accepts_flat_and_wrapped_selectors() {
        for wrapped in [false, true] {
            let (parents, mut configured, placed, tags) = fixtures();
            if wrapped {
                configured.insert("example:selector_config".into(),json!({"type":"minecraft:random_selector","config":{"features":[{"chance":0.5,"feature":"example:tree"}],"default":"example:plant"}}).to_string());
            }
            let data = derive(&parents, &configured, &placed, &tags, &BTreeMap::new()).unwrap();
            let steppe: Value = serde_json::from_str(&data.biomes["reterraforged:steppe"]).unwrap();
            assert_eq!(
                steppe["features"][9],
                json!([
                    "example:plant",
                    "reterraforged:classic_dead_bush",
                    "reterraforged:classic_grass_plain",
                    "reterraforged:classic_grass_normal",
                    "reterraforged:classic_grass_savanna"
                ])
            );
            let scrub: Value =
                serde_json::from_str(&data.biomes["reterraforged:taiga_scrub"]).unwrap();
            assert_eq!(scrub["features"][9][0], "minecraft:forest_rock");
            assert_eq!(scrub["features"][9][1], "example:plant");
        }
    }
    #[test]
    fn inline_placed_holders_resolve_their_configured_feature() {
        let (parents, mut configured, placed, tags) = fixtures();
        configured.insert("example:selector_config".into(), json!({"type":"minecraft:simple_random_selector","features":[{"feature":{"type":"minecraft:root_system","feature":{"feature":"example:tree_config","placement":[]}},"placement":[]}]}).to_string());
        let data = derive(&parents, &configured, &placed, &tags, &BTreeMap::new()).unwrap();
        let scrub: Value = serde_json::from_str(&data.biomes["reterraforged:taiga_scrub"]).unwrap();
        assert_eq!(scrub["features"][9][0], "minecraft:forest_rock");
        assert_eq!(scrub["features"][9][1], "example:plant");
    }
    #[test]
    fn previous_modifiers_are_removed_before_resolving_features() {
        let (mut parents, configured, placed, tags) = fixtures();
        for raw in parents.values_mut() {
            let mut parent: Value = serde_json::from_str(raw).unwrap();
            parent["features"][0] = json!([
                "reterraforged:erode",
                "reterraforged:decorate_snow",
                "reterraforged:swamp_surface"
            ]);
            *raw = parent.to_string();
        }
        let data = derive(&parents, &configured, &placed, &tags, &BTreeMap::new()).unwrap();
        for raw in data.biomes.values() {
            let biome: Value = serde_json::from_str(raw).unwrap();
            for step in biome["features"].as_array().unwrap() {
                for id in step.as_array().unwrap() {
                    assert!(!matches!(
                        id.as_str(),
                        Some(
                            "reterraforged:erode"
                                | "reterraforged:decorate_snow"
                                | "reterraforged:swamp_surface"
                        )
                    ));
                }
            }
        }
    }
    #[test]
    fn missing_parents_and_incompatible_step_schemas_fail() {
        let (mut parents, configured, placed, tags) = fixtures();
        parents.remove("minecraft:taiga");
        assert!(
            derive(&parents, &configured, &placed, &tags, &BTreeMap::new())
                .err()
                .unwrap()
                .contains("minecraft:taiga")
        );
        let (mut parents, configured, placed, tags) = fixtures();
        let mut badlands: Value = serde_json::from_str(&parents["minecraft:badlands"]).unwrap();
        badlands["features"] = json!([]);
        parents.insert("minecraft:badlands".into(), badlands.to_string());
        assert!(
            derive(&parents, &configured, &placed, &tags, &BTreeMap::new())
                .err()
                .unwrap()
                .contains("eleven-step")
        );
    }
    #[test]
    fn feature_tags_missing_references_and_cycles_are_checked() {
        let (parents, mut configured, placed, tags) = fixtures();
        configured.insert(
            "example:selector_config".into(),
            json!({"type":"minecraft:simple_random_selector","features":"#example:trees"})
                .to_string(),
        );
        let mut feature_tags = BTreeMap::from([(
            "example:trees".into(),
            vec!["example:tree".into(), "example:plant".into()],
        )]);
        assert!(derive(&parents, &configured, &placed, &tags, &feature_tags).is_ok());
        feature_tags.insert("example:trees".into(), vec!["#example:trees".into()]);
        assert!(
            derive(&parents, &configured, &placed, &tags, &feature_tags)
                .err()
                .unwrap()
                .contains("Cyclic")
        );
        configured.insert("example:selector_config".into(),json!({"type":"minecraft:random_selector","features":[{"chance":0.5,"feature":"example:tree"}],"default":"example:missing"}).to_string());
        assert!(
            derive(&parents, &configured, &placed, &tags, &BTreeMap::new())
                .err()
                .unwrap()
                .contains("example:missing")
        );
        configured.insert(
            "example:selector_config".into(),
            json!({"type":"minecraft:simple_random_selector","features":["example:selector"]})
                .to_string(),
        );
        assert!(
            derive(&parents, &configured, &placed, &tags, &BTreeMap::new())
                .err()
                .unwrap()
                .contains("Cyclic")
        );
    }
}
