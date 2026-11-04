use terraforged_native_core::noise_util as n;

fn bits(value: f32) -> String {
    format!("{:08x}", value.to_bits())
}

fn main() {
    let seeds = [0, 1, -1, 42, i32::MIN, i32::MAX];
    let coordinates: [f32; 17] = [
        -30_000_000.0,
        -17.0,
        -16.25,
        -16.0,
        -1.5,
        -1.0,
        -0.25,
        -0.0,
        0.0,
        0.25,
        1.0,
        15.0,
        15.75,
        16.0,
        16.25,
        17.0,
        30_000_000.0,
    ];
    for seed in seeds {
        for (i, x) in coordinates.iter().enumerate() {
            for (j, z) in coordinates.iter().enumerate() {
                let ix = n::floor_legacy(*x);
                let iz = n::floor_legacy(*z);
                println!(
                    "noise/{seed}/{i}/{j}\t{ix},{iz},{},{},{}",
                    n::hash(ix, iz),
                    n::hash_2d(seed, ix, iz),
                    bits(n::val_coord_2d(seed, ix, iz))
                );
            }
        }
    }
    let values: [f32; 10] = [-16.0, -1.0, -0.25, -0.0, 0.0, 0.125, 0.5, 0.999, 1.0, 16.0];
    for (i, x) in values.iter().enumerate() {
        let t = (i as f32 + 0.5) / values.len() as f32;
        println!(
            "math/{i}\t{},{},{},{},{},{},{},{},{},{}",
            n::floor_legacy(*x),
            n::round_legacy(*x),
            n::to_int(*x),
            bits(n::interp_hermite(t)),
            bits(n::interp_quintic(t)),
            bits(n::lerp_f32(-1.0, 1.0, t)),
            bits(n::cubic_lerp(-1.0, 0.0, 1.0, 2.0, t)),
            bits(n::map_clamped_range(*x, -2.0, 2.0, 4.0)),
            bits(n::curve(t, 3.0)),
            bits(n::pow_i32(t, 3))
        );
    }
    println!("legacy/floor-negative-integer\t{}", n::floor_legacy(-1.0));
}
