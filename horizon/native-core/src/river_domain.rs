//! MIT ReTerraForged GenWarp; the original continentScale argument is unused.
use crate::{
    domain::Domain,
    graph::{GraphError, Noise},
    noises,
};
#[derive(Clone, Debug)]
pub struct GenWarp {
    pub lake: Domain,
    pub river: Domain,
}
fn perlin(seed: i32, scale: i32, octaves: i32, strength: f32) -> Result<Domain, GraphError> {
    Domain::warp(
        noises::perlin(seed, scale, octaves)?,
        noises::perlin(seed.wrapping_add(1), scale, octaves)?,
        Noise::constant(strength)?,
    )
}
impl GenWarp {
    pub fn empty() -> Result<Self, GraphError> {
        Ok(Self {
            lake: Domain::direct()?,
            river: Domain::direct()?,
        })
    }
    pub fn new(mut seed: i32, _continent_scale: i32) -> Result<Self, GraphError> {
        seed = seed.wrapping_add(1);
        let lake = perlin(seed, 200, 1, 300.0)?;
        seed = seed.wrapping_add(1);
        let lake = Domain::add(lake, perlin(seed, 50, 2, 50.0)?)?;
        seed = seed.wrapping_add(1);
        let river = perlin(seed, 95, 1, 25.0)?;
        seed = seed.wrapping_add(1);
        let river = Domain::add(river, perlin(seed, 16, 1, 5.0)?)?;
        Ok(Self { lake, river })
    }
}
