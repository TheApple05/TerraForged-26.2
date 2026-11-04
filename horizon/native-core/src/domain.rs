// MIT ReTerraForged domain records, baseline 29239f1f164d19dbfcccaca4a8277d64e784207c.
use crate::{
    graph::{GraphError, Noise},
    noise_util,
};
use std::sync::Arc;
#[derive(Clone, Debug)]
pub struct Domain(Arc<Data>);
#[derive(Debug)]
struct Data {
    node: Node,
    depth: u16,
    expanded_nodes: usize,
}
#[derive(Debug)]
enum Node {
    Direct,
    Warp {
        _x: Noise,
        _z: Noise,
        mapped_x: Noise,
        mapped_z: Noise,
        distance: Noise,
    },
    Direction {
        direction: Noise,
        strength: Noise,
    },
    Add(Domain, Domain),
    Compound(Domain, Domain),
}
impl Domain {
    fn make(node: Node, dependencies: &[(u16, usize)]) -> Result<Self, GraphError> {
        let (depth, expanded_nodes) = crate::graph::dependency_cost(dependencies.iter().copied())?;
        Ok(Self(Arc::new(Data {
            node,
            depth,
            expanded_nodes,
        })))
    }
    pub(crate) fn cost(&self) -> (u16, usize) {
        (self.0.depth, self.0.expanded_nodes)
    }
    pub fn direct() -> Result<Self, GraphError> {
        Self::make(Node::Direct, &[])
    }
    pub fn warp(x: Noise, z: Noise, distance: Noise) -> Result<Self, GraphError> {
        fn mapped(input: Noise) -> Result<Noise, GraphError> {
            if input.min_value() == -0.5 && input.max_value() == 0.5 {
                Ok(input)
            } else {
                Noise::map(input, Noise::constant(-0.5)?, Noise::constant(0.5)?)
            }
        }
        let mapped_x = mapped(x.clone())?;
        let mapped_z = mapped(z.clone())?;
        let cost = [
            x.cost(),
            z.cost(),
            mapped_x.cost(),
            mapped_z.cost(),
            distance.cost(),
        ];
        Self::make(
            Node::Warp {
                _x: x,
                _z: z,
                mapped_x,
                mapped_z,
                distance,
            },
            &cost,
        )
    }
    pub fn direction(direction: Noise, strength: Noise) -> Result<Self, GraphError> {
        let cost = [direction.cost(), strength.cost()];
        Self::make(
            Node::Direction {
                direction,
                strength,
            },
            &cost,
        )
    }
    pub fn add(a: Self, b: Self) -> Result<Self, GraphError> {
        let cost = [a.cost(), b.cost()];
        Self::make(Node::Add(a, b), &cost)
    }
    pub fn compound(a: Self, b: Self) -> Result<Self, GraphError> {
        let cost = [a.cost(), b.cost()];
        Self::make(Node::Compound(a, b), &cost)
    }
    pub fn get_x(&self, x: f32, z: f32, seed: i32) -> f32 {
        x + self.offset_x(x, z, seed)
    }
    pub fn get_z(&self, x: f32, z: f32, seed: i32) -> f32 {
        z + self.offset_z(x, z, seed)
    }
    pub fn offset_x(&self, x: f32, z: f32, seed: i32) -> f32 {
        match &self.0.node {
            Node::Direct => 0.0,
            Node::Warp {
                mapped_x, distance, ..
            } => mapped_x.compute(x, z, seed) * distance.compute(x, z, seed),
            Node::Direction {
                direction,
                strength,
            } => {
                noise_util::sin(direction.compute(x, z, seed) * 6.2831855f32)
                    * strength.compute(x, z, seed)
            }
            Node::Add(a, b) => a.offset_x(x, z, seed) + b.offset_x(x, z, seed),
            Node::Compound(a, b) => b.offset_x(a.get_x(x, z, seed), a.get_z(x, z, seed), seed),
        }
    }
    pub fn offset_z(&self, x: f32, z: f32, seed: i32) -> f32 {
        match &self.0.node {
            Node::Direct => 0.0,
            Node::Warp {
                mapped_z, distance, ..
            } => mapped_z.compute(x, z, seed) * distance.compute(x, z, seed),
            Node::Direction {
                direction,
                strength,
            } => {
                noise_util::cos(direction.compute(x, z, seed) * 6.2831855f32)
                    * strength.compute(x, z, seed)
            }
            Node::Add(a, b) => a.offset_z(x, z, seed) + b.offset_z(x, z, seed),
            Node::Compound(a, b) => b.offset_z(a.get_x(x, z, seed), a.get_z(x, z, seed), seed),
        }
    }
}
