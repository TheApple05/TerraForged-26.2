//! MIT ReTerraForged VolcanoPopulator, including source terrain mutation semantics.
use crate::{
    cell::Cell,
    domain::Domain,
    graph::{CellFunction, GraphError, Noise as N},
    layout::Levels,
    noises as n,
    region::RegionConfig,
    seed::Seed,
    terrain_factories::{erosion_mid, weirdness_mid},
    worley::{DistanceFunction, EdgeFunction},
};
#[derive(Clone, Debug)]
pub struct Volcano {
    cone: N,
    height: N,
    lowlands: N,
    inversion_point: f32,
    blend_lower: f32,
    blend_upper: f32,
    blend_range: f32,
    bias: f32,
    pub weight: f32,
}
type R<T> = Result<T, GraphError>;
impl Volcano {
    pub fn new(seed: &mut Seed, region: &RegionConfig, levels: &Levels, weight: f32) -> R<Self> {
        let lookup = n::map_range(n::perlin(seed.next_seed(), 2, 1)?, 0.45, 0.65)?;
        let height = n::worley_with(
            region.seed,
            region.scale,
            CellFunction::NoiseLookup,
            DistanceFunction::Euclidean,
            lookup,
        )?;
        let domain = Domain::warp(
            region.warp_x.clone(),
            region.warp_z.clone(),
            N::constant(region.warp_strength)?,
        )?;
        let height = N::warp(height, domain.clone())?;
        let cone = n::worley_edge_with(
            region.seed,
            region.scale,
            EdgeFunction::Distance2Div,
            DistanceFunction::Euclidean,
        )?;
        let cone = N::warp(N::invert(cone)?, domain)?;
        let cone = N::power_curve(cone, 11.0)?;
        let cone = N::clamp(cone, N::constant(0.475)?, N::constant(1.0)?)?;
        let cone = n::map_range(cone, 0.0, 1.0)?;
        let cone = N::gradient(
            cone,
            N::constant(0.0)?,
            N::constant(0.5)?,
            N::constant(0.5)?,
        )?;
        let cone = n::warp_perlin(cone, seed.next_seed(), 15, 2, 10.0)?;
        let cone = N::multiply(cone, height.clone())?;
        let lowlands = n::perlin_ridge(seed.next_seed(), 150, 3)?;
        let lowlands = n::warp_perlin(lowlands, seed.next_seed(), 30, 1, 30.0)?;
        let lowlands = N::multiply(lowlands, N::constant(0.1)?)?;
        let midpoint = 0.3f32;
        let range = 0.3f32;
        let blend_lower = midpoint - range / 2.0;
        let blend_upper = blend_lower + range;
        Ok(Self {
            cone,
            height,
            lowlands,
            inversion_point: 0.94,
            blend_lower,
            blend_upper,
            blend_range: blend_upper - blend_lower,
            bias: levels.ground,
            weight,
        })
    }
    pub(crate) fn dependencies(&self) -> [(u16, usize); 3] {
        [self.cone.cost(), self.height.cost(), self.lowlands.cost()]
    }
    pub fn apply(&self, cell: &mut Cell, x: f32, z: f32) {
        let mut value = self.cone.compute(x, z, 0);
        let limit = self.height.compute(x, z, 0);
        let max_height = limit * self.inversion_point;
        cell.weirdness = weirdness_mid();
        cell.erosion = erosion_mid(4);
        if value > max_height {
            let delta = (value - max_height) * 1.0;
            let range = limit - max_height;
            let alpha = delta / range;
            if alpha > 0.925 {
                cell.terrain = 17;
            }
            value = max_height - max_height / 5.0 * alpha;
        } else if value < self.blend_lower {
            value += self.lowlands.compute(x, z, 0);
            cell.terrain = 16;
        } else if value < self.blend_upper {
            let alpha = 1.0 - (value - self.blend_lower) / self.blend_range;
            value += self.lowlands.compute(x, z, 0) * alpha;
            cell.terrain = 16;
        }
        cell.height = self.bias + value;
    }
    pub fn modify_type(cell: &mut Cell, levels: &Levels) {
        if cell.terrain == 17 && (cell.height < levels.water || cell.river_mask < 0.85) {
            cell.terrain = 16;
        }
    }
}
