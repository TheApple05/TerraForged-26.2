use terraforged_native_core::{
    noise_vectors as v, perlin::Interpolation, perlin2::Perlin2 as Perlin,
    simplex2::Simplex2 as Simplex,
};

fn bits(value: f32) -> String {
    format!("{:08x}", value.to_bits())
}

fn main() {
    for (name, values) in [
        ("grad", &v::GRAD_2D[..]),
        ("grad24", &v::GRAD_2D_24[..]),
        ("cell", &v::CELL_2D[..]),
    ] {
        for (i, (x, y)) in values.iter().enumerate() {
            println!("vectors/{name}/{i}\t{},{}", bits(*x), bits(*y));
        }
    }
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
    let params = [
        (1.0f32, 0, 2.0f32, 0.5f32),
        (1.0, 1, 2.0, 0.5),
        (0.0125, 3, 2.0, 0.5),
        (0.001, 7, 2.5, 0.75),
    ];
    for seed in seeds {
        for (i, x) in coords.iter().enumerate() {
            for (j, z) in coords.iter().enumerate() {
                let key = format!("{seed}/{i}/{j}");
                for (mode, interpolation) in [
                    Interpolation::Linear,
                    Interpolation::Curve3,
                    Interpolation::Curve4,
                ]
                .iter()
                .enumerate()
                {
                    println!(
                        "perlin2/sample/{mode}/{key}\t{}",
                        bits(Perlin::sample(*x, *z, seed, *interpolation))
                    );
                    for (p, (frequency, octaves, lacunarity, gain)) in params.iter().enumerate() {
                        let n = Perlin::new(
                            seed,
                            *frequency,
                            *octaves,
                            *lacunarity,
                            *gain,
                            *interpolation,
                        )
                        .unwrap();
                        println!(
                            "perlin2/compute/{mode}/{p}/{key}\t{},{}",
                            bits(n.compute(*x, *z, seed)),
                            bits(n.compute(*x, *z, seed.wrapping_add(104729)))
                        );
                    }
                }
                println!(
                    "simplex2/sample/{key}\t{}",
                    bits(Simplex::sample(*x, *z, seed))
                );
                for (p, (frequency, octaves, lacunarity, gain)) in params.iter().enumerate() {
                    let n = Simplex::new(
                        *frequency,
                        *octaves,
                        *lacunarity,
                        *gain,
                        Interpolation::Curve4,
                    )
                    .unwrap();
                    println!(
                        "simplex2/compute/{p}/{key}\t{}",
                        bits(n.compute(*x, *z, seed))
                    );
                }
            }
        }
    }
}
