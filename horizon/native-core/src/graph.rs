// Numerical graph operations translated from MIT ReTerraForged noise records.
// Baseline 29239f1f164d19dbfcccaca4a8277d64e784207c; see LICENSE-RETERRAFORGED.
// This bounded immutable graph is not a registry decoder or an active terrain generator.
use crate::{
    noise_util,
    perlin::{Interpolation, Perlin},
    perlin2::Perlin2,
    ridges::{Billow, PerlinRidge, SimplexRidge},
    simplex::Simplex,
    simplex2::Simplex2,
    sources::{Cubic, White},
};
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
    IterationLimit,
}

#[derive(Clone, Copy, Debug)]
pub enum CurveFunction {
    Interpolation(Interpolation),
    SCurve { lower: f32, upper: f32 },
}
impl CurveFunction {
    pub fn scurve(lower: f32, upper: f32) -> Self {
        Self::SCurve {
            lower,
            upper: if upper < 0.0 {
                java_max(-lower, upper)
            } else {
                upper
            },
        }
    }
    pub fn apply(self, value: f32) -> f32 {
        match self {
            Self::Interpolation(mode) => mode.apply(value),
            Self::SCurve { lower, upper } => noise_util::pow_f32(value, lower + upper * value),
        }
    }
}

#[derive(Debug)]
enum Node {
    Constant(f32),
    Perlin(Perlin),
    Perlin2(Perlin2),
    Simplex(Simplex),
    Simplex2(Simplex2),
    White(White),
    Cubic(Cubic),
    PerlinRidge(PerlinRidge),
    SimplexRidge(SimplexRidge),
    Billow(Billow),
    Sin {
        frequency: f32,
        alpha: Noise,
    },
    Power {
        input: Noise,
        power: f32,
    },
    PowerCurve {
        input: Noise,
        power: f32,
        mid: f32,
        min: f32,
        max: f32,
    },
    Curve {
        input: Noise,
        function: CurveFunction,
    },
    Gradient {
        input: Noise,
        lower: Noise,
        upper: Noise,
        strength: Noise,
    },
    Blend {
        alpha: Noise,
        lower: Noise,
        upper: Noise,
        mid: f32,
        range: f32,
        interpolation: Interpolation,
    },
    Alpha {
        input: Noise,
        alpha: Noise,
    },
    Boost {
        input: Noise,
        iterations: i32,
    },
    Steps {
        input: Noise,
        steps: Noise,
        slope_min: Noise,
        slope_max: Noise,
        curve: CurveFunction,
    },
    Threshold {
        input: Noise,
        lower: Noise,
        upper: Noise,
        threshold: Noise,
    },
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

