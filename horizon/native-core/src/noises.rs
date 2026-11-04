//! Source Noises factory scale/default/seed behavior; raw record graphs remain separate.
use crate::{
    domain::Domain,
    graph::CellFunction,
    graph::{GraphError, Noise},
    perlin::{Interpolation, Perlin},
    perlin2::Perlin2,
    ridges::{Billow, PerlinRidge, SimplexRidge},
    simplex::Simplex,
    simplex2::Simplex2,
    sources::{Cubic, White},
    worley::{DistanceFunction, EdgeFunction},
};
type Result<T> = std::result::Result<T, GraphError>;
pub fn perlin(seed: i32, scale: i32, octaves: i32) -> Result<Noise> {
    perlin_with(seed, scale, octaves, 2.0, 0.5)
}
pub fn perlin_with(
    seed: i32,
    scale: i32,
    octaves: i32,
    lacunarity: f32,
    gain: f32,
) -> Result<Noise> {
    Noise::perlin(
        Perlin::new(
            seed,
            1.0 / scale as f32,
            octaves,
            lacunarity,
            gain,
            Interpolation::Curve3,
        )
        .map_err(|_| GraphError::InvalidSourceParameters("Perlin octave limit 0..256"))?,
    )
}
pub fn perlin2(seed: i32, scale: i32, octaves: i32) -> Result<Noise> {
    perlin2_with(seed, scale, octaves, 2.0, 0.5)
}
pub fn perlin2_with(
    seed: i32,
    scale: i32,
    octaves: i32,
    lacunarity: f32,
    gain: f32,
) -> Result<Noise> {
    Noise::perlin2(
        Perlin2::new(
            seed,
            1.0 / scale as f32,
            octaves,
            lacunarity,
            gain,
            Interpolation::Curve3,
        )
        .map_err(|_| GraphError::InvalidSourceParameters("Perlin2 octave limit 0..256"))?,
    )
}
pub fn simplex(seed: i32, scale: i32, octaves: i32) -> Result<Noise> {
    simplex_with(seed, scale, octaves, 2.0, 0.5)
}
pub fn simplex_with(
    seed: i32,
    scale: i32,
    octaves: i32,
    lacunarity: f32,
    gain: f32,
) -> Result<Noise> {
    Noise::shift_seed(
        Noise::simplex(
            Simplex::new(
                1.0 / scale as f32,
                octaves,
                lacunarity,
                gain,
                Interpolation::Curve3,
            )
            .map_err(|_| GraphError::InvalidSourceParameters("Simplex octave limit 0..32"))?,
        )?,
        seed,
    )
}
pub fn simplex2(seed: i32, scale: i32, octaves: i32) -> Result<Noise> {
    simplex2_with(seed, scale, octaves, 2.0, 0.5)
}
pub fn simplex2_with(
    seed: i32,
    scale: i32,
    octaves: i32,
    lacunarity: f32,
    gain: f32,
) -> Result<Noise> {
    Noise::shift_seed(
        Noise::simplex2(
            Simplex2::new(
                1.0 / scale as f32,
                octaves,
                lacunarity,
                gain,
                Interpolation::Curve3,
            )
            .map_err(|_| GraphError::InvalidSourceParameters("Simplex2 octave limit 0..32"))?,
        )?,
        seed,
    )
}
pub fn perlin_ridge(seed: i32, scale: i32, octaves: i32) -> Result<Noise> {
    perlin_ridge_with(seed, scale, octaves, 2.0, 0.975)
}
pub fn perlin_ridge_with(
    seed: i32,
    scale: i32,
    octaves: i32,
    lacunarity: f32,
    gain: f32,
) -> Result<Noise> {
    Noise::shift_seed(
        Noise::perlin_ridge(
            PerlinRidge::new(
                1.0 / scale as f32,
                octaves,
                lacunarity,
                gain,
                Interpolation::Curve3,
            )
            .map_err(|_| GraphError::InvalidSourceParameters("PerlinRidge octave limit 0..32"))?,
        )?,
        seed,
    )
}
pub fn simplex_ridge(seed: i32, scale: i32, octaves: i32) -> Result<Noise> {
    simplex_ridge_with(seed, scale, octaves, 2.0, 0.975)
}
pub fn simplex_ridge_with(
    seed: i32,
    scale: i32,
    octaves: i32,
    lacunarity: f32,
    gain: f32,
) -> Result<Noise> {
    Noise::shift_seed(
        Noise::simplex_ridge(
            SimplexRidge::new(
                1.0 / scale as f32,
                octaves,
                lacunarity,
                gain,
                Interpolation::Curve3,
            )
            .map_err(|_| GraphError::InvalidSourceParameters("SimplexRidge octave limit 0..30"))?,
        )?,
        seed,
    )
}
pub fn billow(seed: i32, scale: i32, octaves: i32) -> Result<Noise> {
    billow_with(seed, scale, octaves, 2.0, 0.5)
}
pub fn billow_with(
    seed: i32,
    scale: i32,
    octaves: i32,
    lacunarity: f32,
    gain: f32,
) -> Result<Noise> {
    Noise::shift_seed(
        Noise::billow(
            Billow::new(
                1.0 / scale as f32,
                octaves,
                lacunarity,
                gain,
                Interpolation::Curve3,
            )
            .map_err(|_| GraphError::InvalidSourceParameters("Billow octave limit 0..32"))?,
        )?,
        seed,
    )
}
pub fn cubic(seed: i32, scale: i32, octaves: i32) -> Result<Noise> {
    cubic_with(seed, scale, octaves, 2.0, 0.5)
}
pub fn cubic_with(
    seed: i32,
    scale: i32,
    octaves: i32,
    lacunarity: f32,
    gain: f32,
) -> Result<Noise> {
    Noise::shift_seed(
        Noise::cubic(
            Cubic::new(1.0 / scale as f32, octaves, lacunarity, gain)
                .map_err(|_| GraphError::InvalidSourceParameters("Cubic octave limit 0..32"))?,
        )?,
        seed,
    )
}
pub fn white(seed: i32, scale: i32) -> Result<Noise> {
    Noise::shift_seed(Noise::white(White::new(1.0 / scale as f32))?, seed)
}
pub fn sin(seed: i32, frequency: f32, alpha: Noise) -> Result<Noise> {
    Noise::shift_seed(Noise::sin(frequency, alpha)?, seed)
}
pub fn worley(seed: i32, scale: i32) -> Result<Noise> {
    worley_with(
        seed,
        scale,
        CellFunction::CellValue,
        DistanceFunction::Euclidean,
        Noise::constant(0.0)?,
    )
}
pub fn worley_with(
    seed: i32,
    scale: i32,
    function: CellFunction,
    metric: DistanceFunction,
    lookup: Noise,
) -> Result<Noise> {
    Noise::shift_seed(
        Noise::worley(1.0 / scale as f32, 1.0, function, metric, lookup)?,
        seed,
    )
}
pub fn worley_edge(seed: i32, scale: i32) -> Result<Noise> {
    worley_edge_with(
        seed,
        scale,
        EdgeFunction::Distance2,
        DistanceFunction::Euclidean,
    )
}
pub fn worley_edge_with(
    seed: i32,
    scale: i32,
    function: EdgeFunction,
    metric: DistanceFunction,
) -> Result<Noise> {
    Noise::shift_seed(
        Noise::worley_edge(1.0 / scale as f32, 1.0, function, metric)?,
        seed,
    )
}
pub fn map(input: Noise, from: Noise, to: Noise) -> Result<Noise> {
    if from.min_value() == from.max_value()
        && from.min_value() == input.min_value()
        && to.min_value() == to.max_value()
        && to.max_value() == input.max_value()
    {
        Ok(input)
    } else {
        Noise::map(input, from, to)
    }
}
pub fn map_range(input: Noise, from: f32, to: f32) -> Result<Noise> {
    map(input, Noise::constant(from)?, Noise::constant(to)?)
}
pub fn frequency(input: Noise, x: f32, z: f32) -> Result<Noise> {
    Noise::frequency(input, Noise::constant(x)?, Noise::constant(z)?)
}
pub fn warp_perlin(
    input: Noise,
    seed: i32,
    scale: i32,
    octaves: i32,
    strength: f32,
) -> Result<Noise> {
    Noise::warp(
        input,
        Domain::warp(
            perlin(seed, scale, octaves)?,
            perlin(seed.wrapping_add(1), scale, octaves)?,
            Noise::constant(strength)?,
        )?,
    )
}
pub fn warp_white(input: Noise, seed: i32, scale: i32, strength: f32) -> Result<Noise> {
    Noise::warp(
        input,
        Domain::warp(
            white(seed, scale)?,
            white(seed.wrapping_add(1), scale)?,
            Noise::constant(strength)?,
        )?,
    )
}
