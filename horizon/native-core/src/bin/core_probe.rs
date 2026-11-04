use terraforged_native_core::noise_util as n;
mod utilities {
    include!("utility_probe.rs");
    pub fn emit() {
        main();
    }
}
fn bits(value: f32) -> String {
    format!("{:08x}", value.to_bits())
}

fn main() {
    let seeds = [0, 1, -1, 42, i32::MIN, i32::MAX];
    let coords: [f32; 17] = [
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
        for (i, x) in coords.iter().enumerate() {
            for (j, z) in coords.iter().enumerate() {
                let ix = n::floor_legacy(*x);
                let iz = n::floor_legacy(*z);
                let dx = x - ix as f32;
                let dz = z - iz as f32;
                let cell = n::cell(seed, ix, iz);
                let gradient = n::coord_2d(seed, ix, iz);
                let gradient24 = n::coord_2d_24(seed, ix, iz);
                println!(
                    "noise/{seed}/{i}/{j}\t{ix},{iz},{},{},{},{},{},{},{},{},{},{},{},{}",
                    n::hash(ix, iz),
                    n::hash_2d(seed, ix, iz),
                    n::seed(ix, iz),
                    bits(n::val_coord_2d(seed, ix, iz)),
                    bits(n::grad_coord_2d(seed, ix, iz, dx, dz)),
                    bits(n::grad_coord_2d_24(seed, ix, iz, dx, dz)),
                    bits(cell.0),
                    bits(cell.1),
                    bits(gradient.0),
                    bits(gradient.1),
                    bits(gradient24.0),
                    bits(gradient24.1)
                );
            }
        }
    }
    let values: [f32; 10] = [-16.0, -1.0, -0.25, -0.0, 0.0, 0.125, 0.5, 0.999, 1.0, 16.0];
    for (i, x) in values.iter().enumerate() {
        let t = (i as f32 + 0.5) / values.len() as f32;
        println!(
            "math/{i}\t{},{},{},{},{},{},{},{},{},{},{},{},{}",
            n::floor_legacy(*x),
            n::round_legacy(*x),
            n::to_int(*x),
            bits(n::sin(*x)),
            bits(n::cos(*x)),
            bits(n::interp_hermite(t)),
            bits(n::interp_quintic(t)),
            bits(n::lerp_f32(-1.0, 1.0, t)),
            bits(n::cubic_lerp(-1.0, 0.0, 1.0, 2.0, t)),
            bits(n::map_clamped_range(*x, -2.0, 2.0, 4.0)),
            bits(n::curve(t, 3.0)),
            bits(n::pow_i32(t, 3)),
            bits(n::sqrt(t))
        );
    }
    println!("legacy/floor-negative-integer\t{}", n::floor_legacy(-1.0));
    utilities::emit();
}
