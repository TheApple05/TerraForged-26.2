use terraforged_native_core::{
    cell::{BiomeType, Cell},
    layout::{Levels, Size},
};
const INTS: [i32; 13] = [
    i32::MIN,
    -1024,
    -17,
    -16,
    -1,
    0,
    1,
    15,
    16,
    17,
    63,
    256,
    i32::MAX,
];
const FLOATS: [u32; 6] = [0, 0x80000000, 1, 0x7f7fffff, 0xbf800000, 0x7fc01234];
fn raw(f: f32) -> String {
    format!("{:08x}", f.to_bits())
}
fn bits(f: f32) -> String {
    if f.is_nan() { "NaN".into() } else { raw(f) }
}
fn size(name: &str, s: Size) {
    println!(
        "size/meta/{name}\t{},{},{},{},{},{},{}",
        s.size, s.mask, s.border, s.total, s.lower_border, s.upper_border, s.array_size
    );
    for (i, x) in INTS.into_iter().enumerate() {
        for (j, z) in INTS.into_iter().enumerate() {
            println!(
                "size/index/{name}/{i}/{j}\t{},{},{}",
                s.index_of(x, z),
                s.mask_index(x),
                s.mask_index(z)
            );
        }
    }
}
fn snapshot(name: &str, c: &Cell) {
    for (field, value) in [
        ("beachNoise", raw(c.beach_noise)),
        ("biome", (c.biome as u8).to_string()),
        ("biomeRegionEdge", raw(c.biome_region_edge)),
        ("biomeRegionId", raw(c.biome_region_id)),
        ("continentEdge", raw(c.continent_edge)),
        ("continentId", raw(c.continent_id)),
        ("continentX", c.continent_x.to_string()),
        ("continentZ", c.continent_z.to_string()),
        ("erosion", raw(c.erosion)),
        ("erosionMask", c.erosion_mask.to_string()),
        ("gradient", raw(c.gradient)),
        ("height", raw(c.height)),
        ("heightErosion", raw(c.height_erosion)),
        ("macroBiomeId", raw(c.macro_biome_id)),
        ("moisture", raw(c.moisture)),
        ("regionMoisture", raw(c.region_moisture)),
        ("regionTemperature", raw(c.region_temperature)),
        ("riverMask", raw(c.river_mask)),
        ("sediment", raw(c.sediment)),
        ("temperature", raw(c.temperature)),
        ("terrain", c.terrain.to_string()),
        ("terrainRegionEdge", raw(c.terrain_region_edge)),
        ("terrainRegionId", raw(c.terrain_region_id)),
        ("weirdness", raw(c.weirdness)),
    ] {
        println!("cell/{name}/{field}\t{value}");
    }
}
fn fill(c: &mut Cell, pattern: usize) {
    for (index, field) in [
        (0, &mut c.beach_noise),
        (2, &mut c.biome_region_edge),
        (3, &mut c.biome_region_id),
        (4, &mut c.continent_edge),
        (5, &mut c.continent_id),
        (8, &mut c.erosion),
        (10, &mut c.gradient),
        (11, &mut c.height),
        (12, &mut c.height_erosion),
        (13, &mut c.macro_biome_id),
        (14, &mut c.moisture),
        (15, &mut c.region_moisture),
        (16, &mut c.region_temperature),
        (17, &mut c.river_mask),
        (18, &mut c.sediment),
        (19, &mut c.temperature),
        (21, &mut c.terrain_region_edge),
        (22, &mut c.terrain_region_id),
        (23, &mut c.weirdness),
    ] {
        *field = f32::from_bits(FLOATS[(index + pattern) % FLOATS.len()]);
    }
    c.continent_x = i32::MIN + 6 + pattern as i32 * 100;
    c.continent_z = i32::MIN + 7 + pattern as i32 * 100;
    c.erosion_mask = true;
    c.terrain = 8;
    c.biome = BiomeType::Taiga;
}
fn main() {
    let heights = [i32::MIN, -1, 0, 1, 2, 256, 384, 1024, i32::MAX];
    let seas = [i32::MIN, -1, 0, 1, 63, 255, 256, i32::MAX];
    let values = [
        -f32::MAX,
        -1.5,
        -1.0,
        -f32::from_bits(1),
        -0.0,
        0.0,
        f32::from_bits(1),
        0.25,
        0.5,
        1.0,
        1.5,
        f32::MAX,
        f32::INFINITY,
        f32::NAN,
    ];
    for (h, height) in heights.into_iter().enumerate() {
        for (s, sea) in seas.into_iter().enumerate() {
            let l = Levels::new(height, sea);
            let key = format!("{h}/{s}");
            println!(
                "levels/meta/{key}\t{},{},{},{},{},{},{}",
                l.world_height,
                bits(l.unit),
                l.water_y,
                l.ground_level,
                l.water_level,
                bits(l.ground),
                bits(l.water)
            );
            for (i, y) in INTS.into_iter().enumerate() {
                println!(
                    "levels/int/{key}/{i}\t{},{},{},{}",
                    bits(l.scale_i32(y)),
                    bits(l.elevation_i32(y)),
                    bits(l.water(y)),
                    bits(l.ground(y))
                );
            }
            for (i, f) in values.into_iter().enumerate() {
                println!(
                    "levels/float/{key}/{i}\t{},{}",
                    l.scale_f32(f),
                    bits(l.elevation_f32(f))
                );
            }
        }
    }
    let factors = [-32, -1, 0, 1, 2, 3, 4, 30, 31, 32, 33];
    let borders = [-1, 0, 1, 2, i32::MIN, i32::MAX];
    let sizes = [-1, 0, 1, 3, 128, i32::MAX];
    for (i, s) in sizes.into_iter().enumerate() {
        for (j, b) in borders.into_iter().enumerate() {
            size(&format!("make/{i}/{j}"), Size::make(s, b));
        }
    }
    for (i, f) in factors.into_iter().enumerate() {
        for (j, b) in borders.into_iter().enumerate() {
            size(&format!("chunks/{i}/{j}"), Size::chunks(f, b));
            size(&format!("blocks/{i}/{j}"), Size::blocks(f, b));
        }
    }
    for (i, x) in INTS.into_iter().enumerate() {
        println!(
            "size/convert/{i}\t{},{}",
            Size::chunk_to_block(x),
            Size::block_to_chunk(x)
        );
        for (j, z) in INTS.into_iter().enumerate() {
            println!("size/count/{i}/{j}\t{}", Size::count(x, z, z, x));
        }
    }
    snapshot("default", &Cell::default());
    for pattern in 0..2 {
        let mut source = Cell::default();
        fill(&mut source, pattern);
        snapshot(&format!("{pattern}/filled"), &source);
        let mut copy = Cell::default();
        copy.copy_from(&source);
        snapshot(&format!("{pattern}/copy"), &copy);
        source.height = -999.0;
        snapshot(&format!("{pattern}/detached"), &copy);
        source.reset();
        snapshot(&format!("{pattern}/reset"), &source);
    }
}
