use terraforged_native_core::{
    biome::{self, ALL},
    cell::BiomeType,
};
fn bits(f: f32) -> String {
    if f.is_nan() {
        "NaN".into()
    } else {
        format!("{:08x}", f.to_bits())
    }
}
fn lookup(t: f32, m: f32, curve: bool) -> String {
    let result = if curve {
        biome::get_curve(t, m)
    } else {
        biome::get_linear(t, m)
    };
    result.map_or_else(|_| "error".into(), |b| (b as u8).to_string())
}
fn pair(b: BiomeType, identity: f32) -> String {
    b.temperature_moisture(identity).map_or_else(
        |_| "error".into(),
        |(t, m)| format!("{},{}", bits(t), bits(m)),
    )
}
fn main() {
    for t in 0..256 {
        for m in 0..256 {
            let t_value = t as f32 / 255.0;
            let m_value = m as f32 / 255.0;
            println!(
                "grid/{t}/{m}\t{},{}",
                lookup(t_value, m_value, true),
                lookup(t_value, m_value, false)
            );
        }
    }
    let edges: [f32; 15] = [
        -1.0,
        -0.001,
        -0.0,
        0.0,
        f32::from_bits(1),
        0.25,
        f32::from_bits(0x3effffff),
        0.5,
        f32::from_bits(0x3f000001),
        0.75,
        1.0,
        f32::from_bits(0x3f800001),
        1.5,
        f32::NAN,
        f32::INFINITY,
    ];
    for (t, t_value) in edges.into_iter().enumerate() {
        for (m, m_value) in edges.into_iter().enumerate() {
            println!(
                "edge/{t}/{m}\t{},{}",
                lookup(t_value, m_value, true),
                lookup(t_value, m_value, false)
            );
        }
    }
    for b in ALL {
        let index = b as u8;
        let r = b.ranges();
        println!(
            "type/{index}\t{},{},{},{},{}",
            b.name(),
            bits(r.min_temperature),
            bits(r.max_temperature),
            bits(r.min_moisture),
            bits(r.max_moisture)
        );
        for i in 0..=256 {
            println!("pair/{index}/{i}\t{}", pair(b, i as f32 / 256.0));
        }
        for (i, value) in edges.into_iter().enumerate() {
            println!("pair-edge/{index}/{i}\t{}", pair(b, value));
            println!(
                "map/{index}/{i}\t{},{}",
                bits(b.map_temperature(value)),
                bits(b.map_moisture(value))
            );
        }
    }
    assert!(BiomeType::from_index(-1).is_none());
    assert!(BiomeType::from_index(11).is_none());
    eprintln!("NATIVE_BIOME_GUARDS_OK invalidOrdinal noOutOfRangeFallback");
}
