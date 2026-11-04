// Numerical graph operations translated from MIT ReTerraForged noise records.
// Baseline 29239f1f164d19dbfcccaca4a8277d64e784207c; see LICENSE-RETERRAFORGED.
// This bounded immutable graph is not a registry decoder or an active terrain generator.
use crate::{noise_util, perlin::Perlin, perlin2::Perlin2, simplex::Simplex, simplex2::Simplex2};
use std::sync::Arc;

const MAX_DEPTH: u16 = 256;
const MAX_EXPANDED_NODES: usize = 1_000_000;

#[derive(Clone, Debug)]
pub struct Noise(Arc<Data>);

#[derive(Debug)]
struct Data {
    node: Node,
    min: f32,
    max: f32,
    depth: u16,
    expanded_nodes: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GraphError {
    DepthLimit,
    NodeBudget,
}

#[derive(Debug)]
enum Node {
    Constant(f32),
    Perlin(Perlin),
    Perlin2(Perlin2),
    Simplex(Simplex),
    Simplex2(Simplex2),
    Add(Noise, Noise),
    Multiply(Noise, Noise),
    Frequency {
        input: Noise,
        x_freq: Noise,
        z_freq: Noise,
    },
    ShiftSeed {
        input: Noise,
        shift: i32,
    },
    Abs(Noise),
    Invert(Noise),
    Min(Noise, Noise),
    Max(Noise, Noise),
    Clamp {
        input: Noise,
        min: Noise,
        max: Noise,
    },
    Map {
        alpha: Noise,
        from: Noise,
        to: Noise,
    },
}

impl Noise {
    fn make(node: Node, min: f32, max: f32, children: &[&Self]) -> Result<Self, GraphError> {
        let depth = children.iter().map(|n| n.0.depth).max().unwrap_or(0) + 1;
        if depth > MAX_DEPTH {
            return Err(GraphError::DepthLimit);
        }
        let mut expanded_nodes = 1usize;
        for child in children {
            expanded_nodes = expanded_nodes
                .checked_add(child.0.expanded_nodes)
                .ok_or(GraphError::NodeBudget)?;
            if expanded_nodes > MAX_EXPANDED_NODES {
                return Err(GraphError::NodeBudget);
            }
        }
        Ok(Self(Arc::new(Data {
            node,
            min,
            max,
            depth,
            expanded_nodes,
        })))
    }

    pub fn constant(value: f32) -> Result<Self, GraphError> {
        Self::make(Node::Constant(value), value, value, &[])
    }
    pub fn perlin(value: Perlin) -> Result<Self, GraphError> {
        Self::make(Node::Perlin(value), 0.0, 1.0, &[])
    }
    pub fn perlin2(value: Perlin2) -> Result<Self, GraphError> {
        Self::make(Node::Perlin2(value), 0.0, 1.0, &[])
    }
    pub fn simplex(value: Simplex) -> Result<Self, GraphError> {
        Self::make(Node::Simplex(value), 0.0, 1.0, &[])
    }
    pub fn simplex2(value: Simplex2) -> Result<Self, GraphError> {
        Self::make(Node::Simplex2(value), 0.0, 1.0, &[])
    }

