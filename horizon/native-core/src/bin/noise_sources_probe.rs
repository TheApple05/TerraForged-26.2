use terraforged_native_core::{
    perlin::Interpolation,
    ridges::{Billow, PerlinRidge, SimplexRidge},
    sources::{Cubic, White},
};
fn bits(f: f32) -> String {
    format!("{:08x}", f.to_bits())
}
fn meta(key: &str, min: f32, max: f32) {
    println!("meta/{key}\t{},{}", bits(min), bits(max));
}
fn main() {
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
    let params = [
        (1f32, 0, 2f32, 0.5f32),
        (1., 1, 2., 0.5),
        (0.0125, 3, 2., 0.5),
        (0.001, 7, 2.5, 0.75),
        (0.0000001, 30, 1.1, 0.975),
        (0.0000001, 31, 1.1, 0.975),
        (0.0000001, 32, 1.1, 0.975),
    ];
    for (p, (f, o, l, g)) in params.into_iter().enumerate() {
        let white = White::new(f);
        let cubic = Cubic::new(f, o, l, g).unwrap();
        meta(&format!("white/{p}"), white.min_value(), white.max_value());
        meta(&format!("cubic/{p}"), cubic.min_value(), cubic.max_value());
        for (mode, interpolation) in [
            Interpolation::Linear,
            Interpolation::Curve3,
            Interpolation::Curve4,
        ]
        .into_iter()
        .enumerate()
        {
            let perlin = PerlinRidge::new(f, o, l, g, interpolation).unwrap();
            let billow = Billow::new(f, o, l, g, interpolation).unwrap();
            let simplex = if p < 5 {
                Some(SimplexRidge::new(f, o, l, g, interpolation).unwrap())
            } else {
                None
            };
            let prefix = format!("{mode}/{p}");
            meta(
                &format!("perlin_ridge/{prefix}"),
                perlin.min_value(),
                perlin.max_value(),
            );
            meta(
                &format!("billow/{prefix}"),
                billow.min_value(),
                billow.max_value(),
            );
            if let Some(ref n) = simplex {
                meta(
                    &format!("simplex_ridge/{prefix}"),
                    n.min_value(),
                    n.max_value(),
                );
            }
            for seed in seeds {
                for (i, x) in coords.into_iter().enumerate() {
                    for (j, z) in coords.into_iter().enumerate() {
                        let key = format!("{prefix}/{seed}/{i}/{j}");
                        println!(
                            "perlin_ridge/compute/{key}\t{}",
                            bits(perlin.compute(x, z, seed))
                        );
                        println!("billow/compute/{key}\t{}", bits(billow.compute(x, z, seed)));
                        if let Some(ref n) = simplex {
                            println!(
                                "simplex_ridge/compute/{key}\t{}",
                                bits(n.compute(x, z, seed))
                            );
                        }
                        if mode == 0 {
                            let key = format!("{p}/{seed}/{i}/{j}");
                            println!("white/compute/{key}\t{}", bits(white.compute(x, z, seed)));
                            println!("cubic/compute/{key}\t{}", bits(cubic.compute(x, z, seed)));
                            if p == 0 {
                                println!(
                                    "white/sample/{key}\t{},{}",
                                    bits(White::sample(x, z, seed)),
                                    bits(White::sample_offset(x, z, seed, i32::MAX))
                                );
                                println!("cubic/sample/{key}\t{}", bits(Cubic::sample(x, z, seed)));
                            }
                        }
                    }
                }
            }
        }
    }
}