    pub fn white(value: White) -> Result<Self, GraphError> {
        Self::make(Node::White(value), 0.0, 1.0, &[])
    }
    pub fn cubic(value: Cubic) -> Result<Self, GraphError> {
        let (min, max) = (value.min_value(), value.max_value());
        Self::make(Node::Cubic(value), min, max, &[])
    }
    pub fn perlin_ridge(value: PerlinRidge) -> Result<Self, GraphError> {
        Self::make(Node::PerlinRidge(value), 0.0, 1.0, &[])
    }
    pub fn simplex_ridge(value: SimplexRidge) -> Result<Self, GraphError> {
        Self::make(Node::SimplexRidge(value), 0.0, 1.0, &[])
    }
    pub fn billow(value: Billow) -> Result<Self, GraphError> {
        Self::make(Node::Billow(value), 0.0, 1.0, &[])
    }
    pub fn sin(frequency: f32, alpha: Self) -> Result<Self, GraphError> {
        Self::make(
            Node::Sin {
                frequency,
                alpha: alpha.clone(),
            },
            0.0,
            1.0,
            &[&alpha],
        )
    }
    pub fn power(input: Self, power: f32) -> Result<Self, GraphError> {
        Self::make(
            Node::Power {
                input: input.clone(),
                power,
            },
            input.min_value(),
            input.max_value(),
            &[&input],
        )
    }
    pub fn power_curve(input: Self, power: f32) -> Result<Self, GraphError> {
        let mid = input.min_value() + (input.max_value() - input.min_value()) / 2.0;
        let min = mid - noise_util::pow_f32(mid - input.min_value(), power);
        let max = mid + noise_util::pow_f32(input.max_value() - mid, power);
        Self::make(
            Node::PowerCurve {
                input: input.clone(),
                power,
                mid,
                min,
                max,
            },
            input.min_value(),
            input.max_value(),
            &[&input],
        )
    }
    pub fn curve(input: Self, function: CurveFunction) -> Result<Self, GraphError> {
        Self::make(
            Node::Curve {
                input: input.clone(),
                function,
            },
            input.min_value(),
            input.max_value(),
            &[&input],
        )
    }
    pub fn gradient(
        input: Self,
        lower: Self,
        upper: Self,
        strength: Self,
    ) -> Result<Self, GraphError> {
        Self::make(
            Node::Gradient {
                input: input.clone(),
                lower: lower.clone(),
                upper: upper.clone(),
                strength: strength.clone(),
            },
            input.min_value(),
            input.max_value(),
            &[&input, &lower, &upper, &strength],
        )
    }
    pub fn blend(
        alpha: Self,
        lower: Self,
        upper: Self,
        mid: f32,
        range: f32,
        interpolation: Interpolation,
    ) -> Result<Self, GraphError> {
        Self::make(
            Node::Blend {
                alpha: alpha.clone(),
                lower: lower.clone(),
                upper: upper.clone(),
                mid,
                range,
                interpolation,
            },
            java_min(lower.min_value(), upper.min_value()),
            java_max(lower.max_value(), upper.max_value()),
            &[&alpha, &lower, &upper],
        )
    }
    pub fn alpha(input: Self, alpha: Self) -> Result<Self, GraphError> {
        Self::make(
            Node::Alpha {
                input: input.clone(),
                alpha: alpha.clone(),
            },
            input.min_value(),
            input.max_value(),
            &[&input, &alpha],
        )
    }
    pub fn boost(mut input: Self, iterations: i32) -> Result<Self, GraphError> {
        let iterations = iterations.max(1);
        if iterations > 1024 {
            return Err(GraphError::IterationLimit);
        }
        // The record invokes Noises.map, whose exact endpoint shortcut matters.
        if input.min_value() != 0.0 || input.max_value() != 1.0 {
            input = Self::map(input, Self::constant(0.0)?, Self::constant(1.0)?)?;
        }
        Self::make(
            Node::Boost {
                input: input.clone(),
                iterations,
            },
            input.min_value(),
            input.max_value(),
            &[&input],
        )
    }
    pub fn steps(
        input: Self,
        steps: Self,
        slope_min: Self,
        slope_max: Self,
        curve: CurveFunction,
    ) -> Result<Self, GraphError> {
        Self::make(
            Node::Steps {
                input: input.clone(),
                steps: steps.clone(),
                slope_min: slope_min.clone(),
                slope_max: slope_max.clone(),
                curve,
            },
            input.min_value(),
            input.max_value(),
            &[&input, &steps, &slope_min, &slope_max],
        )
    }
    pub fn threshold(
        input: Self,
        lower: Self,
        upper: Self,
        threshold: Self,
    ) -> Result<Self, GraphError> {
        Self::make(
            Node::Threshold {
                input: input.clone(),
                lower: lower.clone(),
                upper: upper.clone(),
                threshold: threshold.clone(),
            },
            input.min_value(),
            input.max_value(),
            &[&input, &lower, &upper, &threshold],
        )
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
            Node::White(n) => n.compute(x, z, seed),
            Node::Cubic(n) => n.compute(x, z, seed),
            Node::PerlinRidge(n) => n.compute(x, z, seed),
            Node::SimplexRidge(n) => n.compute(x, z, seed),
            Node::Billow(n) => n.compute(x, z, seed),
            Node::Sin { frequency, alpha } => {
                let a = alpha.compute(x, z, seed);
                let x = x * frequency;
                let z = z * frequency;
                let noise = if a == 0.0 {
                    noise_util::sin(x)
                } else if a == 1.0 {
                    noise_util::sin(z)
                } else {
                    noise_util::lerp_f32(noise_util::sin(x), noise_util::sin(z), a)
                };
                noise_util::map_clamped_range(noise, -1.0, 1.0, 2.0)
            }
            Node::Power { input, power } => noise_util::pow_f32(input.compute(x, z, seed), *power),
            Node::PowerCurve {
                input,
                power,
                mid,
                min,
                max,
            } => {
                let value = input.compute(x, z, seed);
                let value = if value >= *mid {
                    *mid + noise_util::pow_f32(value - mid, *power)
                } else {
                    *mid - noise_util::pow_f32(*mid - value, *power)
                };
                noise_util::map_clamped_range(value, *min, *max, *max - *min)
            }
            Node::Curve { input, function } => function.apply(input.compute(x, z, seed)),
            Node::Gradient {
                input,
                lower,
                upper,
                strength,
            } => {
                let value = input.compute(x, z, seed);
                let hi = upper.compute(x, z, seed);
                if value > hi {
                    return value;
                }
                let amount = strength.compute(x, z, seed);
                let lo = lower.compute(x, z, seed);
                if value < lo {
                    return noise_util::pow_f32(value, 1.0 - amount);
                }
                let alpha = 1.0 - (value - lo) / (hi - lo);
                noise_util::pow_f32(value, 1.0 - amount * alpha)
            }
            Node::Blend {
                alpha,
                lower,
                upper,
                mid,
                range,
                interpolation,
            } => {
                let middle = alpha.min_value() + (alpha.max_value() - alpha.min_value()) * mid;
                let lo = java_max(alpha.min_value(), middle - range / 2.0);
                let hi = java_min(alpha.max_value(), middle + range / 2.0);
                let amount = alpha.compute(x, z, seed);
                if amount < lo {
                    return lower.compute(x, z, seed);
                }
                if amount > hi {
                    return upper.compute(x, z, seed);
                }
                noise_util::lerp_f32(
                    lower.compute(x, z, seed),
                    upper.compute(x, z, seed),
                    interpolation.apply((amount - lo) / (hi - lo)),
                )
            }
            Node::Alpha { input, alpha } => {
                let value = input.compute(x, z, seed);
                let a = alpha.compute(x, z, seed);
                value * a + (1.0 - a)
            }
            Node::Boost { input, iterations } => {
                let mut value = input.compute(x, z, seed);
                for _ in 0..*iterations {
                    value = noise_util::pow_f32(value, 1.0 - value);
                }
                value
            }
            Node::Steps {
                input,
                steps,
                slope_min,
                slope_max,
                curve,
            } => {
                let mut value = input.compute(x, z, seed);
                let min = slope_min.compute(x, z, seed);
                let max = slope_max.compute(x, z, seed);
                let count = steps.compute(x, z, seed);
                let range = max - min;
                if range <= 0.0 {
                    return (value * count) as i32 as f32 / count;
                }
                value = 1.0 - value;
                let stepped = (value * count) as i32 as f32 / count;
                let delta = value - stepped;
                let alpha = noise_util::map_clamped_range(delta * count, min, max, range);
                1.0 - noise_util::lerp_f32(stepped, value, curve.apply(alpha))
            }
            Node::Threshold {
                input,
                lower,
                upper,
                threshold,
            } => {
                if input.compute(x, z, seed) > threshold.compute(x, z, seed) {
                    upper.compute(x, z, seed)
                } else {
                    lower.compute(x, z, seed)
                }
            }
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
