use terraforged_native_core::{
    graph::{CurveFunction as C, Noise as N},
    perlin::Interpolation as I,
    ridges::{Billow, PerlinRidge, SimplexRidge},
    simplex::Simplex,
    sources::{Cubic, White},
};
fn c(v: f32) -> N {
    N::constant(v).unwrap()
}
fn s() -> N {
    N::simplex(Simplex::new(0.0125, 3, 2.0, 0.5, I::Curve3).unwrap()).unwrap()
}
fn bits(f: f32) -> String {
    format!("{:08x}", f.to_bits())
}
fn main() {
    let graphs: Vec<(&str, N)> = vec![
        ("sin-0.0F-0.0F", N::sin(0.0, c(0.0)).unwrap()),
        ("sin-0.0F--0.0F", N::sin(0.0, c(-0.0)).unwrap()),
        ("sin-0.0F-1.0F", N::sin(0.0, c(1.0)).unwrap()),
        ("sin-0.0F-0.25F", N::sin(0.0, c(0.25)).unwrap()),
        ("sin-0.0F--0.25F", N::sin(0.0, c(-0.25)).unwrap()),
        ("sin-0.0F-1.25F", N::sin(0.0, c(1.25)).unwrap()),
        ("sin-0.125F-0.0F", N::sin(0.125, c(0.0)).unwrap()),
        ("sin-0.125F--0.0F", N::sin(0.125, c(-0.0)).unwrap()),
        ("sin-0.125F-1.0F", N::sin(0.125, c(1.0)).unwrap()),
        ("sin-0.125F-0.25F", N::sin(0.125, c(0.25)).unwrap()),
        ("sin-0.125F--0.25F", N::sin(0.125, c(-0.25)).unwrap()),
        ("sin-0.125F-1.25F", N::sin(0.125, c(1.25)).unwrap()),
        ("sin--0.25F-0.0F", N::sin(-0.25, c(0.0)).unwrap()),
        ("sin--0.25F--0.0F", N::sin(-0.25, c(-0.0)).unwrap()),
        ("sin--0.25F-1.0F", N::sin(-0.25, c(1.0)).unwrap()),
        ("sin--0.25F-0.25F", N::sin(-0.25, c(0.25)).unwrap()),
        ("sin--0.25F--0.25F", N::sin(-0.25, c(-0.25)).unwrap()),
        ("sin--0.25F-1.25F", N::sin(-0.25, c(1.25)).unwrap()),
        ("sin-dynamic", N::sin(0.0125, s()).unwrap()),
        ("power--0.5F", N::power(s(), -0.5).unwrap()),
        ("power-0.0F", N::power(s(), 0.0).unwrap()),
        ("power-0.125F", N::power(s(), 0.125).unwrap()),
        ("power-0.5F", N::power(s(), 0.5).unwrap()),
        ("power-1.25F", N::power(s(), 1.25).unwrap()),
        ("power-2.0F", N::power(s(), 2.0).unwrap()),
        ("power-3.0F", N::power(s(), 3.0).unwrap()),
        (
            "power-negative-2.0F",
            N::power(N::add(s(), c(-2.0)).unwrap(), 2.0).unwrap(),
        ),
        (
            "power-negative-3.0F",
            N::power(N::add(s(), c(-2.0)).unwrap(), 3.0).unwrap(),
        ),
        ("powercurve-unit-0.5F", N::power_curve(s(), 0.5).unwrap()),
        (
            "powercurve-shift-0.5F",
            N::power_curve(N::add(s(), c(3.0)).unwrap(), 0.5).unwrap(),
        ),
        (
            "powercurve-constant-0.5F",
            N::power_curve(c(-0.0), 0.5).unwrap(),
        ),
        ("powercurve-unit-1.0F", N::power_curve(s(), 1.0).unwrap()),
        (
            "powercurve-shift-1.0F",
            N::power_curve(N::add(s(), c(3.0)).unwrap(), 1.0).unwrap(),
        ),
        (
            "powercurve-constant-1.0F",
            N::power_curve(c(-0.0), 1.0).unwrap(),
        ),
        ("powercurve-unit-2.0F", N::power_curve(s(), 2.0).unwrap()),
        (
            "powercurve-shift-2.0F",
            N::power_curve(N::add(s(), c(3.0)).unwrap(), 2.0).unwrap(),
        ),
        (
            "powercurve-constant-2.0F",
            N::power_curve(c(-0.0), 2.0).unwrap(),
        ),
        ("powercurve-unit-3.0F", N::power_curve(s(), 3.0).unwrap()),
        (
            "powercurve-shift-3.0F",
            N::power_curve(N::add(s(), c(3.0)).unwrap(), 3.0).unwrap(),
        ),
        (
            "powercurve-constant-3.0F",
            N::power_curve(c(-0.0), 3.0).unwrap(),
        ),
        (
            "curve-LINEAR",
            N::curve(s(), C::Interpolation(I::Linear)).unwrap(),
        ),
        (
            "curve-CURVE3",
            N::curve(s(), C::Interpolation(I::Curve3)).unwrap(),
        ),
        (
            "curve-CURVE4",
            N::curve(s(), C::Interpolation(I::Curve4)).unwrap(),
        ),
        (
            "scurve-0.5F-2.0F",
            N::curve(s(), C::scurve(0.5, 2.0)).unwrap(),
        ),
        (
            "scurve-2.0F--3.0F",
            N::curve(s(), C::scurve(2.0, -3.0)).unwrap(),
        ),
        (
            "scurve-2.0F--0.5F",
            N::curve(s(), C::scurve(2.0, -0.5)).unwrap(),
        ),
        (
            "gradient-dynamic",
            N::gradient(s(), c(0.25), c(0.75), c(0.6)).unwrap(),
        ),
        (
            "gradient-below",
            N::gradient(c(0.1), c(0.25), c(0.75), c(0.6)).unwrap(),
        ),
        (
            "gradient-lower",
            N::gradient(c(0.25), c(0.25), c(0.75), c(0.6)).unwrap(),
        ),
        (
            "gradient-upper",
            N::gradient(c(0.75), c(0.25), c(0.75), c(0.6)).unwrap(),
        ),
        (
            "gradient-above",
            N::gradient(c(0.9), c(0.25), c(0.75), c(0.6)).unwrap(),
        ),
        (
            "gradient-variable",
            N::gradient(
                s(),
                N::multiply(s(), c(0.5)).unwrap(),
                N::add(c(0.5), N::multiply(s(), c(0.5)).unwrap()).unwrap(),
                s(),
            )
            .unwrap(),
        ),
        ("alpha-0.0F", N::alpha(s(), c(0.0)).unwrap()),
        ("alpha-1.0F", N::alpha(s(), c(1.0)).unwrap()),
        ("alpha-0.25F", N::alpha(s(), c(0.25)).unwrap()),
        ("alpha--0.25F", N::alpha(s(), c(-0.25)).unwrap()),
        ("alpha-1.25F", N::alpha(s(), c(1.25)).unwrap()),
        (
            "alpha-variable",
            N::alpha(N::add(s(), c(-2.0)).unwrap(), s()).unwrap(),
        ),
        ("boost--3", N::boost(s(), -3).unwrap()),
        ("boost-0", N::boost(s(), 0).unwrap()),
        ("boost-1", N::boost(s(), 1).unwrap()),
        ("boost-2", N::boost(s(), 2).unwrap()),
        ("boost-5", N::boost(s(), 5).unwrap()),
        (
            "boost-remap",
            N::boost(N::add(s(), c(3.0)).unwrap(), 3).unwrap(),
        ),
        (
            "steps-LINEAR",
            N::steps(s(), c(4.0), c(0.1), c(0.9), C::Interpolation(I::Linear)).unwrap(),
        ),
        (
            "steps-CURVE3",
            N::steps(s(), c(4.0), c(0.1), c(0.9), C::Interpolation(I::Curve3)).unwrap(),
        ),
        (
            "steps-CURVE4",
            N::steps(s(), c(4.0), c(0.1), c(0.9), C::Interpolation(I::Curve4)).unwrap(),
        ),
        (
            "steps-flat",
            N::steps(s(), c(4.0), c(0.0), c(0.0), C::Interpolation(I::Curve3)).unwrap(),
        ),
        (
            "steps-reverse",
            N::steps(s(), c(2.5), c(0.8), c(0.2), C::Interpolation(I::Curve3)).unwrap(),
        ),
        (
            "steps-fractional",
            N::steps(s(), c(2.5), c(0.1), c(0.9), C::Interpolation(I::Curve3)).unwrap(),
        ),
        (
            "steps-negative",
            N::steps(s(), c(-4.0), c(0.1), c(0.9), C::Interpolation(I::Curve3)).unwrap(),
        ),
        (
            "steps-scurve",
            N::steps(s(), c(3.5), c(0.1), c(0.9), C::scurve(2.0, -0.5)).unwrap(),
        ),
        (
            "threshold-dynamic",
            N::threshold(s(), c(-2.0), N::add(s(), c(3.0)).unwrap(), c(0.5)).unwrap(),
        ),
        (
            "threshold-equal",
            N::threshold(c(0.5), c(-2.0), N::add(s(), c(3.0)).unwrap(), c(0.5)).unwrap(),
        ),
        (
            "threshold-below",
            N::threshold(c(0.25), c(-2.0), N::add(s(), c(3.0)).unwrap(), c(0.5)).unwrap(),
        ),
        (
            "threshold-above",
            N::threshold(c(0.75), c(-2.0), N::add(s(), c(3.0)).unwrap(), c(0.5)).unwrap(),
        ),
        (
            "blend-LINEAR",
            N::blend(
                s(),
                c(-2.0),
                N::add(s(), c(3.0)).unwrap(),
                0.4,
                0.5,
                I::Linear,
            )
            .unwrap(),
        ),
        (
            "blend-CURVE3",
            N::blend(
                s(),
                c(-2.0),
                N::add(s(), c(3.0)).unwrap(),
                0.4,
                0.5,
                I::Curve3,
            )
            .unwrap(),
        ),
        (
            "blend-CURVE4",
            N::blend(
                s(),
                c(-2.0),
                N::add(s(), c(3.0)).unwrap(),
                0.4,
                0.5,
                I::Curve4,
            )
            .unwrap(),
        ),
        (
            "blend-shifted",
            N::blend(
                N::add(s(), c(3.0)).unwrap(),
                s(),
                c(-2.0),
                0.9,
                2.0,
                I::Curve3,
            )
            .unwrap(),
        ),
        ("white-graph", N::white(White::new(0.0125)).unwrap()),
        (
            "cubic-graph",
            N::cubic(Cubic::new(0.0125, 3, 2.0, 0.5).unwrap()).unwrap(),
        ),
        (
            "PerlinRidge-graph",
            N::perlin_ridge(PerlinRidge::new(0.0125, 3, 2.0, 0.5, I::Curve3).unwrap()).unwrap(),
        ),
        (
            "SimplexRidge-graph",
            N::simplex_ridge(SimplexRidge::new(0.0125, 3, 2.0, 0.5, I::Curve3).unwrap()).unwrap(),
        ),
        (
            "Billow-graph",
            N::billow(Billow::new(0.0125, 3, 2.0, 0.5, I::Curve3).unwrap()).unwrap(),
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
    assert!(matches!(
        N::boost(s(), 1025),
        Err(terraforged_native_core::graph::GraphError::IterationLimit)
    ));
    eprintln!("NATIVE_SHAPING_ITERATION_GUARD_OK max=1024");
}
