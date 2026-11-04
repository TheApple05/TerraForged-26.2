use terraforged_native_core::cell::Cell;
pub fn bits(f: f32) -> String {
    if f.is_nan() {
        "NaN".into()
    } else {
        format!("{:08x}", f.to_bits())
    }
}
pub fn snapshot(c: &Cell) -> String {
    [
        bits(c.beach_noise),
        (c.biome as u8).to_string(),
        bits(c.biome_region_edge),
        bits(c.biome_region_id),
        bits(c.continent_edge),
        bits(c.continent_id),
        c.continent_x.to_string(),
        c.continent_z.to_string(),
        bits(c.erosion),
        c.erosion_mask.to_string(),
        bits(c.gradient),
        bits(c.height),
        bits(c.height_erosion),
        bits(c.macro_biome_id),
        bits(c.moisture),
        bits(c.region_moisture),
        bits(c.region_temperature),
        bits(c.river_mask),
        bits(c.sediment),
        bits(c.temperature),
        c.terrain.to_string(),
        bits(c.terrain_region_edge),
        bits(c.terrain_region_id),
        bits(c.weirdness),
    ]
    .join(",")
}
pub fn cell(dirty: bool) -> Cell {
    if !dirty {
        return Cell::default();
    }
    Cell {
        height: -3.25,
        height_erosion: -3.25,
        sediment: -3.25,
        gradient: -3.25,
        region_moisture: -3.25,
        region_temperature: -3.25,
        continent_id: -3.25,
        continent_edge: -3.25,
        terrain_region_id: -3.25,
        terrain_region_edge: -3.25,
        biome_region_id: -3.25,
        biome_region_edge: -3.25,
        macro_biome_id: -3.25,
        river_mask: -3.25,
        continent_x: -123456,
        continent_z: -123456,
        erosion_mask: true,
        erosion: -3.25,
        weirdness: -3.25,
        temperature: -3.25,
        moisture: -3.25,
        beach_noise: -3.25,
        ..Cell::default()
    }
}
