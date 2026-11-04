use terraforged_native_core::{
    graph::{GraphError, Noise},
    perlin::Interpolation,
    perlin2::Perlin2,
    simplex::Simplex,
};

fn c(v: f32) -> Noise {
    Noise::constant(v).unwrap()
}
fn simplex() -> Noise {
    Noise::simplex(Simplex::new(0.0125, 3, 2.0, 0.5, Interpolation::Curve3).unwrap()).unwrap()
}
fn perlin() -> Noise {
    Noise::perlin2(Perlin2::new(12345, 0.02, 2, 2.0, 0.5, Interpolation::Curve4).unwrap()).unwrap()
}
fn add(a: Noise, b: Noise) -> Noise {
    Noise::add(a, b).unwrap()
}
fn mul(a: Noise, b: Noise) -> Noise {
    Noise::multiply(a, b).unwrap()
}
fn abs(n: Noise) -> Noise {
    Noise::abs(n).unwrap()
}
fn invert(n: Noise) -> Noise {
    Noise::invert(n).unwrap()
}
fn min(a: Noise, b: Noise) -> Noise {
    Noise::minimum(a, b).unwrap()
}
fn max(a: Noise, b: Noise) -> Noise {
    Noise::maximum(a, b).unwrap()
}
fn clamp(n: Noise, lo: Noise, hi: Noise) -> Noise {
    Noise::clamp(n, lo, hi).unwrap()
}
fn map(n: Noise, from: Noise, to: Noise) -> Noise {
    Noise::map(n, from, to).unwrap()
}
fn shift(n: Noise, amount: i32) -> Noise {
    Noise::shift_seed(n, amount).unwrap()
}
fn freq(n: Noise, x: Noise, z: Noise) -> Noise {
    Noise::frequency(n, x, z).unwrap()
}
fn bits(v: f32) -> String {
    format!("{:08x}", v.to_bits())
}

fn graphs() -> Vec<(&'static str, Noise)> {
    vec![
        ("constant-poszero", c(0.0)),
        ("constant-negzero", c(-0.0)),
        ("constant-1", c(1.0)),
        ("constant-neg1", c(-1.0)),
        ("constant-neg2", c(-2.0)),
        ("constant-3", c(3.0)),
        ("simplex-base", simplex()),
        ("perlin-base", perlin()),
        ("simplex-negfreq", freq(simplex(), c(-1.5), c(0.75))),
        (
            "perlin-varfreq",
            freq(perlin(), add(c(0.5), mul(simplex(), c(0.5))), c(-0.25)),
        ),
        (
            "simplex-both-varfreq",
            freq(simplex(), add(c(0.125), perlin()), add(c(-0.25), simplex())),
        ),
        ("add-posneg", add(simplex(), c(-2.0))),
        ("add-signedzero", add(perlin(), c(-0.0))),
        ("mul-neg-range", mul(simplex(), c(-1.75))),
        ("mul-signedzero-lhs", mul(c(-0.0), simplex())),
        ("mul-signedzero-rhs", mul(simplex(), c(-0.0))),
        ("abs-simple", abs(add(simplex(), c(-1.0)))),
        ("abs-nested", abs(abs(mul(perlin(), c(2.0))))),
        ("invert-simple-nonzero", invert(add(simplex(), c(3.0)))),
        (
            "invert-nested-nonzero",
            invert(invert(add(perlin(), c(2.0)))),
        ),
        ("min-signedzero", min(c(0.0), c(-0.0))),
        ("min-signedzero-rev", min(c(-0.0), c(0.0))),
        ("max-signedzero", max(c(0.0), c(-0.0))),
        ("max-signedzero-rev", max(c(-0.0), c(0.0))),
        ("min-mixed", min(simplex(), c(-0.5))),
        ("max-mixed", max(perlin(), c(-0.5))),
        ("shiftseed-max", shift(perlin(), i32::MAX)),
        ("shiftseed-max-simplex", shift(simplex(), i32::MAX)),
        ("shiftseed-min", shift(simplex(), i32::MIN)),
        ("shiftseed-neg", shift(add(perlin(), c(1.0)), -12345)),
        ("clamp-basic", clamp(simplex(), c(-1.0), c(1.0))),
        ("clamp-reversed", clamp(perlin(), c(1.0), c(-1.0))),
        ("clamp-negzero-bounds", clamp(simplex(), c(-0.0), c(0.0))),
        (
            "map-nonconst-alpha",
            map(add(simplex(), c(2.0)), c(-1.0), c(3.0)),
        ),
        (
            "map-dynamic-from-to",
            map(add(perlin(), c(1.5)), simplex(), mul(perlin(), c(-2.0))),
        ),
        (
            "nested-1",
            invert(add(abs(mul(simplex(), c(-2.0))), c(1.0))),
        ),
        (
            "nested-2",
            clamp(
                add(freq(perlin(), c(0.5), c(-1.5)), abs(simplex())),
                c(-1.0),
                mul(c(2.0), c(1.5)),
            ),
        ),
        (
            "nested-3",
            map(
                add(abs(perlin()), c(1.0)),
                min(simplex(), c(-0.25)),
                max(shift(simplex(), 777), c(0.25)),
            ),
        ),
        (
            "nested-4",
            invert(invert(clamp(
                mul(add(perlin(), c(-1.0)), abs(simplex())),
                c(-2.0),
                c(3.0),
            ))),
        ),
        (
            "nested-5",
            max(
                min(
                    invert(add(simplex(), c(2.0))),
                    clamp(perlin(), c(-1.0), c(0.5)),
                ),
                shift(mul(abs(perlin()), c(-1.0)), -42),
            ),
        ),
    ]
}

fn check_guards() {
    let mut chain = c(0.5);
    for _ in 1..256 {
        chain = Noise::shift_seed(chain, 1).unwrap();
    }
    assert_eq!(chain.depth(), 256);
    assert_eq!(
        Noise::shift_seed(chain, 1).unwrap_err(),
        GraphError::DepthLimit
    );
    let mut shared = c(0.5);
    loop {
        match Noise::add(shared.clone(), shared.clone()) {
            Ok(next) => shared = next,
            Err(error) => {
                assert_eq!(error, GraphError::NodeBudget);
                break;
            }
        }
    }
    assert!(shared.expanded_nodes() <= 1_000_000);
    eprintln!("NATIVE_GRAPH_GUARDS_OK depth=256 expandedBudget=1000000");
}

fn main() {
    check_guards();
    let values = graphs();
    for (i, (name, n)) in values.iter().enumerate() {
        println!(
            "graph/{i}\t{name},{},{}",
            bits(n.min_value()),
            bits(n.max_value())
        );
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
    for (g, (_, noise)) in values.iter().enumerate() {
        for seed in seeds {
            for (i, x) in coords.iter().enumerate() {
                for (j, z) in coords.iter().enumerate() {
                    let value = noise.compute(*x, *z, seed);
                    assert!(value.is_finite());
                    println!("sample/{g}/{seed}/{i}/{j}\t{}", bits(value));
                }
            }
        }
    }
}
