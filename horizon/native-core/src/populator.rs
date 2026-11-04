//! MIT ReTerraForged terrain/ocean populators and original region/continent blending.
//! Immutable owned composition; terrain IDs belong to the owning kernel registry.
use crate::{
    cell::Cell,
    graph::{GraphError, Noise, dependency_cost, java_max},
    noise_util as u,
    perlin::Interpolation,
};
use std::sync::Arc;
type Result<T> = std::result::Result<T, GraphError>;
#[derive(Clone, Debug)]
pub struct TerrainPopulator {
    pub terrain: usize,
    pub base: Noise,
    pub height: Noise,
    pub erosion: Noise,
    pub weirdness: Noise,
    pub base_scale: f32,
    pub height_scale: f32,
    pub weight: f32,
}
#[derive(Clone, Debug)]
pub struct Populator(Arc<Data>);
#[derive(Debug)]
struct Data {
    node: Node,
    cost: (u16, usize),
}
#[derive(Debug)]
enum Node {
    Terrain(TerrainPopulator),
    Ocean {
        terrain: usize,
        height: Noise,
    },
    RegionLerp {
        lower: Populator,
        upper: Populator,
    },
    Blend {
        control: Noise,
        lower: Populator,
        upper: Populator,
        min: f32,
        max: f32,
        range: f32,
        midpoint: f32,
    },
    Continent2 {
        lower: Populator,
        upper: Populator,
        min: f32,
        max: f32,
        range: f32,
        interpolation: Interpolation,
    },
    Continent3 {
        lower: Populator,
        middle: Populator,
        upper: Populator,
        min: f32,
        mid: f32,
        max: f32,
        lower_range: f32,
        upper_range: f32,
        interpolation: Interpolation,
    },
    Selector {
        nodes: Vec<Populator>,
    },
}
impl Populator {
    fn make(node: Node, dependencies: &[(u16, usize)]) -> Result<Self> {
        Ok(Self(Arc::new(Data {
            node,
            cost: dependency_cost(dependencies.iter().copied())?,
        })))
    }
    fn cost(&self) -> (u16, usize) {
        self.0.cost
    }
    pub fn terrain(p: TerrainPopulator) -> Result<Self> {
        let costs = [
            p.base.cost(),
            p.height.cost(),
            p.erosion.cost(),
            p.weirdness.cost(),
        ];
        Self::make(Node::Terrain(p), &costs)
    }
    pub fn ocean(terrain: usize, height: Noise) -> Result<Self> {
        let costs = [height.cost()];
        Self::make(Node::Ocean { terrain, height }, &costs)
    }
    pub fn region_lerp(lower: Self, upper: Self) -> Result<Self> {
        let costs = [lower.cost(), upper.cost()];
        Self::make(Node::RegionLerp { lower, upper }, &costs)
    }
    pub fn blend(
        control: Noise,
        lower: Self,
        upper: Self,
        min: f32,
        max: f32,
        split: f32,
    ) -> Result<Self> {
        let costs = [control.cost(), lower.cost(), upper.cost()];
        let range = max - min;
        Self::make(
            Node::Blend {
                control,
                lower,
                upper,
                min,
                max,
                range,
                midpoint: min + range * split,
            },
            &costs,
        )
    }
    pub fn continent2(
        lower: Self,
        upper: Self,
        min: f32,
        max: f32,
        interpolation: Interpolation,
    ) -> Result<Self> {
        let costs = [lower.cost(), upper.cost()];
        Self::make(
            Node::Continent2 {
                lower,
                upper,
                min,
                max,
                range: max - min,
                interpolation,
            },
            &costs,
        )
    }
    pub fn continent3(
        lower: Self,
        middle: Self,
        upper: Self,
        min: f32,
        mid: f32,
        max: f32,
        interpolation: Interpolation,
    ) -> Result<Self> {
        let costs = [lower.cost(), middle.cost(), upper.cost()];
        Self::make(
            Node::Continent3 {
                lower,
                middle,
                upper,
                min,
                mid,
                max,
                lower_range: mid - min,
                upper_range: max - mid,
                interpolation,
            },
            &costs,
        )
    }
    fn weight(&self) -> Option<f32> {
        match &self.0.node {
            Node::Terrain(p) => Some(p.weight),
            _ => None,
        }
    }
    pub fn selector(modules: Vec<Self>) -> Result<Self> {
        if modules.is_empty() || modules.len() > 65536 {
            return Err(GraphError::InvalidSourceParameters(
                "selector requires 1..65536 nodes",
            ));
        }
        let mut smallest = f32::MAX;
        for p in &modules {
            let weight = p.weight().unwrap_or(1.0);
            if !weight.is_finite() || weight < 0.0 {
                return Err(GraphError::InvalidSourceParameters(
                    "finite nonnegative selector weights",
                ));
            }
            if p.weight().is_some() && weight == 0.0 {
                continue;
            }
            smallest = smallest.min(weight);
        }
        let mut nodes = Vec::new();
        if smallest != f32::MAX {
            for p in &modules {
                let weight = p.weight().unwrap_or(1.0);
                if p.weight().is_some() && weight == 0.0 {
                    continue;
                }
                let ratio = weight / smallest;
                if !ratio.is_finite() || ratio > 65536.0 {
                    return Err(GraphError::InvalidSourceParameters(
                        "selector expanded node budget 65536",
                    ));
                }
                let count = u::round_java(ratio) as usize;
                if nodes.len() + count > 65536 {
                    return Err(GraphError::InvalidSourceParameters(
                        "selector expanded node budget 65536",
                    ));
                }
                nodes.extend(std::iter::repeat_n(p.clone(), count));
            }
        }
        if nodes.is_empty() {
            nodes = modules;
        }
        let costs: Vec<_> = nodes.iter().map(Self::cost).collect();
        Self::make(Node::Selector { nodes }, &costs)
    }
    pub fn selected(&self, identity: f32) -> Result<&Self> {
        let Node::Selector { nodes } = &self.0.node else {
            return Err(GraphError::InvalidSourceParameters(
                "selected requires region selector",
            ));
        };
        let index = u::round_legacy(identity * (nodes.len() - 1) as f32);
        if index < 0 {
            return Err(GraphError::InvalidSourceParameters(
                "region identity outside selector",
            ));
        }
        nodes
            .get(index as usize)
            .ok_or(GraphError::InvalidSourceParameters(
                "region identity outside selector",
            ))
    }
    pub fn apply(&self, cell: &mut Cell, x: f32, z: f32) -> Result<()> {
        match &self.0.node {
            Node::Terrain(p) => {
                let base = p.base.compute(x, z, 0) * p.base_scale;
                let height = p.height.compute(x, z, 0) * p.height_scale;
                cell.terrain = p.terrain;
                cell.height = java_max(base + height, 0.0);
                cell.erosion = p.erosion.compute(x, z, 0);
                cell.weirdness = p.weirdness.compute(x, z, 0);
            }
            Node::Ocean { terrain, height } => {
                cell.terrain = *terrain;
                cell.height = java_max(height.compute(x, z, 0), 0.0);
                cell.erosion = -1.1;
                cell.weirdness = -1.1;
            }
            Node::RegionLerp { lower, upper } => {
                let alpha = cell.terrain_region_edge;
                if alpha == 0.0 {
                    return lower.apply(cell, x, z);
                }
                if alpha == 1.0 {
                    return upper.apply(cell, x, z);
                }
                lower.apply(cell, x, z)?;
                let (height, erosion, weirdness) = (cell.height, cell.erosion, cell.weirdness);
                upper.apply(cell, x, z)?;
                cell.height = u::lerp_f32(height, cell.height, alpha);
                cell.erosion = u::lerp_f32(erosion, cell.erosion, alpha);
                cell.weirdness = u::lerp_f32(weirdness, cell.weirdness, alpha);
            }
            Node::Blend {
                control,
                lower,
                upper,
                min,
                max,
                range,
                midpoint,
            } => {
                let select = control.compute(x, z, 0);
                if select < *min {
                    return lower.apply(cell, x, z);
                }
                if select > *max {
                    return upper.apply(cell, x, z);
                }
                let alpha = (select - min) / range;
                lower.apply(cell, x, z)?;
                let (height, erosion, weirdness, terrain) =
                    (cell.height, cell.erosion, cell.weirdness, cell.terrain);
                upper.apply(cell, x, z)?;
                cell.height = u::lerp_f32(height, cell.height, alpha);
                cell.erosion = u::lerp_f32(erosion, cell.erosion, alpha);
                cell.weirdness = u::lerp_f32(weirdness, cell.weirdness, alpha);
                if select < *midpoint {
                    cell.terrain = terrain;
                }
            }
            Node::Continent2 {
                lower,
                upper,
                min,
                max,
                range,
                interpolation,
            } => {
                if cell.continent_edge < *min {
                    return lower.apply(cell, x, z);
                }
                if cell.continent_edge > *max {
                    return upper.apply(cell, x, z);
                }
                let alpha = interpolation.apply((cell.continent_edge - min) / range);
                lower.apply(cell, x, z)?;
                let height = cell.height;
                upper.apply(cell, x, z)?;
                cell.height = u::lerp_f32(height, cell.height, alpha);
            }
            Node::Continent3 {
                lower,
                middle,
                upper,
                min,
                mid,
                max,
                lower_range,
                upper_range,
                interpolation,
            } => {
                let select = cell.continent_edge;
                if select < *min {
                    return lower.apply(cell, x, z);
                }
                if select > *max {
                    return upper.apply(cell, x, z);
                }
                if select < *mid {
                    let alpha = interpolation.apply((select - min) / lower_range);
                    lower.apply(cell, x, z)?;
                    let height = cell.height;
                    middle.apply(cell, x, z)?;
                    cell.height = u::lerp_f32(height, cell.height, alpha);
                } else {
                    let alpha = interpolation.apply((select - mid) / upper_range);
                    middle.apply(cell, x, z)?;
                    let height = cell.height;
                    upper.apply(cell, x, z)?;
                    cell.height = u::lerp_f32(height, cell.height, alpha);
                }
            }
            Node::Selector { .. } => {
                return self.selected(cell.terrain_region_id)?.apply(cell, x, z);
            }
        }
        Ok(())
    }
}
