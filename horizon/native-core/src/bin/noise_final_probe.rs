use terraforged_native_core::{
    domain::Domain as D,
    erosion::{BlendMode as B, Erosion as E},
    graph::Noise as N,
    line::{self, Line},
    perlin::Interpolation as I,
    perlin2::Perlin2,
    simplex::Simplex,
};
fn c(v: f32) -> N {
    N::constant(v).unwrap()
}
fn s() -> N {
    N::simplex(Simplex::new(0.0125, 3, 2.0, 0.5, I::Curve3).unwrap()).unwrap()
}
fn p() -> N {
    N::perlin2(Perlin2::new(12345, 0.02, 2, 2.0, 0.5, I::Curve4).unwrap()).unwrap()
}
fn bits(f: f32) -> String {
    format!("{:08x}", f.to_bits())
}
fn d(index: usize) -> D {
    match index {
        0 => D::direct().unwrap(),
        1 => D::warp(s(), p(), c(10.0)).unwrap(),
        2 => D::warp(
            N::add(s(), c(-0.5)).unwrap(),
            N::add(p(), c(-0.5)).unwrap(),
            c(10.0),
        )
        .unwrap(),
        3 => D::direction(s(), c(16.0)).unwrap(),
        4 => D::warp(
            s(),
            p(),
            N::add(c(1.0), N::multiply(s(), c(8.0)).unwrap()).unwrap(),
        )
        .unwrap(),
        5 => D::add(d(1), d(3)).unwrap(),
        6 => D::compound(d(1), d(3)).unwrap(),
        7 => D::compound(d(3), d(1)).unwrap(),
        8 => D::add(D::compound(d(1), d(3)).unwrap(), d(4)).unwrap(),
        _ => panic!("Invalid domain test case"),
    }
}
fn main() {
    let graphs: Vec<(&str, N)> = vec![
        (
            "line-0.0F-1.0F",
            N::line(
                Line::new(-4.125, 2.25, 6.375, -7.75, 0.0),
                c(1.0),
                c(0.25),
                c(0.5),
            )
            .unwrap(),
        ),
        (
            "line-0.0F-20.0F",
            N::line(
                Line::new(-4.125, 2.25, 6.375, -7.75, 0.0),
                c(20.0),
                c(0.25),
                c(0.5),
            )
            .unwrap(),
        ),
        (
            "line-0.0F-100.0F",
            N::line(
                Line::new(-4.125, 2.25, 6.375, -7.75, 0.0),
                c(100.0),
                c(0.25),
                c(0.5),
            )
            .unwrap(),
        ),
        (
            "line-0.5F-1.0F",
            N::line(
                Line::new(-4.125, 2.25, 6.375, -7.75, 0.5),
                c(1.0),
                c(0.25),
                c(0.5),
            )
            .unwrap(),
        ),
        (
            "line-0.5F-20.0F",
            N::line(
                Line::new(-4.125, 2.25, 6.375, -7.75, 0.5),
                c(20.0),
                c(0.25),
                c(0.5),
            )
            .unwrap(),
        ),
        (
            "line-0.5F-100.0F",
            N::line(
                Line::new(-4.125, 2.25, 6.375, -7.75, 0.5),
                c(100.0),
                c(0.25),
                c(0.5),
            )
            .unwrap(),
        ),
        (
            "line-1.0F-1.0F",
            N::line(
                Line::new(-4.125, 2.25, 6.375, -7.75, 1.0),
                c(1.0),
                c(0.25),
                c(0.5),
            )
            .unwrap(),
        ),
        (
            "line-1.0F-20.0F",
            N::line(
                Line::new(-4.125, 2.25, 6.375, -7.75, 1.0),
                c(20.0),
                c(0.25),
                c(0.5),
            )
            .unwrap(),
        ),
        (
            "line-1.0F-100.0F",
            N::line(
                Line::new(-4.125, 2.25, 6.375, -7.75, 1.0),
                c(100.0),
                c(0.25),
                c(0.5),
            )
            .unwrap(),
        ),
        (
            "line-variable",
            N::line(
                Line::new(-4.125, 2.25, 6.375, -7.75, 0.5),
                N::add(c(1.0), N::multiply(s(), c(128.0)).unwrap()).unwrap(),
                s(),
                s(),
            )
            .unwrap(),
        ),
        (
            "terrace-2-0.0F",
            N::terrace(s(), c(0.5), c(0.75), c(0.6), 0.0, 2).unwrap(),
        ),
        (
            "terrace-2-0.25F",
            N::terrace(s(), c(0.5), c(0.75), c(0.6), 0.25, 2).unwrap(),
        ),
        (
            "terrace-2-1.0F",
            N::terrace(s(), c(0.5), c(0.75), c(0.6), 1.0, 2).unwrap(),
        ),
        (
            "terrace-2-1.5F",
            N::terrace(s(), c(0.5), c(0.75), c(0.6), 1.5, 2).unwrap(),
        ),
        (
            "terrace-4-0.0F",
            N::terrace(s(), c(0.5), c(0.75), c(0.6), 0.0, 4).unwrap(),
        ),
        (
            "terrace-4-0.25F",
            N::terrace(s(), c(0.5), c(0.75), c(0.6), 0.25, 4).unwrap(),
        ),
        (
            "terrace-4-1.0F",
            N::terrace(s(), c(0.5), c(0.75), c(0.6), 1.0, 4).unwrap(),
        ),
        (
            "terrace-4-1.5F",
            N::terrace(s(), c(0.5), c(0.75), c(0.6), 1.5, 4).unwrap(),
        ),
        (
            "terrace-8-0.0F",
            N::terrace(s(), c(0.5), c(0.75), c(0.6), 0.0, 8).unwrap(),
        ),
        (
            "terrace-8-0.25F",
            N::terrace(s(), c(0.5), c(0.75), c(0.6), 0.25, 8).unwrap(),
        ),
        (
            "terrace-8-1.0F",
            N::terrace(s(), c(0.5), c(0.75), c(0.6), 1.0, 8).unwrap(),
        ),
        (
            "terrace-8-1.5F",
            N::terrace(s(), c(0.5), c(0.75), c(0.6), 1.5, 8).unwrap(),
        ),
        (
            "terrace-16-0.0F",
            N::terrace(s(), c(0.5), c(0.75), c(0.6), 0.0, 16).unwrap(),
        ),
        (
            "terrace-16-0.25F",
            N::terrace(s(), c(0.5), c(0.75), c(0.6), 0.25, 16).unwrap(),
        ),
        (
            "terrace-16-1.0F",
            N::terrace(s(), c(0.5), c(0.75), c(0.6), 1.0, 16).unwrap(),
        ),
        (
            "terrace-16-1.5F",
            N::terrace(s(), c(0.5), c(0.75), c(0.6), 1.5, 16).unwrap(),
        ),
        (
            "terrace-variable",
            N::terrace(s(), s(), s(), s(), 0.25, 6).unwrap(),
        ),
        (
            "terrace-nonzero-bounds",
            N::terrace(N::add(s(), c(3.0)).unwrap(), s(), s(), s(), 0.25, 6).unwrap(),
        ),
        (
            "advanced-3-0-0.0F",
            N::advanced_terrace(s(), c(0.2), c(0.0), c(0.75), 0.1, 0.8, 3, 0).unwrap(),
        ),
        (
            "advanced-3-0-0.25F",
            N::advanced_terrace(s(), c(0.2), c(0.25), c(0.75), 0.1, 0.8, 3, 0).unwrap(),
        ),
        (
            "advanced-3-0-1.0F",
            N::advanced_terrace(s(), c(0.2), c(1.0), c(0.75), 0.1, 0.8, 3, 0).unwrap(),
        ),
        (
            "advanced-3-1-0.0F",
            N::advanced_terrace(s(), c(0.2), c(0.0), c(0.75), 0.1, 0.8, 3, 1).unwrap(),
        ),
        (
            "advanced-3-1-0.25F",
            N::advanced_terrace(s(), c(0.2), c(0.25), c(0.75), 0.1, 0.8, 3, 1).unwrap(),
        ),
        (
            "advanced-3-1-1.0F",
            N::advanced_terrace(s(), c(0.2), c(1.0), c(0.75), 0.1, 0.8, 3, 1).unwrap(),
        ),
        (
            "advanced-3-3-0.0F",
            N::advanced_terrace(s(), c(0.2), c(0.0), c(0.75), 0.1, 0.8, 3, 3).unwrap(),
        ),
        (
            "advanced-3-3-0.25F",
            N::advanced_terrace(s(), c(0.2), c(0.25), c(0.75), 0.1, 0.8, 3, 3).unwrap(),
        ),
        (
            "advanced-3-3-1.0F",
            N::advanced_terrace(s(), c(0.2), c(1.0), c(0.75), 0.1, 0.8, 3, 3).unwrap(),
        ),
        (
            "advanced-8-0-0.0F",
            N::advanced_terrace(s(), c(0.2), c(0.0), c(0.75), 0.1, 0.8, 8, 0).unwrap(),
        ),
        (
            "advanced-8-0-0.25F",
            N::advanced_terrace(s(), c(0.2), c(0.25), c(0.75), 0.1, 0.8, 8, 0).unwrap(),
        ),
        (
            "advanced-8-0-1.0F",
            N::advanced_terrace(s(), c(0.2), c(1.0), c(0.75), 0.1, 0.8, 8, 0).unwrap(),
        ),
        (
            "advanced-8-1-0.0F",
            N::advanced_terrace(s(), c(0.2), c(0.0), c(0.75), 0.1, 0.8, 8, 1).unwrap(),
        ),
        (
            "advanced-8-1-0.25F",
            N::advanced_terrace(s(), c(0.2), c(0.25), c(0.75), 0.1, 0.8, 8, 1).unwrap(),
        ),
        (
            "advanced-8-1-1.0F",
            N::advanced_terrace(s(), c(0.2), c(1.0), c(0.75), 0.1, 0.8, 8, 1).unwrap(),
        ),
        (
            "advanced-8-3-0.0F",
            N::advanced_terrace(s(), c(0.2), c(0.0), c(0.75), 0.1, 0.8, 8, 3).unwrap(),
        ),
        (
            "advanced-8-3-0.25F",
            N::advanced_terrace(s(), c(0.2), c(0.25), c(0.75), 0.1, 0.8, 8, 3).unwrap(),
        ),
        (
            "advanced-8-3-1.0F",
            N::advanced_terrace(s(), c(0.2), c(1.0), c(0.75), 0.1, 0.8, 8, 3).unwrap(),
        ),
        (
            "advanced-below",
            N::advanced_terrace(c(0.05), c(0.2), c(1.0), c(0.75), 0.1, 0.8, 4, 2).unwrap(),
        ),
        (
            "advanced-equal",
            N::advanced_terrace(c(0.1), c(0.2), c(1.0), c(0.75), 0.1, 0.8, 4, 2).unwrap(),
        ),
        (
            "advanced-mid",
            N::advanced_terrace(c(0.5), c(0.2), c(1.0), c(0.75), 0.1, 0.8, 4, 2).unwrap(),
        ),
        (
            "advanced-above",
            N::advanced_terrace(c(0.9), c(0.2), c(1.0), c(0.75), 0.1, 0.8, 4, 2).unwrap(),
        ),
        (
            "advanced-variable",
            N::advanced_terrace(
                s(),
                N::multiply(s(), c(0.25)).unwrap(),
                s(),
                s(),
                0.1,
                0.8,
                4,
                2,
            )
            .unwrap(),
        ),
        ("warp-0", N::warp(s(), d(0)).unwrap()),
        ("warp-1", N::warp(s(), d(1)).unwrap()),
        ("warp-2", N::warp(s(), d(2)).unwrap()),
        ("warp-3", N::warp(s(), d(3)).unwrap()),
        ("warp-4", N::warp(s(), d(4)).unwrap()),
        ("warp-5", N::warp(s(), d(5)).unwrap()),
        ("warp-6", N::warp(s(), d(6)).unwrap()),
        ("warp-7", N::warp(s(), d(7)).unwrap()),
        ("warp-8", N::warp(s(), d(8)).unwrap()),
        (
            "erosion-CONSTANT-0-1",
            N::erosion(
                s(),
                E::new(0, 1, 0.6, 80.0, 0.5, 2.0, 0.5, B::Constant).unwrap(),
            )
            .unwrap(),
        ),
        (
            "erosion-CONSTANT-0-3",
            N::erosion(
                s(),
                E::new(0, 3, 0.6, 80.0, 0.5, 2.0, 0.5, B::Constant).unwrap(),
            )
            .unwrap(),
        ),
        (
            "erosion-CONSTANT-42-1",
            N::erosion(
                s(),
                E::new(42, 1, 0.6, 80.0, 0.5, 2.0, 0.5, B::Constant).unwrap(),
            )
            .unwrap(),
        ),
        (
            "erosion-CONSTANT-42-3",
            N::erosion(
                s(),
                E::new(42, 3, 0.6, 80.0, 0.5, 2.0, 0.5, B::Constant).unwrap(),
            )
            .unwrap(),
        ),
        (
            "erosion-CONSTANT--2147483648-1",
            N::erosion(
                s(),
                E::new(-2147483648, 1, 0.6, 80.0, 0.5, 2.0, 0.5, B::Constant).unwrap(),
            )
            .unwrap(),
        ),
        (
            "erosion-CONSTANT--2147483648-3",
            N::erosion(
                s(),
                E::new(-2147483648, 3, 0.6, 80.0, 0.5, 2.0, 0.5, B::Constant).unwrap(),
            )
            .unwrap(),
        ),
        (
            "erosion-INPUT_LINEAR-0-1",
            N::erosion(
                s(),
                E::new(0, 1, 0.6, 80.0, 0.5, 2.0, 0.5, B::InputLinear).unwrap(),
            )
            .unwrap(),
        ),
        (
            "erosion-INPUT_LINEAR-0-3",
            N::erosion(
                s(),
                E::new(0, 3, 0.6, 80.0, 0.5, 2.0, 0.5, B::InputLinear).unwrap(),
            )
            .unwrap(),
        ),
        (
            "erosion-INPUT_LINEAR-42-1",
            N::erosion(
                s(),
                E::new(42, 1, 0.6, 80.0, 0.5, 2.0, 0.5, B::InputLinear).unwrap(),
            )
            .unwrap(),
        ),
        (
            "erosion-INPUT_LINEAR-42-3",
            N::erosion(
                s(),
                E::new(42, 3, 0.6, 80.0, 0.5, 2.0, 0.5, B::InputLinear).unwrap(),
            )
            .unwrap(),
        ),
        (
            "erosion-INPUT_LINEAR--2147483648-1",
            N::erosion(
                s(),
                E::new(-2147483648, 1, 0.6, 80.0, 0.5, 2.0, 0.5, B::InputLinear).unwrap(),
            )
            .unwrap(),
        ),
        (
            "erosion-INPUT_LINEAR--2147483648-3",
            N::erosion(
                s(),
                E::new(-2147483648, 3, 0.6, 80.0, 0.5, 2.0, 0.5, B::InputLinear).unwrap(),
            )
            .unwrap(),
        ),
        (
            "erosion-OUTPUT_LINEAR-0-1",
            N::erosion(
                s(),
                E::new(0, 1, 0.6, 80.0, 0.5, 2.0, 0.5, B::OutputLinear).unwrap(),
            )
            .unwrap(),
        ),
        (
            "erosion-OUTPUT_LINEAR-0-3",
            N::erosion(
                s(),
                E::new(0, 3, 0.6, 80.0, 0.5, 2.0, 0.5, B::OutputLinear).unwrap(),
            )
            .unwrap(),
        ),
        (
            "erosion-OUTPUT_LINEAR-42-1",
            N::erosion(
                s(),
                E::new(42, 1, 0.6, 80.0, 0.5, 2.0, 0.5, B::OutputLinear).unwrap(),
            )
            .unwrap(),
        ),
        (
            "erosion-OUTPUT_LINEAR-42-3",
            N::erosion(
                s(),
                E::new(42, 3, 0.6, 80.0, 0.5, 2.0, 0.5, B::OutputLinear).unwrap(),
            )
            .unwrap(),
        ),
        (
            "erosion-OUTPUT_LINEAR--2147483648-1",
            N::erosion(
                s(),
                E::new(-2147483648, 1, 0.6, 80.0, 0.5, 2.0, 0.5, B::OutputLinear).unwrap(),
            )
            .unwrap(),
        ),
        (
            "erosion-OUTPUT_LINEAR--2147483648-3",
            N::erosion(
                s(),
                E::new(-2147483648, 3, 0.6, 80.0, 0.5, 2.0, 0.5, B::OutputLinear).unwrap(),
            )
            .unwrap(),
        ),
    ];
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
    for (n, (name, g)) in graphs.into_iter().enumerate() {
        println!(
            "graph/{n}\t{name},{},{}",
            bits(g.min_value()),
            bits(g.max_value())
        );
        for seed in seeds {
            for (i, x) in coords.into_iter().enumerate() {
                for (j, z) in coords.into_iter().enumerate() {
                    println!("sample/{n}/{seed}/{i}/{j}\t{}", bits(g.compute(x, z, seed)));
                }
            }
        }
    }
    for n in 0..9 {
        let domain = d(n);
        for seed in seeds {
            for (i, x) in coords.into_iter().enumerate() {
                for (j, z) in coords.into_iter().enumerate() {
                    println!(
                        "domain/{n}/{seed}/{i}/{j}\t{},{},{},{}",
                        bits(domain.offset_x(x, z, seed)),
                        bits(domain.offset_z(x, z, seed)),
                        bits(domain.get_x(x, z, seed)),
                        bits(domain.get_z(x, z, seed))
                    );
                }
            }
        }
    }
    for (i, x) in coords.into_iter().enumerate() {
        for (j, z) in coords.into_iter().enumerate() {
            println!(
                "line-helper/{i}/{j}\t{},{},{}",
                bits(line::dist_sq(x, z, -4.125, 2.25)),
                bits(line::distance_on_line(x, z, -4.125, 2.25, 6.375, -7.75)),
                line::intersect(x, z, 6.375, -7.75, -1.0, 0.0, 1.0, 0.0)
            );
        }
    }
    let line = N::line(
        Line::new(-4.125, 2.25, 6.375, -7.75, 0.5),
        c(100.0),
        c(0.25),
        c(0.5),
    )
    .unwrap();
    println!(
        "nonfinite/line-start\t{}",
        line.compute(-4.125, 2.25, 42).is_nan()
    );
    println!(
        "nonfinite/line-end\t{}",
        line.compute(6.375, -7.75, 42).is_nan()
    );
    let erosion0 = N::erosion(
        s(),
        E::new(42, 0, 0.6, 80.0, 0.5, 2.0, 0.5, B::Constant).unwrap(),
    )
    .unwrap();
    println!(
        "nonfinite/erosion-zero-octaves\t{}",
        erosion0.compute(0.0, 0.0, 42).is_nan()
    );
    assert!(N::terrace(s(), s(), s(), s(), 0.25, 1).is_err());
    assert!(N::terrace(s(), s(), s(), s(), 0.25, 4097).is_err());
    assert!(N::advanced_terrace(s(), s(), s(), s(), 0.1, 0.8, 4, 33).is_err());
    assert!(E::new(42, 33, 0.6, 80.0, 0.5, 2.0, 0.5, B::Constant).is_err());
    let mut nested = D::direct().unwrap();
    let mut depth_guard = false;
    for _ in 0..300 {
        match D::add(nested.clone(), D::direct().unwrap()) {
            Ok(next) => nested = next,
            Err(terraforged_native_core::graph::GraphError::DepthLimit) => {
                depth_guard = true;
                break;
            }
            Err(e) => panic!("Unexpected guard {e:?}"),
        }
    }
    assert!(depth_guard);
    assert!(N::warp(s(), nested).is_err());
    eprintln!("NATIVE_FINAL_GUARDS_OK terrace=2..4096 octaves=0..32 combinedDomainDepth=256");
}
