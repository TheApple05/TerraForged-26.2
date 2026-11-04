use terraforged_native_core::{
    graph::CellFunction as F,
    worley::{DistanceFunction as D, EdgeFunction as E},
};
use terraforged_native_core::{graph::Noise as N, perlin::Interpolation as I, simplex::Simplex};
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
        (
            "worley-EUCLIDEAN-0.0F-CELL_VALUE",
            N::worley(
                0.0125,
                0.0,
                F::CellValue,
                D::Euclidean,
                N::add(s(), c(3.0)).unwrap(),
            )
            .unwrap(),
        ),
        (
            "worley-EUCLIDEAN-0.0F-NOISE_LOOKUP",
            N::worley(
                0.0125,
                0.0,
                F::NoiseLookup,
                D::Euclidean,
                N::add(s(), c(3.0)).unwrap(),
            )
            .unwrap(),
        ),
        (
            "worley-EUCLIDEAN-0.0F-DISTANCE",
            N::worley(
                0.0125,
                0.0,
                F::Distance,
                D::Euclidean,
                N::add(s(), c(3.0)).unwrap(),
            )
            .unwrap(),
        ),
        (
            "edge-EUCLIDEAN-0.0F-DISTANCE_2",
            N::worley_edge(0.0125, 0.0, E::Distance2, D::Euclidean).unwrap(),
        ),
        (
            "edge-EUCLIDEAN-0.0F-DISTANCE_2_ADD",
            N::worley_edge(0.0125, 0.0, E::Distance2Add, D::Euclidean).unwrap(),
        ),
        (
            "edge-EUCLIDEAN-0.0F-DISTANCE_2_SUB",
            N::worley_edge(0.0125, 0.0, E::Distance2Sub, D::Euclidean).unwrap(),
        ),
        (
            "edge-EUCLIDEAN-0.0F-DISTANCE_2_MUL",
            N::worley_edge(0.0125, 0.0, E::Distance2Mul, D::Euclidean).unwrap(),
        ),
        (
            "worley-EUCLIDEAN-0.25F-CELL_VALUE",
            N::worley(
                0.0125,
                0.25,
                F::CellValue,
                D::Euclidean,
                N::add(s(), c(3.0)).unwrap(),
            )
            .unwrap(),
        ),
        (
            "worley-EUCLIDEAN-0.25F-NOISE_LOOKUP",
            N::worley(
                0.0125,
                0.25,
                F::NoiseLookup,
                D::Euclidean,
                N::add(s(), c(3.0)).unwrap(),
            )
            .unwrap(),
        ),
        (
            "worley-EUCLIDEAN-0.25F-DISTANCE",
            N::worley(
                0.0125,
                0.25,
                F::Distance,
                D::Euclidean,
                N::add(s(), c(3.0)).unwrap(),
            )
            .unwrap(),
        ),
        (
            "edge-EUCLIDEAN-0.25F-DISTANCE_2",
            N::worley_edge(0.0125, 0.25, E::Distance2, D::Euclidean).unwrap(),
        ),
        (
            "edge-EUCLIDEAN-0.25F-DISTANCE_2_ADD",
            N::worley_edge(0.0125, 0.25, E::Distance2Add, D::Euclidean).unwrap(),
        ),
        (
            "edge-EUCLIDEAN-0.25F-DISTANCE_2_SUB",
            N::worley_edge(0.0125, 0.25, E::Distance2Sub, D::Euclidean).unwrap(),
        ),
        (
            "edge-EUCLIDEAN-0.25F-DISTANCE_2_MUL",
            N::worley_edge(0.0125, 0.25, E::Distance2Mul, D::Euclidean).unwrap(),
        ),
        (
            "edge-EUCLIDEAN-0.25F-DISTANCE_2_DIV",
            N::worley_edge(0.0125, 0.25, E::Distance2Div, D::Euclidean).unwrap(),
        ),
        (
            "worley-EUCLIDEAN-1.0F-CELL_VALUE",
            N::worley(
                0.0125,
                1.0,
                F::CellValue,
                D::Euclidean,
                N::add(s(), c(3.0)).unwrap(),
            )
            .unwrap(),
        ),
        (
            "worley-EUCLIDEAN-1.0F-NOISE_LOOKUP",
            N::worley(
                0.0125,
                1.0,
                F::NoiseLookup,
                D::Euclidean,
                N::add(s(), c(3.0)).unwrap(),
            )
            .unwrap(),
        ),
        (
            "worley-EUCLIDEAN-1.0F-DISTANCE",
            N::worley(
                0.0125,
                1.0,
                F::Distance,
                D::Euclidean,
                N::add(s(), c(3.0)).unwrap(),
            )
            .unwrap(),
        ),
        (
            "edge-EUCLIDEAN-1.0F-DISTANCE_2",
            N::worley_edge(0.0125, 1.0, E::Distance2, D::Euclidean).unwrap(),
        ),
        (
            "edge-EUCLIDEAN-1.0F-DISTANCE_2_ADD",
            N::worley_edge(0.0125, 1.0, E::Distance2Add, D::Euclidean).unwrap(),
        ),
        (
            "edge-EUCLIDEAN-1.0F-DISTANCE_2_SUB",
            N::worley_edge(0.0125, 1.0, E::Distance2Sub, D::Euclidean).unwrap(),
        ),
        (
            "edge-EUCLIDEAN-1.0F-DISTANCE_2_MUL",
            N::worley_edge(0.0125, 1.0, E::Distance2Mul, D::Euclidean).unwrap(),
        ),
        (
            "edge-EUCLIDEAN-1.0F-DISTANCE_2_DIV",
            N::worley_edge(0.0125, 1.0, E::Distance2Div, D::Euclidean).unwrap(),
        ),
        (
            "worley-EUCLIDEAN--1.0F-CELL_VALUE",
            N::worley(
                0.0125,
                -1.0,
                F::CellValue,
                D::Euclidean,
                N::add(s(), c(3.0)).unwrap(),
            )
            .unwrap(),
        ),
        (
            "worley-EUCLIDEAN--1.0F-NOISE_LOOKUP",
            N::worley(
                0.0125,
                -1.0,
                F::NoiseLookup,
                D::Euclidean,
                N::add(s(), c(3.0)).unwrap(),
            )
            .unwrap(),
        ),
        (
            "worley-EUCLIDEAN--1.0F-DISTANCE",
            N::worley(
                0.0125,
                -1.0,
                F::Distance,
                D::Euclidean,
                N::add(s(), c(3.0)).unwrap(),
            )
            .unwrap(),
        ),
        (
            "edge-EUCLIDEAN--1.0F-DISTANCE_2",
            N::worley_edge(0.0125, -1.0, E::Distance2, D::Euclidean).unwrap(),
        ),
        (
            "edge-EUCLIDEAN--1.0F-DISTANCE_2_ADD",
            N::worley_edge(0.0125, -1.0, E::Distance2Add, D::Euclidean).unwrap(),
        ),
        (
            "edge-EUCLIDEAN--1.0F-DISTANCE_2_SUB",
            N::worley_edge(0.0125, -1.0, E::Distance2Sub, D::Euclidean).unwrap(),
        ),
        (
            "edge-EUCLIDEAN--1.0F-DISTANCE_2_MUL",
            N::worley_edge(0.0125, -1.0, E::Distance2Mul, D::Euclidean).unwrap(),
        ),
        (
            "edge-EUCLIDEAN--1.0F-DISTANCE_2_DIV",
            N::worley_edge(0.0125, -1.0, E::Distance2Div, D::Euclidean).unwrap(),
        ),
        (
            "worley-EUCLIDEAN-2.5F-CELL_VALUE",
            N::worley(
                0.0125,
                2.5,
                F::CellValue,
                D::Euclidean,
                N::add(s(), c(3.0)).unwrap(),
            )
            .unwrap(),
        ),
        (
            "worley-EUCLIDEAN-2.5F-NOISE_LOOKUP",
            N::worley(
                0.0125,
                2.5,
                F::NoiseLookup,
                D::Euclidean,
                N::add(s(), c(3.0)).unwrap(),
            )
            .unwrap(),
        ),
        (
            "worley-EUCLIDEAN-2.5F-DISTANCE",
            N::worley(
                0.0125,
                2.5,
                F::Distance,
                D::Euclidean,
                N::add(s(), c(3.0)).unwrap(),
            )
            .unwrap(),
        ),
        (
            "edge-EUCLIDEAN-2.5F-DISTANCE_2",
            N::worley_edge(0.0125, 2.5, E::Distance2, D::Euclidean).unwrap(),
        ),
        (
            "edge-EUCLIDEAN-2.5F-DISTANCE_2_ADD",
            N::worley_edge(0.0125, 2.5, E::Distance2Add, D::Euclidean).unwrap(),
        ),
        (
            "edge-EUCLIDEAN-2.5F-DISTANCE_2_SUB",
            N::worley_edge(0.0125, 2.5, E::Distance2Sub, D::Euclidean).unwrap(),
        ),
        (
            "edge-EUCLIDEAN-2.5F-DISTANCE_2_MUL",
            N::worley_edge(0.0125, 2.5, E::Distance2Mul, D::Euclidean).unwrap(),
        ),
        (
            "edge-EUCLIDEAN-2.5F-DISTANCE_2_DIV",
            N::worley_edge(0.0125, 2.5, E::Distance2Div, D::Euclidean).unwrap(),
        ),
        (
            "worley-MANHATTAN-0.0F-CELL_VALUE",
            N::worley(
                0.0125,
                0.0,
                F::CellValue,
                D::Manhattan,
                N::add(s(), c(3.0)).unwrap(),
            )
            .unwrap(),
        ),
        (
            "worley-MANHATTAN-0.0F-NOISE_LOOKUP",
            N::worley(
                0.0125,
                0.0,
                F::NoiseLookup,
                D::Manhattan,
                N::add(s(), c(3.0)).unwrap(),
            )
            .unwrap(),
        ),
        (
            "worley-MANHATTAN-0.0F-DISTANCE",
            N::worley(
                0.0125,
                0.0,
                F::Distance,
                D::Manhattan,
                N::add(s(), c(3.0)).unwrap(),
            )
            .unwrap(),
        ),
        (
            "edge-MANHATTAN-0.0F-DISTANCE_2",
            N::worley_edge(0.0125, 0.0, E::Distance2, D::Manhattan).unwrap(),
        ),
        (
            "edge-MANHATTAN-0.0F-DISTANCE_2_ADD",
            N::worley_edge(0.0125, 0.0, E::Distance2Add, D::Manhattan).unwrap(),
        ),
        (
            "edge-MANHATTAN-0.0F-DISTANCE_2_SUB",
            N::worley_edge(0.0125, 0.0, E::Distance2Sub, D::Manhattan).unwrap(),
        ),
        (
            "edge-MANHATTAN-0.0F-DISTANCE_2_MUL",
            N::worley_edge(0.0125, 0.0, E::Distance2Mul, D::Manhattan).unwrap(),
        ),
        (
            "worley-MANHATTAN-0.25F-CELL_VALUE",
            N::worley(
                0.0125,
                0.25,
                F::CellValue,
                D::Manhattan,
                N::add(s(), c(3.0)).unwrap(),
            )
            .unwrap(),
        ),
        (
            "worley-MANHATTAN-0.25F-NOISE_LOOKUP",
            N::worley(
                0.0125,
                0.25,
                F::NoiseLookup,
                D::Manhattan,
                N::add(s(), c(3.0)).unwrap(),
            )
            .unwrap(),
        ),
        (
            "worley-MANHATTAN-0.25F-DISTANCE",
            N::worley(
                0.0125,
                0.25,
                F::Distance,
                D::Manhattan,
                N::add(s(), c(3.0)).unwrap(),
            )
            .unwrap(),
        ),
        (
            "edge-MANHATTAN-0.25F-DISTANCE_2",
            N::worley_edge(0.0125, 0.25, E::Distance2, D::Manhattan).unwrap(),
        ),
        (
            "edge-MANHATTAN-0.25F-DISTANCE_2_ADD",
            N::worley_edge(0.0125, 0.25, E::Distance2Add, D::Manhattan).unwrap(),
        ),
        (
            "edge-MANHATTAN-0.25F-DISTANCE_2_SUB",
            N::worley_edge(0.0125, 0.25, E::Distance2Sub, D::Manhattan).unwrap(),
        ),
        (
            "edge-MANHATTAN-0.25F-DISTANCE_2_MUL",
            N::worley_edge(0.0125, 0.25, E::Distance2Mul, D::Manhattan).unwrap(),
        ),
        (
            "edge-MANHATTAN-0.25F-DISTANCE_2_DIV",
            N::worley_edge(0.0125, 0.25, E::Distance2Div, D::Manhattan).unwrap(),
        ),
        (
            "worley-MANHATTAN-1.0F-CELL_VALUE",
            N::worley(
                0.0125,
                1.0,
                F::CellValue,
                D::Manhattan,
                N::add(s(), c(3.0)).unwrap(),
            )
            .unwrap(),
        ),
        (
            "worley-MANHATTAN-1.0F-NOISE_LOOKUP",
            N::worley(
                0.0125,
                1.0,
                F::NoiseLookup,
                D::Manhattan,
                N::add(s(), c(3.0)).unwrap(),
            )
            .unwrap(),
        ),
        (
            "worley-MANHATTAN-1.0F-DISTANCE",
            N::worley(
                0.0125,
                1.0,
                F::Distance,
                D::Manhattan,
                N::add(s(), c(3.0)).unwrap(),
            )
            .unwrap(),
        ),
        (
            "edge-MANHATTAN-1.0F-DISTANCE_2",
            N::worley_edge(0.0125, 1.0, E::Distance2, D::Manhattan).unwrap(),
        ),
        (
            "edge-MANHATTAN-1.0F-DISTANCE_2_ADD",
            N::worley_edge(0.0125, 1.0, E::Distance2Add, D::Manhattan).unwrap(),
        ),
        (
            "edge-MANHATTAN-1.0F-DISTANCE_2_SUB",
            N::worley_edge(0.0125, 1.0, E::Distance2Sub, D::Manhattan).unwrap(),
        ),
        (
            "edge-MANHATTAN-1.0F-DISTANCE_2_MUL",
            N::worley_edge(0.0125, 1.0, E::Distance2Mul, D::Manhattan).unwrap(),
        ),
        (
            "edge-MANHATTAN-1.0F-DISTANCE_2_DIV",
            N::worley_edge(0.0125, 1.0, E::Distance2Div, D::Manhattan).unwrap(),
        ),
        (
            "worley-MANHATTAN--1.0F-CELL_VALUE",
            N::worley(
                0.0125,
                -1.0,
                F::CellValue,
                D::Manhattan,
                N::add(s(), c(3.0)).unwrap(),
            )
            .unwrap(),
        ),
        (
            "worley-MANHATTAN--1.0F-NOISE_LOOKUP",
            N::worley(
                0.0125,
                -1.0,
                F::NoiseLookup,
                D::Manhattan,
                N::add(s(), c(3.0)).unwrap(),
            )
            .unwrap(),
        ),
        (
            "worley-MANHATTAN--1.0F-DISTANCE",
            N::worley(
                0.0125,
                -1.0,
                F::Distance,
                D::Manhattan,
                N::add(s(), c(3.0)).unwrap(),
            )
            .unwrap(),
        ),
        (
            "edge-MANHATTAN--1.0F-DISTANCE_2",
            N::worley_edge(0.0125, -1.0, E::Distance2, D::Manhattan).unwrap(),
        ),
        (
            "edge-MANHATTAN--1.0F-DISTANCE_2_ADD",
            N::worley_edge(0.0125, -1.0, E::Distance2Add, D::Manhattan).unwrap(),
        ),
        (
            "edge-MANHATTAN--1.0F-DISTANCE_2_SUB",
            N::worley_edge(0.0125, -1.0, E::Distance2Sub, D::Manhattan).unwrap(),
        ),
        (
            "edge-MANHATTAN--1.0F-DISTANCE_2_MUL",
            N::worley_edge(0.0125, -1.0, E::Distance2Mul, D::Manhattan).unwrap(),
        ),
        (
            "edge-MANHATTAN--1.0F-DISTANCE_2_DIV",
            N::worley_edge(0.0125, -1.0, E::Distance2Div, D::Manhattan).unwrap(),
        ),
        (
            "worley-MANHATTAN-2.5F-CELL_VALUE",
            N::worley(
                0.0125,
                2.5,
                F::CellValue,
                D::Manhattan,
                N::add(s(), c(3.0)).unwrap(),
            )
            .unwrap(),
        ),
        (
            "worley-MANHATTAN-2.5F-NOISE_LOOKUP",
            N::worley(
                0.0125,
                2.5,
                F::NoiseLookup,
                D::Manhattan,
                N::add(s(), c(3.0)).unwrap(),
            )
            .unwrap(),
        ),
        (
            "worley-MANHATTAN-2.5F-DISTANCE",
            N::worley(
                0.0125,
                2.5,
                F::Distance,
                D::Manhattan,
                N::add(s(), c(3.0)).unwrap(),
            )
            .unwrap(),
        ),
        (
            "edge-MANHATTAN-2.5F-DISTANCE_2",
            N::worley_edge(0.0125, 2.5, E::Distance2, D::Manhattan).unwrap(),
        ),
        (
            "edge-MANHATTAN-2.5F-DISTANCE_2_ADD",
            N::worley_edge(0.0125, 2.5, E::Distance2Add, D::Manhattan).unwrap(),
        ),
        (
            "edge-MANHATTAN-2.5F-DISTANCE_2_SUB",
            N::worley_edge(0.0125, 2.5, E::Distance2Sub, D::Manhattan).unwrap(),
        ),
        (
            "edge-MANHATTAN-2.5F-DISTANCE_2_MUL",
            N::worley_edge(0.0125, 2.5, E::Distance2Mul, D::Manhattan).unwrap(),
        ),
        (
            "edge-MANHATTAN-2.5F-DISTANCE_2_DIV",
            N::worley_edge(0.0125, 2.5, E::Distance2Div, D::Manhattan).unwrap(),
        ),
        (
            "worley-NATURAL-0.0F-CELL_VALUE",
            N::worley(
                0.0125,
                0.0,
                F::CellValue,
                D::Natural,
                N::add(s(), c(3.0)).unwrap(),
            )
            .unwrap(),
        ),
        (
            "worley-NATURAL-0.0F-NOISE_LOOKUP",
            N::worley(
                0.0125,
                0.0,
                F::NoiseLookup,
                D::Natural,
                N::add(s(), c(3.0)).unwrap(),
            )
            .unwrap(),
        ),
        (
            "worley-NATURAL-0.0F-DISTANCE",
            N::worley(
                0.0125,
                0.0,
                F::Distance,
                D::Natural,
                N::add(s(), c(3.0)).unwrap(),
            )
            .unwrap(),
        ),
        (
            "edge-NATURAL-0.0F-DISTANCE_2",
            N::worley_edge(0.0125, 0.0, E::Distance2, D::Natural).unwrap(),
        ),
        (
            "edge-NATURAL-0.0F-DISTANCE_2_ADD",
            N::worley_edge(0.0125, 0.0, E::Distance2Add, D::Natural).unwrap(),
        ),
        (
            "edge-NATURAL-0.0F-DISTANCE_2_SUB",
            N::worley_edge(0.0125, 0.0, E::Distance2Sub, D::Natural).unwrap(),
        ),
        (
            "edge-NATURAL-0.0F-DISTANCE_2_MUL",
            N::worley_edge(0.0125, 0.0, E::Distance2Mul, D::Natural).unwrap(),
        ),
        (
            "worley-NATURAL-0.25F-CELL_VALUE",
            N::worley(
                0.0125,
                0.25,
                F::CellValue,
                D::Natural,
                N::add(s(), c(3.0)).unwrap(),
            )
            .unwrap(),
        ),
        (
            "worley-NATURAL-0.25F-NOISE_LOOKUP",
            N::worley(
                0.0125,
                0.25,
                F::NoiseLookup,
                D::Natural,
                N::add(s(), c(3.0)).unwrap(),
            )
            .unwrap(),
        ),
        (
            "worley-NATURAL-0.25F-DISTANCE",
            N::worley(
                0.0125,
                0.25,
                F::Distance,
                D::Natural,
                N::add(s(), c(3.0)).unwrap(),
            )
            .unwrap(),
        ),
        (
            "edge-NATURAL-0.25F-DISTANCE_2",
            N::worley_edge(0.0125, 0.25, E::Distance2, D::Natural).unwrap(),
        ),
        (
            "edge-NATURAL-0.25F-DISTANCE_2_ADD",
            N::worley_edge(0.0125, 0.25, E::Distance2Add, D::Natural).unwrap(),
        ),
        (
            "edge-NATURAL-0.25F-DISTANCE_2_SUB",
            N::worley_edge(0.0125, 0.25, E::Distance2Sub, D::Natural).unwrap(),
        ),
        (
            "edge-NATURAL-0.25F-DISTANCE_2_MUL",
            N::worley_edge(0.0125, 0.25, E::Distance2Mul, D::Natural).unwrap(),
        ),
        (
            "edge-NATURAL-0.25F-DISTANCE_2_DIV",
            N::worley_edge(0.0125, 0.25, E::Distance2Div, D::Natural).unwrap(),
        ),
        (
            "worley-NATURAL-1.0F-CELL_VALUE",
            N::worley(
                0.0125,
                1.0,
                F::CellValue,
                D::Natural,
                N::add(s(), c(3.0)).unwrap(),
            )
            .unwrap(),
        ),
        (
            "worley-NATURAL-1.0F-NOISE_LOOKUP",
            N::worley(
                0.0125,
                1.0,
                F::NoiseLookup,
                D::Natural,
                N::add(s(), c(3.0)).unwrap(),
            )
            .unwrap(),
        ),
        (
            "worley-NATURAL-1.0F-DISTANCE",
            N::worley(
                0.0125,
                1.0,
                F::Distance,
                D::Natural,
                N::add(s(), c(3.0)).unwrap(),
            )
            .unwrap(),
        ),
        (
            "edge-NATURAL-1.0F-DISTANCE_2",
            N::worley_edge(0.0125, 1.0, E::Distance2, D::Natural).unwrap(),
        ),
        (
            "edge-NATURAL-1.0F-DISTANCE_2_ADD",
            N::worley_edge(0.0125, 1.0, E::Distance2Add, D::Natural).unwrap(),
        ),
        (
            "edge-NATURAL-1.0F-DISTANCE_2_SUB",
            N::worley_edge(0.0125, 1.0, E::Distance2Sub, D::Natural).unwrap(),
        ),
        (
            "edge-NATURAL-1.0F-DISTANCE_2_MUL",
            N::worley_edge(0.0125, 1.0, E::Distance2Mul, D::Natural).unwrap(),
        ),
        (
            "edge-NATURAL-1.0F-DISTANCE_2_DIV",
            N::worley_edge(0.0125, 1.0, E::Distance2Div, D::Natural).unwrap(),
        ),
        (
            "worley-NATURAL--1.0F-CELL_VALUE",
            N::worley(
                0.0125,
                -1.0,
                F::CellValue,
                D::Natural,
                N::add(s(), c(3.0)).unwrap(),
            )
            .unwrap(),
        ),
        (
            "worley-NATURAL--1.0F-NOISE_LOOKUP",
            N::worley(
                0.0125,
                -1.0,
                F::NoiseLookup,
                D::Natural,
                N::add(s(), c(3.0)).unwrap(),
            )
            .unwrap(),
        ),
        (
            "worley-NATURAL--1.0F-DISTANCE",
            N::worley(
                0.0125,
                -1.0,
                F::Distance,
                D::Natural,
                N::add(s(), c(3.0)).unwrap(),
            )
            .unwrap(),
        ),
        (
            "edge-NATURAL--1.0F-DISTANCE_2",
            N::worley_edge(0.0125, -1.0, E::Distance2, D::Natural).unwrap(),
        ),
        (
            "edge-NATURAL--1.0F-DISTANCE_2_ADD",
            N::worley_edge(0.0125, -1.0, E::Distance2Add, D::Natural).unwrap(),
        ),
        (
            "edge-NATURAL--1.0F-DISTANCE_2_SUB",
            N::worley_edge(0.0125, -1.0, E::Distance2Sub, D::Natural).unwrap(),
        ),
        (
            "edge-NATURAL--1.0F-DISTANCE_2_MUL",
            N::worley_edge(0.0125, -1.0, E::Distance2Mul, D::Natural).unwrap(),
        ),
        (
            "edge-NATURAL--1.0F-DISTANCE_2_DIV",
            N::worley_edge(0.0125, -1.0, E::Distance2Div, D::Natural).unwrap(),
        ),
        (
            "worley-NATURAL-2.5F-CELL_VALUE",
            N::worley(
                0.0125,
                2.5,
                F::CellValue,
                D::Natural,
                N::add(s(), c(3.0)).unwrap(),
            )
            .unwrap(),
        ),
        (
            "worley-NATURAL-2.5F-NOISE_LOOKUP",
            N::worley(
                0.0125,
                2.5,
                F::NoiseLookup,
                D::Natural,
                N::add(s(), c(3.0)).unwrap(),
            )
            .unwrap(),
        ),
        (
            "worley-NATURAL-2.5F-DISTANCE",
            N::worley(
                0.0125,
                2.5,
                F::Distance,
                D::Natural,
                N::add(s(), c(3.0)).unwrap(),
            )
            .unwrap(),
        ),
        (
            "edge-NATURAL-2.5F-DISTANCE_2",
            N::worley_edge(0.0125, 2.5, E::Distance2, D::Natural).unwrap(),
        ),
        (
            "edge-NATURAL-2.5F-DISTANCE_2_ADD",
            N::worley_edge(0.0125, 2.5, E::Distance2Add, D::Natural).unwrap(),
        ),
        (
            "edge-NATURAL-2.5F-DISTANCE_2_SUB",
            N::worley_edge(0.0125, 2.5, E::Distance2Sub, D::Natural).unwrap(),
        ),
        (
            "edge-NATURAL-2.5F-DISTANCE_2_MUL",
            N::worley_edge(0.0125, 2.5, E::Distance2Mul, D::Natural).unwrap(),
        ),
        (
            "edge-NATURAL-2.5F-DISTANCE_2_DIV",
            N::worley_edge(0.0125, 2.5, E::Distance2Div, D::Natural).unwrap(),
        ),
        (
            "temperature-0.0F--3",
            N::legacy_temperature(0.0, -3).unwrap(),
        ),
        (
            "temperature-0.0125F--3",
            N::legacy_temperature(0.0125, -3).unwrap(),
        ),
        (
            "temperature--0.0125F--3",
            N::legacy_temperature(-0.0125, -3).unwrap(),
        ),
        ("moisture--3", N::legacy_moisture(s(), -3).unwrap()),
        ("temperature-0.0F-0", N::legacy_temperature(0.0, 0).unwrap()),
        (
            "temperature-0.0125F-0",
            N::legacy_temperature(0.0125, 0).unwrap(),
        ),
        (
            "temperature--0.0125F-0",
            N::legacy_temperature(-0.0125, 0).unwrap(),
        ),
        ("moisture-0", N::legacy_moisture(s(), 0).unwrap()),
        ("temperature-0.0F-1", N::legacy_temperature(0.0, 1).unwrap()),
        (
            "temperature-0.0125F-1",
            N::legacy_temperature(0.0125, 1).unwrap(),
        ),
        (
            "temperature--0.0125F-1",
            N::legacy_temperature(-0.0125, 1).unwrap(),
        ),
        ("moisture-1", N::legacy_moisture(s(), 1).unwrap()),
        ("temperature-0.0F-2", N::legacy_temperature(0.0, 2).unwrap()),
        (
            "temperature-0.0125F-2",
            N::legacy_temperature(0.0125, 2).unwrap(),
        ),
        (
            "temperature--0.0125F-2",
            N::legacy_temperature(-0.0125, 2).unwrap(),
        ),
        ("moisture-2", N::legacy_moisture(s(), 2).unwrap()),
        ("temperature-0.0F-3", N::legacy_temperature(0.0, 3).unwrap()),
        (
            "temperature-0.0125F-3",
            N::legacy_temperature(0.0125, 3).unwrap(),
        ),
        (
            "temperature--0.0125F-3",
            N::legacy_temperature(-0.0125, 3).unwrap(),
        ),
        ("moisture-3", N::legacy_moisture(s(), 3).unwrap()),
        ("temperature-0.0F-4", N::legacy_temperature(0.0, 4).unwrap()),
        (
            "temperature-0.0125F-4",
            N::legacy_temperature(0.0125, 4).unwrap(),
        ),
        (
            "temperature--0.0125F-4",
            N::legacy_temperature(-0.0125, 4).unwrap(),
        ),
        ("moisture-4", N::legacy_moisture(s(), 4).unwrap()),
        ("temperature-0.0F-5", N::legacy_temperature(0.0, 5).unwrap()),
        (
            "temperature-0.0125F-5",
            N::legacy_temperature(0.0125, 5).unwrap(),
        ),
        (
            "temperature--0.0125F-5",
            N::legacy_temperature(-0.0125, 5).unwrap(),
        ),
        ("moisture-5", N::legacy_moisture(s(), 5).unwrap()),
        (
            "temperature-0.0F-11",
            N::legacy_temperature(0.0, 11).unwrap(),
        ),
        (
            "temperature-0.0125F-11",
            N::legacy_temperature(0.0125, 11).unwrap(),
        ),
        (
            "temperature--0.0125F-11",
            N::legacy_temperature(-0.0125, 11).unwrap(),
        ),
        ("moisture-11", N::legacy_moisture(s(), 11).unwrap()),
        (
            "spline-dynamic",
            N::linear_spline(s(), vec![(0.0, c(-2.0)), (0.5, s()), (1.0, c(3.0))]).unwrap(),
        ),
        (
            "spline-below",
            N::linear_spline(c(-1.0), vec![(0.0, c(-2.0)), (0.5, s()), (1.0, c(3.0))]).unwrap(),
        ),
        (
            "spline-first",
            N::linear_spline(c(0.0), vec![(0.0, c(-2.0)), (0.5, s()), (1.0, c(3.0))]).unwrap(),
        ),
        (
            "spline-middle",
            N::linear_spline(c(0.5), vec![(0.0, c(-2.0)), (0.5, s()), (1.0, c(3.0))]).unwrap(),
        ),
        (
            "spline-last",
            N::linear_spline(c(1.0), vec![(0.0, c(-2.0)), (0.5, s()), (1.0, c(3.0))]).unwrap(),
        ),
        (
            "spline-above",
            N::linear_spline(c(2.0), vec![(0.0, c(-2.0)), (0.5, s()), (1.0, c(3.0))]).unwrap(),
        ),
        (
            "spline-one",
            N::linear_spline(s(), vec![(0.5, s())]).unwrap(),
        ),
        (
            "spline-duplicate",
            N::linear_spline(
                c(0.5),
                vec![(0.0, c(-1.0)), (0.5, c(1.0)), (0.5, c(2.0)), (1.0, c(3.0))],
            )
            .unwrap(),
        ),
        ("cache-graph", N::cache(s()).unwrap()),
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
    let cached = N::cache(s()).unwrap();
    print!("{}", cache_sequence(&cached, 0, 42));
    for thread in 1..=2 {
        let clone = cached.clone();
        print!(
            "{}",
            std::thread::spawn(move || cache_sequence(&clone, thread, 42 + thread))
                .join()
                .unwrap()
        );
    }
    assert!(matches!(
        N::linear_spline(s(), vec![]),
        Err(terraforged_native_core::graph::GraphError::InvalidSpline)
    ));
    assert!(matches!(
        N::linear_spline(s(), vec![(1.0, c(0.0)), (0.0, c(1.0))]),
        Err(terraforged_native_core::graph::GraphError::InvalidSpline)
    ));
    assert!(matches!(
        N::legacy_temperature(1.0, 1025),
        Err(terraforged_native_core::graph::GraphError::IterationLimit)
    ));
    assert!(matches!(
        N::legacy_moisture(s(), 1025),
        Err(terraforged_native_core::graph::GraphError::IterationLimit)
    ));
    eprintln!("NATIVE_CELLULAR_GUARDS_OK orderedNonemptySpline legacyPowerLimit=1024");
}
fn cache_sequence(cached: &N, thread: i32, seed: i32) -> String {
    let points: [(f32, f32); 6] = [
        (-0.0, 0.0),
        (1.25, -1.5),
        (1.25, -1.5),
        (-0.0, 0.0),
        (0.0, 0.0),
        (0.0, 0.0),
    ];
    let mut out = String::new();
    for (i, (x, z)) in points.into_iter().enumerate() {
        out.push_str(&format!(
            "cache-sequence/{thread}/{i}\t{}\n",
            bits(cached.compute(x, z, seed + i as i32))
        ));
    }
    out
}