    pub fn add(a: Self, b: Self) -> Result<Self, GraphError> {
        Self::make(
            Node::Add(a.clone(), b.clone()),
            a.min_value() + b.min_value(),
            a.max_value() + b.max_value(),
            &[&a, &b],
        )
    }
    pub fn multiply(a: Self, b: Self) -> Result<Self, GraphError> {
        // These are the source's reported endpoints, not corrected interval bounds.
        Self::make(
            Node::Multiply(a.clone(), b.clone()),
            a.min_value() * b.min_value(),
            a.max_value() * b.max_value(),
            &[&a, &b],
        )
    }
    pub fn frequency(input: Self, x_freq: Self, z_freq: Self) -> Result<Self, GraphError> {
        Self::make(
            Node::Frequency {
                input: input.clone(),
                x_freq: x_freq.clone(),
                z_freq: z_freq.clone(),
            },
            input.min_value(),
            input.max_value(),
            &[&input, &x_freq, &z_freq],
        )
    }
    pub fn shift_seed(input: Self, shift: i32) -> Result<Self, GraphError> {
        Self::make(
            Node::ShiftSeed {
                input: input.clone(),
                shift,
            },
            input.min_value(),
            input.max_value(),
            &[&input],
        )
    }
    pub fn abs(input: Self) -> Result<Self, GraphError> {
        Self::make(
            Node::Abs(input.clone()),
            input.min_value().abs(),
            input.max_value().abs(),
            &[&input],
        )
    }
    pub fn invert(input: Self) -> Result<Self, GraphError> {
        Self::make(
            Node::Invert(input.clone()),
            input.min_value(),
            input.max_value(),
            &[&input],
        )
    }
    pub fn minimum(a: Self, b: Self) -> Result<Self, GraphError> {
        Self::make(
            Node::Min(a.clone(), b.clone()),
            java_min(a.min_value(), b.min_value()),
            java_min(a.max_value(), b.max_value()),
            &[&a, &b],
        )
    }
    pub fn maximum(a: Self, b: Self) -> Result<Self, GraphError> {
        Self::make(
            Node::Max(a.clone(), b.clone()),
            java_max(a.min_value(), b.min_value()),
            java_max(a.max_value(), b.max_value()),
            &[&a, &b],
        )
    }
    pub fn clamp(input: Self, min: Self, max: Self) -> Result<Self, GraphError> {
        Self::make(
            Node::Clamp {
                input: input.clone(),
                min: min.clone(),
                max: max.clone(),
            },
            min.min_value(),
            max.max_value(),
            &[&input, &min, &max],
        )
    }
    pub fn map(alpha: Self, from: Self, to: Self) -> Result<Self, GraphError> {
        Self::make(
            Node::Map {
                alpha: alpha.clone(),
                from: from.clone(),
                to: to.clone(),
            },
            from.min_value(),
            to.max_value(),
            &[&alpha, &from, &to],
        )
    }
    pub fn min_value(&self) -> f32 {
        self.0.min
    }
    pub fn max_value(&self) -> f32 {
        self.0.max
    }
    pub fn depth(&self) -> u16 {
        self.0.depth
    }
    pub fn expanded_nodes(&self) -> usize {
        self.0.expanded_nodes
    }

    pub fn compute(&self, x: f32, z: f32, seed: i32) -> f32 {
        match &self.0.node {
            Node::Constant(value) => *value,
            Node::Perlin(n) => n.compute(x, z, seed),
            Node::Perlin2(n) => n.compute(x, z, seed),
            Node::Simplex(n) => n.compute(x, z, seed),
            Node::Simplex2(n) => n.compute(x, z, seed),
            Node::Add(a, b) => a.compute(x, z, seed) + b.compute(x, z, seed),
            Node::Multiply(a, b) => {
                let first = a.compute(x, z, seed);
                if first != 0.0 {
                    first * b.compute(x, z, seed)
                } else {
                    0.0
                }
            }
            Node::Frequency {
                input,
                x_freq,
                z_freq,
            } => {
                let xf = x_freq.compute(x, z, seed);
                let zf = z_freq.compute(x, z, seed);
                input.compute(x * xf, z * zf, seed)
            }
            Node::ShiftSeed { input, shift } => input.compute(x, z, seed.wrapping_add(*shift)),
            Node::Abs(input) => input.compute(x, z, seed).abs(),
            Node::Invert(input) => {
                input.max_value()
                    - noise_util::clamp_f32(
                        input.compute(x, z, seed),
                        input.min_value(),
                        input.max_value(),
                    )
            }
            Node::Min(a, b) => java_min(a.compute(x, z, seed), b.compute(x, z, seed)),
            Node::Max(a, b) => java_max(a.compute(x, z, seed), b.compute(x, z, seed)),
            Node::Clamp { input, min, max } => noise_util::clamp_f32(
                input.compute(x, z, seed),
                min.compute(x, z, seed),
                max.compute(x, z, seed),
            ),
            Node::Map { alpha, from, to } => {
                let value = alpha.compute(x, z, seed);
                let amount = (value - alpha.min_value()) / (alpha.max_value() - alpha.min_value());
                let min = from.compute(x, z, seed);
                let max = to.compute(x, z, seed);
                min + amount * (max - min)
            }
        }
    }
}

fn java_min(a: f32, b: f32) -> f32 {
    if a.is_nan() {
        return a;
    }
    if a == 0.0 && b == 0.0 && b.to_bits() == 0x80000000 {
        return b;
    }
    if a <= b { a } else { b }
}

fn java_max(a: f32, b: f32) -> f32 {
    if a.is_nan() {
        return a;
    }
    if a == 0.0 && b == 0.0 && a.to_bits() == 0x80000000 {
        return b;
    }
    if a >= b { a } else { b }
}
