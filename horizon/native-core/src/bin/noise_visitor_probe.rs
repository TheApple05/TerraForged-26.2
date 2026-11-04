use terraforged_native_core::{
    domain::Domain as D,
    erosion::{BlendMode as B, Erosion as E},
    graph::{CellFunction as F, CurveFunction as C, GraphError, Noise as N},
    line::Line,
    perlin::{Interpolation as I, Perlin},
    perlin2::Perlin2,
    ridges::{Billow, PerlinRidge, SimplexRidge},
    simplex::Simplex,
    simplex2::Simplex2,
    sources::{Cubic, White},
    worley::{DistanceFunction as DF, EdgeFunction as EF},
};
fn c(f: f32) -> N {
    N::constant(f).unwrap()
}
fn s() -> N {
    N::simplex(Simplex::new(0.0125, 3, 2.0, 0.5, I::Curve3).unwrap()).unwrap()
}
fn p() -> N {
    N::perlin2(Perlin2::new(12345, 0.02, 2, 2.0, 0.5, I::Curve4).unwrap()).unwrap()
}
fn a() -> N {
    N::add(s(), c(0.125)).unwrap()
}
fn b() -> N {
    N::multiply(p(), c(0.5)).unwrap()
}
fn bits(f: f32) -> String {
    format!("{:08x}", f.to_bits())
}
fn graphs() -> Vec<(&'static str, N)> {
    vec![
        ("constant", c(0.5)),
        (
            "Perlin",
            N::perlin(Perlin::new(12345, 0.0125, 3, 2.0, 0.5, I::Curve3).unwrap()).unwrap(),
        ),
        (
            "Perlin2",
            N::perlin2(Perlin2::new(12345, 0.0125, 3, 2.0, 0.5, I::Curve3).unwrap()).unwrap(),
        ),
        (
            "Simplex",
            N::simplex(Simplex::new(0.0125, 3, 2.0, 0.5, I::Curve3).unwrap()).unwrap(),
        ),
        (
            "Simplex2",
            N::simplex2(Simplex2::new(0.0125, 3, 2.0, 0.5, I::Curve3).unwrap()).unwrap(),
        ),
        ("White", N::white(White::new(0.0125)).unwrap()),
        (
            "Cubic",
            N::cubic(Cubic::new(0.0125, 3, 2.0, 0.5).unwrap()).unwrap(),
        ),
        (
            "PerlinRidge",
            N::perlin_ridge(PerlinRidge::new(0.0125, 3, 2.0, 0.5, I::Curve3).unwrap()).unwrap(),
        ),
        (
            "SimplexRidge",
            N::simplex_ridge(SimplexRidge::new(0.0125, 3, 2.0, 0.5, I::Curve3).unwrap()).unwrap(),
        ),
        (
            "Billow",
            N::billow(Billow::new(0.0125, 3, 2.0, 0.5, I::Curve3).unwrap()).unwrap(),
        ),
        (
            "Worley",
            N::worley(0.0125, 1.0, F::NoiseLookup, DF::Natural, a()).unwrap(),
        ),
        (
            "WorleyEdge",
            N::worley_edge(0.0125, 1.0, EF::Distance2Add, DF::Euclidean).unwrap(),
        ),
        ("Sin", N::sin(0.0125, a()).unwrap()),
        ("ShiftSeed", N::shift_seed(a(), 12345).unwrap()),
        ("Frequency", N::frequency(a(), b(), s()).unwrap()),
        ("Add", N::add(a(), b()).unwrap()),
        ("Multiply", N::multiply(a(), b()).unwrap()),
        ("Power", N::power(a(), 1.5).unwrap()),
        ("PowerCurve", N::power_curve(a(), 1.5).unwrap()),
        ("Curve", N::curve(a(), C::scurve(2.0, -0.5)).unwrap()),
        ("Gradient", N::gradient(a(), c(0.25), c(0.75), b()).unwrap()),
        (
            "Terrace",
            N::terrace(a(), b(), s(), c(0.75), 0.25, 6).unwrap(),
        ),
        (
            "AdvancedTerrace",
            N::advanced_terrace(a(), b(), s(), c(0.75), 0.1, 0.8, 6, 2).unwrap(),
        ),
        ("Invert", N::invert(a()).unwrap()),
        (
            "Blend",
            N::blend(a(), b(), s(), 0.5, 0.75, I::Curve3).unwrap(),
        ),
        ("Alpha", N::alpha(a(), b()).unwrap()),
        ("Boost", N::boost(a(), 2).unwrap()),
        (
            "Steps",
            N::steps(a(), c(4.0), c(0.1), c(0.9), C::Interpolation(I::Curve3)).unwrap(),
        ),
        ("Abs", N::abs(N::add(a(), c(-1.0)).unwrap()).unwrap()),
        ("Map", N::map(a(), b(), c(2.0)).unwrap()),
        ("Clamp", N::clamp(a(), b(), c(0.9)).unwrap()),
        ("Threshold", N::threshold(a(), b(), s(), c(0.5)).unwrap()),
        ("Min", N::minimum(a(), b()).unwrap()),
        ("Max", N::maximum(a(), b()).unwrap()),
        (
            "Warp",
            N::warp(a(), D::warp(b(), s(), c(16.0)).unwrap()).unwrap(),
        ),
        (
            "Erosion",
            N::erosion(
                a(),
                E::new(42, 1, 0.6, 80.0, 0.5, 2.0, 0.5, B::Constant).unwrap(),
            )
            .unwrap(),
        ),
        (
            "Line",
            N::line(Line::new(-4.125, 2.25, 6.375, -7.75, 0.5), a(), b(), s()).unwrap(),
        ),
        (
            "LinearSpline",
            N::linear_spline(
                a(),
                vec![(0.0, b()), (0.5, N::abs(s()).unwrap()), (1.0, a())],
            )
            .unwrap(),
        ),
        ("Cache2d", N::cache(a()).unwrap()),
        (
            "LegacyTemperature",
            N::legacy_temperature(0.0125, 3).unwrap(),
        ),
        ("LegacyMoisture", N::legacy_moisture(a(), 3).unwrap()),
        ("HolderHolder", N::holder(a()).unwrap()),
        ("Abs-nested", N::abs(N::abs(a()).unwrap()).unwrap()),
        (
            "Min-PowerCurve",
            N::minimum(N::power_curve(a(), 2.0).unwrap(), N::abs(b()).unwrap()).unwrap(),
        ),
        ("Warp-direct", N::warp(a(), D::direct().unwrap()).unwrap()),
        (
            "Warp-direction",
            N::warp(a(), D::direction(b(), s()).unwrap()).unwrap(),
        ),
        (
            "Warp-add",
            N::warp(
                a(),
                D::add(
                    D::direction(b(), s()).unwrap(),
                    D::warp(a(), b(), c(10.0)).unwrap(),
                )
                .unwrap(),
            )
            .unwrap(),
        ),
        (
            "Warp-compound",
            N::warp(
                a(),
                D::compound(
                    D::warp(a(), b(), c(10.0)).unwrap(),
                    D::direction(b(), s()).unwrap(),
                )
                .unwrap(),
            )
            .unwrap(),
        ),
    ]
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
    for mode in 0..3 {
        for (n, (name, graph)) in graphs().into_iter().enumerate() {
            let mut trace = Vec::new();
            let result = graph
                .map_all(&mut |input: N| -> Result<N, GraphError> {
                    trace.push(input.kind());
                    let count = trace.len();
                    if mode == 1 {
                        return N::add(input, c(0.03125));
                    }
                    if mode == 2 {
                        if let Some(value) = input.constant_value() {
                            return N::constant(value + ((count % 7) as i32 - 3) as f32 * 0.015625);
                        }
                        if matches!(input.kind(), "Simplex" | "Perlin2") {
                            return N::shift_seed(input, count as i32);
                        }
                    }
                    Ok(input)
                })
                .unwrap();
            println!(
                "graph/{mode}/{n}\t{name},{},{},{}",
                result.kind(),
                bits(result.min_value()),
                bits(result.max_value())
            );
            println!("trace/{mode}/{n}\t{}", trace.join("|"));
            for seed in seeds {
                for (i, x) in coords.into_iter().enumerate() {
                    for (j, z) in coords.into_iter().enumerate() {
                        println!(
                            "sample/{mode}/{n}/{seed}/{i}/{j}\t{}",
                            bits(result.compute(x, z, seed))
                        );
                    }
                }
            }
        }
    }
}
