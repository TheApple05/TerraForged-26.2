use std::collections::BTreeMap;
use terraforged_native_adapter::preset::CellPresetInput;
use terraforged_native_core::{
    graph::Noise as N, noises as n, preset::*, worley::DistanceFunction,
};
fn bits(f: f32) -> String {
    if f.is_nan() {
        "NaN".into()
    } else {
        format!("{:08x}", f.to_bits())
    }
}
fn s() -> N {
    n::simplex(12345, 80, 3).unwrap()
}
fn flatten(name: &str, p: &KernelPreset) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    out.insert(
        format!("preset/{name}/world/continent/continentType"),
        p.world.continent.continent_type.serialized_name().into(),
    );
    out.insert(
        format!("preset/{name}/world/continent/continentShape"),
        match p.world.continent.continent_shape {
            DistanceFunction::Euclidean => "EUCLIDEAN",
            DistanceFunction::Manhattan => "MANHATTAN",
            DistanceFunction::Natural => "NATURAL",
        }
        .into(),
    );
    out.insert(
        format!("preset/{name}/world/continent/continentScale"),
        p.world.continent.continent_scale.to_string(),
    );
    out.insert(
        format!("preset/{name}/world/continent/continentJitter"),
        bits(p.world.continent.continent_jitter),
    );
    out.insert(
        format!("preset/{name}/world/continent/continentSkipping"),
        bits(p.world.continent.continent_skipping),
    );
    out.insert(
        format!("preset/{name}/world/continent/continentSizeVariance"),
        bits(p.world.continent.continent_size_variance),
    );
    out.insert(
        format!("preset/{name}/world/continent/continentNoiseOctaves"),
        p.world.continent.continent_noise_octaves.to_string(),
    );
    out.insert(
        format!("preset/{name}/world/continent/continentNoiseGain"),
        bits(p.world.continent.continent_noise_gain),
    );
    out.insert(
        format!("preset/{name}/world/continent/continentNoiseLacunarity"),
        bits(p.world.continent.continent_noise_lacunarity),
    );
    out.insert(
        format!("preset/{name}/world/controlPoints/mushroomFieldsInland"),
        bits(p.world.control_points.mushroom_fields_inland),
    );
    out.insert(
        format!("preset/{name}/world/controlPoints/mushroomFieldsCoast"),
        bits(p.world.control_points.mushroom_fields_coast),
    );
    out.insert(
        format!("preset/{name}/world/controlPoints/deepOcean"),
        bits(p.world.control_points.deep_ocean),
    );
    out.insert(
        format!("preset/{name}/world/controlPoints/shallowOcean"),
        bits(p.world.control_points.shallow_ocean),
    );
    out.insert(
        format!("preset/{name}/world/controlPoints/beach"),
        bits(p.world.control_points.beach),
    );
    out.insert(
        format!("preset/{name}/world/controlPoints/coast"),
        bits(p.world.control_points.coast),
    );
    out.insert(
        format!("preset/{name}/world/controlPoints/inland"),
        bits(p.world.control_points.inland),
    );
    out.insert(
        format!("preset/{name}/world/properties/spawnType"),
        p.world.properties.spawn_type.to_string(),
    );
    out.insert(
        format!("preset/{name}/world/properties/worldHeight"),
        p.world.properties.world_height.to_string(),
    );
    out.insert(
        format!("preset/{name}/world/properties/worldDepth"),
        p.world.properties.world_depth.to_string(),
    );
    out.insert(
        format!("preset/{name}/world/properties/seaLevel"),
        p.world.properties.sea_level.to_string(),
    );
    out.insert(
        format!("preset/{name}/world/properties/lavaLevel"),
        p.world.properties.lava_level.to_string(),
    );
    out.insert(
        format!("preset/{name}/terrain/general/terrainSeedOffset"),
        p.terrain.general.terrain_seed_offset.to_string(),
    );
    out.insert(
        format!("preset/{name}/terrain/general/terrainRegionSize"),
        p.terrain.general.terrain_region_size.to_string(),
    );
    out.insert(
        format!("preset/{name}/terrain/general/globalVerticalScale"),
        bits(p.terrain.general.global_vertical_scale),
    );
    out.insert(
        format!("preset/{name}/terrain/general/globalHorizontalScale"),
        bits(p.terrain.general.global_horizontal_scale),
    );
    out.insert(
        format!("preset/{name}/terrain/general/fancyMountains"),
        p.terrain.general.fancy_mountains.to_string(),
    );
    out.insert(
        format!("preset/{name}/terrain/steppe/weight"),
        bits(p.terrain.steppe.weight),
    );
    out.insert(
        format!("preset/{name}/terrain/steppe/baseScale"),
        bits(p.terrain.steppe.base_scale),
    );
    out.insert(
        format!("preset/{name}/terrain/steppe/verticalScale"),
        bits(p.terrain.steppe.vertical_scale),
    );
    out.insert(
        format!("preset/{name}/terrain/steppe/horizontalScale"),
        bits(p.terrain.steppe.horizontal_scale),
    );
    out.insert(
        format!("preset/{name}/terrain/plains/weight"),
        bits(p.terrain.plains.weight),
    );
    out.insert(
        format!("preset/{name}/terrain/plains/baseScale"),
        bits(p.terrain.plains.base_scale),
    );
    out.insert(
        format!("preset/{name}/terrain/plains/verticalScale"),
        bits(p.terrain.plains.vertical_scale),
    );
    out.insert(
        format!("preset/{name}/terrain/plains/horizontalScale"),
        bits(p.terrain.plains.horizontal_scale),
    );
    out.insert(
        format!("preset/{name}/terrain/hills/weight"),
        bits(p.terrain.hills.weight),
    );
    out.insert(
        format!("preset/{name}/terrain/hills/baseScale"),
        bits(p.terrain.hills.base_scale),
    );
    out.insert(
        format!("preset/{name}/terrain/hills/verticalScale"),
        bits(p.terrain.hills.vertical_scale),
    );
    out.insert(
        format!("preset/{name}/terrain/hills/horizontalScale"),
        bits(p.terrain.hills.horizontal_scale),
    );
    out.insert(
        format!("preset/{name}/terrain/dales/weight"),
        bits(p.terrain.dales.weight),
    );
    out.insert(
        format!("preset/{name}/terrain/dales/baseScale"),
        bits(p.terrain.dales.base_scale),
    );
    out.insert(
        format!("preset/{name}/terrain/dales/verticalScale"),
        bits(p.terrain.dales.vertical_scale),
    );
    out.insert(
        format!("preset/{name}/terrain/dales/horizontalScale"),
        bits(p.terrain.dales.horizontal_scale),
    );
    out.insert(
        format!("preset/{name}/terrain/plateau/weight"),
        bits(p.terrain.plateau.weight),
    );
    out.insert(
        format!("preset/{name}/terrain/plateau/baseScale"),
        bits(p.terrain.plateau.base_scale),
    );
    out.insert(
        format!("preset/{name}/terrain/plateau/verticalScale"),
        bits(p.terrain.plateau.vertical_scale),
    );
    out.insert(
        format!("preset/{name}/terrain/plateau/horizontalScale"),
        bits(p.terrain.plateau.horizontal_scale),
    );
    out.insert(
        format!("preset/{name}/terrain/badlands/weight"),
        bits(p.terrain.badlands.weight),
    );
    out.insert(
        format!("preset/{name}/terrain/badlands/baseScale"),
        bits(p.terrain.badlands.base_scale),
    );
    out.insert(
        format!("preset/{name}/terrain/badlands/verticalScale"),
        bits(p.terrain.badlands.vertical_scale),
    );
    out.insert(
        format!("preset/{name}/terrain/badlands/horizontalScale"),
        bits(p.terrain.badlands.horizontal_scale),
    );
    out.insert(
        format!("preset/{name}/terrain/torridonian/weight"),
        bits(p.terrain.torridonian.weight),
    );
    out.insert(
        format!("preset/{name}/terrain/torridonian/baseScale"),
        bits(p.terrain.torridonian.base_scale),
    );
    out.insert(
        format!("preset/{name}/terrain/torridonian/verticalScale"),
        bits(p.terrain.torridonian.vertical_scale),
    );
    out.insert(
        format!("preset/{name}/terrain/torridonian/horizontalScale"),
        bits(p.terrain.torridonian.horizontal_scale),
    );
    out.insert(
        format!("preset/{name}/terrain/mountains/weight"),
        bits(p.terrain.mountains.weight),
    );
    out.insert(
        format!("preset/{name}/terrain/mountains/baseScale"),
        bits(p.terrain.mountains.base_scale),
    );
    out.insert(
        format!("preset/{name}/terrain/mountains/verticalScale"),
        bits(p.terrain.mountains.vertical_scale),
    );
    out.insert(
        format!("preset/{name}/terrain/mountains/horizontalScale"),
        bits(p.terrain.mountains.horizontal_scale),
    );
    out.insert(
        format!("preset/{name}/terrain/volcano/weight"),
        bits(p.terrain.volcano.weight),
    );
    out.insert(
        format!("preset/{name}/terrain/volcano/baseScale"),
        bits(p.terrain.volcano.base_scale),
    );
    out.insert(
        format!("preset/{name}/terrain/volcano/verticalScale"),
        bits(p.terrain.volcano.vertical_scale),
    );
    out.insert(
        format!("preset/{name}/terrain/volcano/horizontalScale"),
        bits(p.terrain.volcano.horizontal_scale),
    );
    out.insert(
        format!("preset/{name}/climate/temperature/seedOffset"),
        p.climate.temperature.seed_offset.to_string(),
    );
    out.insert(
        format!("preset/{name}/climate/temperature/scale"),
        p.climate.temperature.scale.to_string(),
    );
    out.insert(
        format!("preset/{name}/climate/temperature/falloff"),
        p.climate.temperature.falloff.to_string(),
    );
    out.insert(
        format!("preset/{name}/climate/temperature/min"),
        bits(p.climate.temperature.min),
    );
    out.insert(
        format!("preset/{name}/climate/temperature/max"),
        bits(p.climate.temperature.max),
    );
    out.insert(
        format!("preset/{name}/climate/temperature/bias"),
        bits(p.climate.temperature.bias),
    );
    out.insert(
        format!("preset/{name}/climate/moisture/seedOffset"),
        p.climate.moisture.seed_offset.to_string(),
    );
    out.insert(
        format!("preset/{name}/climate/moisture/scale"),
        p.climate.moisture.scale.to_string(),
    );
    out.insert(
        format!("preset/{name}/climate/moisture/falloff"),
        p.climate.moisture.falloff.to_string(),
    );
    out.insert(
        format!("preset/{name}/climate/moisture/min"),
        bits(p.climate.moisture.min),
    );
    out.insert(
        format!("preset/{name}/climate/moisture/max"),
        bits(p.climate.moisture.max),
    );
    out.insert(
        format!("preset/{name}/climate/moisture/bias"),
        bits(p.climate.moisture.bias),
    );
    out.insert(
        format!("preset/{name}/climate/biomeShape/biomeSize"),
        p.climate.biome_shape.biome_size.to_string(),
    );
    out.insert(
        format!("preset/{name}/climate/biomeShape/macroNoiseSize"),
        p.climate.biome_shape.macro_noise_size.to_string(),
    );
    out.insert(
        format!("preset/{name}/climate/biomeShape/biomeWarpScale"),
        p.climate.biome_shape.biome_warp_scale.to_string(),
    );
    out.insert(
        format!("preset/{name}/climate/biomeShape/biomeWarpStrength"),
        p.climate.biome_shape.biome_warp_strength.to_string(),
    );
    out.insert(
        format!("preset/{name}/climate/biomeEdgeShape/type"),
        p.climate.biome_edge_shape.r#type.serialized_name().into(),
    );
    out.insert(
        format!("preset/{name}/climate/biomeEdgeShape/scale"),
        p.climate.biome_edge_shape.scale.to_string(),
    );
    out.insert(
        format!("preset/{name}/climate/biomeEdgeShape/octaves"),
        p.climate.biome_edge_shape.octaves.to_string(),
    );
    out.insert(
        format!("preset/{name}/climate/biomeEdgeShape/gain"),
        bits(p.climate.biome_edge_shape.gain),
    );
    out.insert(
        format!("preset/{name}/climate/biomeEdgeShape/lacunarity"),
        bits(p.climate.biome_edge_shape.lacunarity),
    );
    out.insert(
        format!("preset/{name}/climate/biomeEdgeShape/strength"),
        p.climate.biome_edge_shape.strength.to_string(),
    );
    out.insert(
        format!("preset/{name}/rivers/seedOffset"),
        p.rivers.seed_offset.to_string(),
    );
    out.insert(
        format!("preset/{name}/rivers/riverCount"),
        p.rivers.river_count.to_string(),
    );
    out.insert(
        format!("preset/{name}/rivers/mainRivers/bedDepth"),
        p.rivers.main_rivers.bed_depth.to_string(),
    );
    out.insert(
        format!("preset/{name}/rivers/mainRivers/minBankHeight"),
        p.rivers.main_rivers.min_bank_height.to_string(),
    );
    out.insert(
        format!("preset/{name}/rivers/mainRivers/maxBankHeight"),
        p.rivers.main_rivers.max_bank_height.to_string(),
    );
    out.insert(
        format!("preset/{name}/rivers/mainRivers/bankWidth"),
        p.rivers.main_rivers.bank_width.to_string(),
    );
    out.insert(
        format!("preset/{name}/rivers/mainRivers/bedWidth"),
        p.rivers.main_rivers.bed_width.to_string(),
    );
    out.insert(
        format!("preset/{name}/rivers/mainRivers/fade"),
        bits(p.rivers.main_rivers.fade),
    );
    out.insert(
        format!("preset/{name}/rivers/branchRivers/bedDepth"),
        p.rivers.branch_rivers.bed_depth.to_string(),
    );
    out.insert(
        format!("preset/{name}/rivers/branchRivers/minBankHeight"),
        p.rivers.branch_rivers.min_bank_height.to_string(),
    );
    out.insert(
        format!("preset/{name}/rivers/branchRivers/maxBankHeight"),
        p.rivers.branch_rivers.max_bank_height.to_string(),
    );
    out.insert(
        format!("preset/{name}/rivers/branchRivers/bankWidth"),
        p.rivers.branch_rivers.bank_width.to_string(),
    );
    out.insert(
        format!("preset/{name}/rivers/branchRivers/bedWidth"),
        p.rivers.branch_rivers.bed_width.to_string(),
    );
    out.insert(
        format!("preset/{name}/rivers/branchRivers/fade"),
        bits(p.rivers.branch_rivers.fade),
    );
    out.insert(
        format!("preset/{name}/rivers/lakes/chance"),
        bits(p.rivers.lakes.chance),
    );
    out.insert(
        format!("preset/{name}/rivers/lakes/minStartDistance"),
        bits(p.rivers.lakes.min_start_distance),
    );
    out.insert(
        format!("preset/{name}/rivers/lakes/maxStartDistance"),
        bits(p.rivers.lakes.max_start_distance),
    );
    out.insert(
        format!("preset/{name}/rivers/lakes/depth"),
        p.rivers.lakes.depth.to_string(),
    );
    out.insert(
        format!("preset/{name}/rivers/lakes/sizeMin"),
        p.rivers.lakes.size_min.to_string(),
    );
    out.insert(
        format!("preset/{name}/rivers/lakes/sizeMax"),
        p.rivers.lakes.size_max.to_string(),
    );
    out.insert(
        format!("preset/{name}/rivers/lakes/minBankHeight"),
        p.rivers.lakes.min_bank_height.to_string(),
    );
    out.insert(
        format!("preset/{name}/rivers/lakes/maxBankHeight"),
        p.rivers.lakes.max_bank_height.to_string(),
    );
    out.insert(
        format!("preset/{name}/rivers/wetlands/chance"),
        bits(p.rivers.wetlands.chance),
    );
    out.insert(
        format!("preset/{name}/rivers/wetlands/sizeMin"),
        p.rivers.wetlands.size_min.to_string(),
    );
    out.insert(
        format!("preset/{name}/rivers/wetlands/sizeMax"),
        p.rivers.wetlands.size_max.to_string(),
    );
    out.insert(
        format!("preset/{name}/filters/erosion/dropletsPerChunk"),
        p.filters.erosion.droplets_per_chunk.to_string(),
    );
    out.insert(
        format!("preset/{name}/filters/erosion/dropletLifetime"),
        p.filters.erosion.droplet_lifetime.to_string(),
    );
    out.insert(
        format!("preset/{name}/filters/erosion/dropletVolume"),
        bits(p.filters.erosion.droplet_volume),
    );
    out.insert(
        format!("preset/{name}/filters/erosion/dropletVelocity"),
        bits(p.filters.erosion.droplet_velocity),
    );
    out.insert(
        format!("preset/{name}/filters/erosion/erosionRate"),
        bits(p.filters.erosion.erosion_rate),
    );
    out.insert(
        format!("preset/{name}/filters/erosion/depositeRate"),
        bits(p.filters.erosion.deposite_rate),
    );
    out.insert(
        format!("preset/{name}/filters/smoothing/iterations"),
        p.filters.smoothing.iterations.to_string(),
    );
    out.insert(
        format!("preset/{name}/filters/smoothing/smoothingRadius"),
        bits(p.filters.smoothing.smoothing_radius),
    );
    out.insert(
        format!("preset/{name}/filters/smoothing/smoothingRate"),
        bits(p.filters.smoothing.smoothing_rate),
    );
    out
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file = std::env::args()
        .nth(1)
        .ok_or("Explicit preset inputs required")?;
    let raw = std::fs::read_to_string(file)?;
    let cases: serde_json::Value = serde_json::from_str(&raw)?;
    let mut graphs: Vec<(String, N)> = Vec::new();
    for (name, value) in cases.as_object().ok_or("Expected input dictionary")? {
        let input = CellPresetInput::from_json(&value.to_string())?;
        let p = &input.kernel;
        for (key, value) in flatten(name, p) {
            println!("{key}\t{value}");
        }
        let c = p.world.control_points.make();
        println!(
            "controls/{name}\t{},{},{},{},{},{},{},{}",
            bits(c.mushroom_fields_inland),
            bits(c.mushroom_fields_coast),
            bits(c.deep_ocean),
            bits(c.shallow_ocean),
            bits(c.beach),
            bits(c.coast),
            bits(c.coast_marker),
            bits(c.inland)
        );
        println!(
            "controls-valid/{name}\t{}",
            p.world.control_points.validate()
        );
        println!(
            "terrain-scaler/{name}\t{}",
            p.world.properties.terrain_scaler()
        );
        for (kind, r) in [
            ("temperature", &p.climate.temperature),
            ("moisture", &p.climate.moisture),
        ] {
            println!(
                "range/{name}/{kind}\t{},{},{}",
                bits(r.get_min()),
                bits(r.get_max()),
                bits(r.get_bias())
            );
            graphs.push((format!("range-{name}-{kind}"), r.apply(s())?));
        }
    }
    graphs.push(("perlin-default".into(), n::perlin(12345, 80, 3)?));
    graphs.push(("perlin2-default".into(), n::perlin2(12345, 80, 3)?));
    graphs.push(("simplex-default".into(), n::simplex(12345, 80, 3)?));
    graphs.push(("simplex2-default".into(), n::simplex2(12345, 80, 3)?));
    graphs.push((
        "perlin-ridge-default".into(),
        n::perlin_ridge(12345, 80, 3)?,
    ));
    graphs.push((
        "simplex-ridge-default".into(),
        n::simplex_ridge(12345, 80, 3)?,
    ));
    graphs.push(("billow-default".into(), n::billow(12345, 80, 3)?));
    graphs.push(("cubic-default".into(), n::cubic(12345, 80, 3)?));
    graphs.push((
        "perlin-full".into(),
        n::perlin_with(i32::MIN, 24, 4, 1.75, 0.625)?,
    ));
    graphs.push((
        "perlin2-full".into(),
        n::perlin2_with(i32::MIN, 24, 4, 1.75, 0.625)?,
    ));
    graphs.push((
        "simplex-full".into(),
        n::simplex_with(i32::MIN, 24, 4, 1.75, 0.625)?,
    ));
    graphs.push((
        "simplex2-full".into(),
        n::simplex2_with(i32::MIN, 24, 4, 1.75, 0.625)?,
    ));
    graphs.push((
        "perlin-ridge-full".into(),
        n::perlin_ridge_with(i32::MIN, 24, 4, 1.75, 0.625)?,
    ));
    graphs.push((
        "simplex-ridge-full".into(),
        n::simplex_ridge_with(i32::MIN, 24, 4, 1.75, 0.625)?,
    ));
    graphs.push((
        "billow-full".into(),
        n::billow_with(i32::MIN, 24, 4, 1.75, 0.625)?,
    ));
    graphs.push((
        "cubic-full".into(),
        n::cubic_with(i32::MIN, 24, 4, 1.75, 0.625)?,
    ));
    graphs.push(("worley-default".into(), n::worley(12345, 80)?));
    graphs.push(("edge-default".into(), n::worley_edge(12345, 80)?));
    graphs.push(("white-default".into(), n::white(12345, 80)?));
    graphs.push(("sin-default".into(), n::sin(12345, 0.0125, s())?));
    graphs.push((
        "map-constant-shortcut".into(),
        n::map_range(N::constant(0.5)?, 0.5, 0.5)?,
    ));
    graphs.push(("map-unit-shortcut".into(), n::map_range(s(), 0.0, 1.0)?));
    graphs.push(("map-reverse".into(), n::map_range(s(), 1.0, -1.0)?));
    graphs.push((
        "warp-perlin".into(),
        n::warp_perlin(s(), i32::MAX, 80, 3, 100.0)?,
    ));
    graphs.push(("warp-white".into(), n::warp_white(s(), i32::MAX, 8, 10.0)?));
    graphs.push(("frequency".into(), n::frequency(s(), -0.25, 0.75)?));
    for kind in [
        EdgeType::Billow,
        EdgeType::Voronoi,
        EdgeType::VoronoiEdge,
        EdgeType::Constant,
        EdgeType::Cubic,
        EdgeType::Perlin,
        EdgeType::Perlin2,
        EdgeType::PerlinRidge,
        EdgeType::Simplex,
        EdgeType::Simplex2,
        EdgeType::SimplexRidge,
        EdgeType::Sin,
        EdgeType::White,
    ] {
        graphs.push((
            format!("edge-type-{}", kind.serialized_name()),
            BiomeNoise {
                r#type: kind,
                scale: 24,
                octaves: 3,
                gain: 0.5,
                lacunarity: 2.65,
                strength: 14,
            }
            .build(12345)?,
        ));
    }
    let seeds = [0, 1, -1, 42, i32::MIN, i32::MAX];
    let coords: [f32; 17] = [
        -30_000_000.,
        -17.,
        -16.25,
        -16.,
        -1.5,
        -1.,
        -0.25,
        -0.,
        0.,
        0.25,
        1.,
        15.,
        15.75,
        16.,
        16.25,
        17.,
        30_000_000.,
    ];
    for (index, (name, noise)) in graphs.into_iter().enumerate() {
        println!(
            "graph/{index}\t{name},{},{},{}",
            noise.kind(),
            bits(noise.min_value()),
            bits(noise.max_value())
        );
        for seed in seeds {
            for (i, x) in coords.into_iter().enumerate() {
                for (j, z) in coords.into_iter().enumerate() {
                    println!(
                        "sample/{index}/{seed}/{i}/{j}\t{}",
                        bits(noise.compute(x, z, seed))
                    );
                }
            }
        }
    }
    assert!(n::perlin(0, 80, 257).is_err());
    assert!(n::simplex_ridge(0, 80, 31).is_err());
    let base = cases.as_object().unwrap().values().next().unwrap();
    let mut bad = base.clone();
    bad["world"]["continent"]["continentType"] = serde_json::json!("MISSING");
    assert!(CellPresetInput::from_json(&bad.to_string()).is_err());
    let mut bad = base.clone();
    bad["climate"]["biomeEdgeShape"]["type"] = serde_json::json!("MISSING");
    assert!(CellPresetInput::from_json(&bad.to_string()).is_err());
    let mut bad = base.clone();
    bad["world"]["properties"]["spawnType"] = serde_json::json!("MISSING");
    assert!(CellPresetInput::from_json(&bad.to_string()).is_err());
    let mut bad = base.clone();
    bad.as_object_mut().unwrap().remove("terrain");
    assert!(CellPresetInput::from_json(&bad.to_string()).is_err());
    eprintln!("NATIVE_PRESET_GUARDS_OK missingSections unknownEnums sourceOctaves");
    Ok(())
}
