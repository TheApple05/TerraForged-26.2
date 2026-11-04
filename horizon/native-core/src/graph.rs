// Numerical graph operations translated from MIT ReTerraForged noise records.
// Baseline 29239f1f164d19dbfcccaca4a8277d64e784207c; see LICENSE-RETERRAFORGED.
// This bounded immutable graph is not a registry decoder or an active terrain generator.
use crate::worley::{self, DistanceFunction, EdgeFunction};
use crate::{domain::Domain, erosion::Erosion, line::Line};
use crate::{
    noise_util,
    perlin::{Interpolation, Perlin},
    perlin2::Perlin2,
    ridges::{Billow, PerlinRidge, SimplexRidge},
    simplex::Simplex,
    simplex2::Simplex2,
    sources::{Cubic, White},
};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;

#[derive(Clone, Copy, Debug)]
pub enum CellFunction {
    CellValue,
    NoiseLookup,
    Distance,
}

#[derive(Debug)]
struct Cached {
    position: i64,
    value: f32,
}
impl Default for Cached {
    fn default() -> Self {
        Self {
            position: i64::MIN,
            value: 0.0,
        }
    }
}

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
    InvalidSpline,
    InvalidSteps,
}

pub(crate) fn dependency_cost(
    dependencies: impl IntoIterator<Item = (u16, usize)>,
) -> Result<(u16, usize), GraphError> {
    let mut depth = 1u16;
    let mut expanded_nodes = 1usize;
    for (child_depth, child_nodes) in dependencies {
        depth = depth.max(child_depth.checked_add(1).ok_or(GraphError::DepthLimit)?);
        if depth > MAX_DEPTH {
            return Err(GraphError::DepthLimit);
        }
        expanded_nodes = expanded_nodes
            .checked_add(child_nodes)
            .ok_or(GraphError::NodeBudget)?;
        if expanded_nodes > MAX_EXPANDED_NODES {
            return Err(GraphError::NodeBudget);
        }
    }
    Ok((depth, expanded_nodes))
}

#[derive(Clone, Copy, Debug)]
struct TerraceStep {
    value: f32,
    lower: f32,
    upper: f32,
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
    Holder(Noise),
    Perlin(Perlin),
    Perlin2(Perlin2),
    Simplex(Simplex),
    Simplex2(Simplex2),
    White(White),
    Cubic(Cubic),
    PerlinRidge(PerlinRidge),
    SimplexRidge(SimplexRidge),
    Billow(Billow),
    Worley {
        frequency: f32,
        jitter: f32,
        function: CellFunction,
        metric: DistanceFunction,
        lookup: Noise,
    },
    WorleyEdge {
        frequency: f32,
        jitter: f32,
        function: EdgeFunction,
        metric: DistanceFunction,
    },
    LegacyTemperature {
        frequency: f32,
        power: i32,
    },
    LegacyMoisture {
        input: Noise,
        power: i32,
    },
    LinearSpline {
        input: Noise,
        points: Vec<(f32, Noise)>,
    },
    Cache {
        input: Noise,
        values: Mutex<HashMap<std::thread::ThreadId, Cached>>,
    },
    Warp {
        input: Noise,
        domain: Domain,
    },
    Erosion {
        input: Noise,
        config: Erosion,
    },
    Line {
        geometry: Line,
        radius: Noise,
        fade_in: Noise,
        fade_out: Noise,
    },
    Terrace {
        input: Noise,
        ramp: Noise,
        cliff: Noise,
        ramp_height: Noise,
        blend_range: f32,
        steps: Vec<TerraceStep>,
    },
    AdvancedTerrace {
        source: Noise,
        modulation: Noise,
        mask: Noise,
        slope: Noise,
        blend_min: f32,
        blend_max: f32,
        steps: i32,
        octaves: i32,
    },
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
        let (depth, expanded_nodes) = dependency_cost(children.iter().map(|n| n.cost()))?;
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
    pub fn holder(input: Self) -> Result<Self, GraphError> {
        Self::make(
            Node::Holder(input.clone()),
            input.min_value(),
            input.max_value(),
            &[&input],
        )
    }
    pub fn constant_value(&self) -> Option<f32> {
        if let Node::Constant(value) = self.0.node {
            Some(value)
        } else {
            None
        }
    }
    pub(crate) fn cost(&self) -> (u16, usize) {
        (self.0.depth, self.0.expanded_nodes)
    }
    pub fn warp(input: Self, domain: Domain) -> Result<Self, GraphError> {
        let (depth, expanded_nodes) = dependency_cost([input.cost(), domain.cost()])?;
        Ok(Self(Arc::new(Data {
            min: input.min_value(),
            max: input.max_value(),
            depth,
            expanded_nodes,
            node: Node::Warp { input, domain },
        })))
    }
    pub fn erosion(input: Self, config: Erosion) -> Result<Self, GraphError> {
        Self::make(
            Node::Erosion {
                input: input.clone(),
                config,
            },
            0.0,
            1.0,
            &[&input],
        )
    }
    pub fn line(
        geometry: Line,
        radius: Self,
        fade_in: Self,
        fade_out: Self,
    ) -> Result<Self, GraphError> {
        Self::make(
            Node::Line {
                geometry,
                radius: radius.clone(),
                fade_in: fade_in.clone(),
                fade_out: fade_out.clone(),
            },
            0.0,
            1.0,
            &[&radius, &fade_in, &fade_out],
        )
    }
    pub fn terrace(
        input: Self,
        ramp: Self,
        cliff: Self,
        ramp_height: Self,
        blend_range: f32,
        count: i32,
    ) -> Result<Self, GraphError> {
        if !(2..=4096).contains(&count) {
            return Err(GraphError::InvalidSteps);
        }
        let spacing = (input.max_value() - input.min_value()) / (count - 1) as f32;
        let steps = (0..count)
            .map(|i| {
                let value = i as f32 * spacing;
                let blend = spacing * blend_range;
                let bound = (spacing - blend) / 2.0;
                TerraceStep {
                    value,
                    lower: value - bound,
                    upper: value + bound,
                }
            })
            .collect();
        Self::make(
            Node::Terrace {
                input: input.clone(),
                ramp: ramp.clone(),
                cliff: cliff.clone(),
                ramp_height: ramp_height.clone(),
                blend_range,
                steps,
            },
            input.min_value(),
            input.max_value(),
            &[&input, &ramp, &cliff, &ramp_height],
        )
    }
    pub fn advanced_terrace(
        source: Self,
        modulation: Self,
        mask: Self,
        slope: Self,
        blend_min: f32,
        blend_max: f32,
        steps: i32,
        octaves: i32,
    ) -> Result<Self, GraphError> {
        if !(1..=4096).contains(&steps) {
            return Err(GraphError::InvalidSteps);
        }
        if !(0..=32).contains(&octaves) {
            return Err(GraphError::IterationLimit);
        }
        Self::make(
            Node::AdvancedTerrace {
                source: source.clone(),
                modulation: modulation.clone(),
                mask: mask.clone(),
                slope: slope.clone(),
                blend_min,
                blend_max,
                steps,
                octaves,
            },
            source.min_value(),
            source.max_value(),
            &[&source, &modulation, &mask, &slope],
        )
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
    pub fn worley(
        frequency: f32,
        jitter: f32,
        function: CellFunction,
        metric: DistanceFunction,
        lookup: Self,
    ) -> Result<Self, GraphError> {
        Self::make(
            Node::Worley {
                frequency,
                jitter,
                function,
                metric,
                lookup: lookup.clone(),
            },
            0.0,
            1.0,
            &[&lookup],
        )
    }
    pub fn worley_edge(
        frequency: f32,
        jitter: f32,
        function: EdgeFunction,
        metric: DistanceFunction,
    ) -> Result<Self, GraphError> {
        Self::make(
            Node::WorleyEdge {
                frequency,
                jitter,
                function,
                metric,
            },
            0.0,
            1.0,
            &[],
        )
    }
    pub fn legacy_temperature(frequency: f32, power: i32) -> Result<Self, GraphError> {
        if power > 1024 {
            return Err(GraphError::IterationLimit);
        }
        Self::make(Node::LegacyTemperature { frequency, power }, 0.0, 1.0, &[])
    }
    pub fn legacy_moisture(input: Self, power: i32) -> Result<Self, GraphError> {
        if power > 1024 {
            return Err(GraphError::IterationLimit);
        }
        Self::make(
            Node::LegacyMoisture {
                input: input.clone(),
                power,
            },
            0.0,
            1.0,
            &[&input],
        )
    }
    pub fn linear_spline(input: Self, points: Vec<(f32, Self)>) -> Result<Self, GraphError> {
        // Codecs require nonempty points. Reject unsupported unordered knots explicitly.
        if points.is_empty()
            || points.iter().any(|p| !p.0.is_finite())
            || points.windows(2).any(|p| p[0].0 > p[1].0)
        {
            return Err(GraphError::InvalidSpline);
        }
        let min = points
            .iter()
            .map(|p| p.1.min_value())
            .reduce(java_min)
            .unwrap();
        let max = points
            .iter()
            .map(|p| p.1.max_value())
            .reduce(java_max)
            .unwrap();
        let mut children = vec![&input];
        children.extend(points.iter().map(|p| &p.1));
        Self::make(
            Node::LinearSpline {
                input: input.clone(),
                points: points.clone(),
            },
            min,
            max,
            &children,
        )
    }
    pub fn cache(input: Self) -> Result<Self, GraphError> {
        Self::make(
            Node::Cache {
                input: input.clone(),
                values: Mutex::new(HashMap::new()),
            },
            input.min_value(),
            input.max_value(),
            &[&input],
        )
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
            Node::Holder(input) => input.compute(x, z, seed),
            Node::Perlin(n) => n.compute(x, z, seed),
            Node::Perlin2(n) => n.compute(x, z, seed),
            Node::Simplex(n) => n.compute(x, z, seed),
            Node::Simplex2(n) => n.compute(x, z, seed),
            Node::White(n) => n.compute(x, z, seed),
            Node::Cubic(n) => n.compute(x, z, seed),
            Node::PerlinRidge(n) => n.compute(x, z, seed),
            Node::SimplexRidge(n) => n.compute(x, z, seed),
            Node::Billow(n) => n.compute(x, z, seed),
            Node::Worley {
                frequency,
                jitter,
                function,
                metric,
                lookup,
            } => {
                let x = x * frequency;
                let z = z * frequency;
                let nearest = worley::closest(x, z, seed, *jitter, *metric);
                match function {
                    // Source DISTANCE overrides normalization to constant zero.
                    CellFunction::Distance => 0.0,
                    CellFunction::CellValue => {
                        let (cx, cz) = nearest.map(|n| (n.x, n.z)).unwrap_or_else(|_| {
                            (noise_util::floor_legacy(x), noise_util::floor_legacy(z))
                        });
                        noise_util::map_clamped_range(
                            noise_util::val_coord_2d(seed, cx, cz),
                            -1.0,
                            1.0,
                            2.0,
                        )
                    }
                    CellFunction::NoiseLookup => match nearest {
                        Ok(n) => {
                            lookup.compute(n.x as f32 + n.vector.0, n.z as f32 + n.vector.1, seed)
                        }
                        Err(_) => f32::NAN,
                    },
                }
            }
            Node::WorleyEdge {
                frequency,
                jitter,
                function,
                metric,
            } => worley::edge_compute(x, z, seed, *frequency, *jitter, *function, *metric),
            Node::LegacyTemperature { frequency, power } => {
                let sin = noise_util::clamp_f32(noise_util::sin(z * frequency), -1.0, 1.0);
                let value = noise_util::copy_sign_legacy(noise_util::pow_i32(sin, *power), sin);
                noise_util::map_clamped_range(value, -1.0, 1.0, 2.0)
            }
            Node::LegacyMoisture { input, power } => {
                let value = input.compute(x, z, seed);
                if *power < 2 {
                    return value;
                }
                let value = (value - 0.5) * 2.0;
                let result =
                    noise_util::copy_sign_legacy(noise_util::pow_i32(value, *power), value);
                noise_util::map_clamped_range(result, -1.0, 1.0, 2.0)
            }
            Node::LinearSpline { input, points } => {
                let value = input.compute(x, z, seed);
                let first = &points[0];
                let last = &points[points.len() - 1];
                if value <= first.0 {
                    return first.1.compute(x, z, seed);
                }
                if value >= last.0 {
                    return last.1.compute(x, z, seed);
                }
                // Mth.binarySearch returns the first point strictly above input.
                let (mut low, mut high) = (0usize, points.len());
                while low < high {
                    let mid = low + (high - low) / 2;
                    if value < points[mid].0 {
                        high = mid;
                    } else {
                        low = mid + 1;
                    }
                }
                if low == 0 || low >= points.len() {
                    return f32::NAN;
                }
                let start = &points[low - 1];
                let end = &points[low];
                let from = start.1.compute(x, z, seed);
                let to = end.1.compute(x, z, seed);
                let amount = noise_util::map_f32(value, 0.0, 1.0, start.0, end.0);
                noise_util::lerp_f32(from, to, noise_util::clamp_f32(amount, 0.0, 1.0))
            }
            Node::Cache { input, values } => {
                // Exact deprecated source semantics: per thread, raw coordinate key,
                // no seed in key, and Long.MIN_VALUE initially reads zero.
                // State belongs to this graph; dropping it releases every entry.
                let mut values = values.lock().unwrap_or_else(|p| p.into_inner());
                let cached = values.entry(std::thread::current().id()).or_default();
                let position = crate::pos_util::pack_floats(x, z);
                if cached.position != position {
                    cached.value = input.compute(x, z, seed);
                    cached.position = position;
                }
                cached.value
            }
            Node::Warp { input, domain } => {
                input.compute(domain.get_x(x, z, seed), domain.get_z(x, z, seed), seed)
            }
            Node::Erosion { input, config } => config.compute(input, x, z, seed),
            Node::Line {
                geometry,
                radius,
                fade_in,
                fade_out,
            } => geometry.compute(radius, fade_in, fade_out, x, z, seed),
            Node::Terrace {
                input,
                ramp,
                cliff,
                ramp_height,
                steps,
                ..
            } => {
                let value = noise_util::clamp_f32(input.compute(x, z, seed), 0.0, 0.999999);
                let index = noise_util::floor_legacy(value * steps.len() as f32) as usize;
                let step = steps[index];
                if index == steps.len() - 1 || value < step.lower {
                    return step.value;
                }
                let next = steps[index + 1];
                if value > step.upper {
                    return next.value;
                }
                let ramp = 1.0 - ramp.compute(x, z, seed) * 0.5;
                let cliff = 1.0 - cliff.compute(x, z, seed) * 0.5;
                let alpha = (value - step.lower) / (step.upper - step.lower);
                let mut result = step.value;
                if alpha > ramp {
                    let ramp_size = 1.0 - ramp;
                    let ramp_alpha = (alpha - ramp) / ramp_size;
                    let height = ramp_height.compute(x, z, seed);
                    result += (next.value - result) * ramp_alpha * height;
                }
                if alpha > cliff {
                    let cliff_alpha = (alpha - cliff) / (1.0 - cliff);
                    result = noise_util::lerp_f32(result, next.value, cliff_alpha);
                }
                result
            }
            Node::AdvancedTerrace {
                source,
                modulation,
                mask,
                slope,
                blend_min,
                blend_max,
                steps,
                octaves,
            } => {
                let value = source.compute(x, z, seed);
                if value <= *blend_min {
                    return value;
                }
                let mask = mask.compute(x, z, seed);
                if mask == 0.0 {
                    return value;
                }
                let mut result = value;
                let slope = slope.compute(x, z, seed);
                let mod_value = modulation.compute(x, z, seed);
                for i in 1..=*octaves {
                    let count = steps.wrapping_mul(i);
                    result = noise_util::round_legacy(result * count as f32) as f32 / count as f32;
                    let delta = value - result;
                    let amount = delta * slope;
                    result += amount;
                }
                result = (result + mod_value) / (source.max_value() + modulation.max_value());
                let mut alpha = if value > *blend_max {
                    1.0
                } else {
                    (value - blend_min) / (blend_max - blend_min)
                };
                if mask != 1.0 {
                    alpha *= mask;
                }
                noise_util::lerp_f32(value, result, alpha)
            }
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

impl Noise {
    /// Source record name, including the bound-holder adapter used by codecs.
    pub fn kind(&self) -> &'static str {
        match self.0.node {
            Node::Constant(_) => "Constant",
            Node::Holder(_) => "HolderHolder",
            Node::Perlin(_) => "Perlin",
            Node::Perlin2(_) => "Perlin2",
            Node::Simplex(_) => "Simplex",
            Node::Simplex2(_) => "Simplex2",
            Node::White(_) => "White",
            Node::Cubic(_) => "Cubic",
            Node::PerlinRidge(_) => "PerlinRidge",
            Node::SimplexRidge(_) => "SimplexRidge",
            Node::Billow(_) => "Billow",
            Node::Worley { .. } => "Worley",
            Node::WorleyEdge { .. } => "WorleyEdge",
            Node::LegacyTemperature { .. } => "LegacyTemperature",
            Node::LegacyMoisture { .. } => "LegacyMoisture",
            Node::LinearSpline { .. } => "LinearSpline",
            Node::Cache { .. } => "Cache2d",
            Node::Warp { .. } => "Warp",
            Node::Erosion { .. } => "Erosion",
            Node::Line { .. } => "Line",
            Node::Terrace { .. } => "Terrace",
            Node::AdvancedTerrace { .. } => "AdvancedTerrace",
            Node::Sin { .. } => "Sin",
            Node::Power { .. } => "Power",
            Node::PowerCurve { .. } => "PowerCurve",
            Node::Curve { .. } => "Curve",
            Node::Gradient { .. } => "Gradient",
            Node::Blend { .. } => "Blend",
            Node::Alpha { .. } => "Alpha",
            Node::Boost { .. } => "Boost",
            Node::Steps { .. } => "Steps",
            Node::Threshold { .. } => "Threshold",
            Node::Add(..) => "Add",
            Node::Multiply(..) => "Multiply",
            Node::Frequency { .. } => "Frequency",
            Node::ShiftSeed { .. } => "ShiftSeed",
            Node::Abs(_) => "Abs",
            Node::Invert(_) => "Invert",
            Node::Min(..) => "Min",
            Node::Max(..) => "Max",
            Node::Clamp { .. } => "Clamp",
            Node::Map { .. } => "Map",
        }
    }
    /// Exact source traversal, including its exceptional wrapper/visitor behavior.
    pub fn map_all<F>(&self, visitor: &mut F) -> Result<Self, GraphError>
    where
        F: FnMut(Self) -> Result<Self, GraphError>,
    {
        let mapped = match &self.0.node {
            Node::Constant(_)
            | Node::Perlin(_)
            | Node::Perlin2(_)
            | Node::Simplex(_)
            | Node::Simplex2(_)
            | Node::White(_)
            | Node::Cubic(_)
            | Node::PerlinRidge(_)
            | Node::SimplexRidge(_)
            | Node::Billow(_)
            | Node::WorleyEdge { .. }
            | Node::LegacyTemperature { .. } => self.clone(),
            Node::Holder(input) => Self::holder(input.map_all(visitor)?)?,
            Node::Worley {
                frequency,
                jitter,
                function,
                metric,
                lookup,
            } => Self::worley(
                *frequency,
                *jitter,
                *function,
                *metric,
                lookup.map_all(visitor)?,
            )?,
            Node::LegacyMoisture { input, power } => {
                Self::legacy_moisture(input.map_all(visitor)?, *power)?
            }
            Node::LinearSpline { input, points } => {
                let input = input.map_all(visitor)?;
                let points = points
                    .iter()
                    .map(|(position, value)| Ok((*position, visitor(value.clone())?)))
                    .collect::<Result<Vec<_>, GraphError>>()?;
                Self::linear_spline(input, points)?
            }
            Node::Cache { input, .. } => Self::cache(input.map_all(visitor)?)?,
            Node::Warp { input, domain } => {
                Self::warp(input.map_all(visitor)?, domain.map_all(visitor)?)?
            }
            Node::Erosion { input, config } => {
                return Self::erosion(input.map_all(visitor)?, config.clone());
            }
            Node::Line {
                geometry,
                radius,
                fade_in,
                fade_out,
            } => {
                return Self::line(
                    geometry.clone(),
                    radius.clone(),
                    fade_in.clone(),
                    fade_out.clone(),
                );
            }
            Node::Terrace {
                input,
                ramp,
                cliff,
                ramp_height,
                blend_range,
                steps,
            } => Self::terrace(
                input.map_all(visitor)?,
                ramp.map_all(visitor)?,
                cliff.map_all(visitor)?,
                ramp_height.map_all(visitor)?,
                *blend_range,
                steps.len() as i32,
            )?,
            Node::AdvancedTerrace {
                source,
                modulation,
                mask,
                slope,
                blend_min,
                blend_max,
                steps,
                octaves,
            } => {
                return Self::advanced_terrace(
                    source.map_all(visitor)?,
                    modulation.map_all(visitor)?,
                    mask.map_all(visitor)?,
                    slope.map_all(visitor)?,
                    *blend_min,
                    *blend_max,
                    *steps,
                    *octaves,
                );
            }
            Node::Sin { frequency, alpha } => Self::sin(*frequency, alpha.map_all(visitor)?)?,
            Node::Power { input, power } => Self::power(input.map_all(visitor)?, *power)?,
            Node::PowerCurve { input, power, .. } => {
                return Self::power_curve(input.map_all(visitor)?, *power);
            }
            Node::Curve { input, function } => Self::curve(input.map_all(visitor)?, *function)?,
            Node::Gradient {
                input,
                lower,
                upper,
                strength,
            } => Self::gradient(
                input.map_all(visitor)?,
                lower.map_all(visitor)?,
                upper.map_all(visitor)?,
                strength.map_all(visitor)?,
            )?,
            Node::Blend {
                alpha,
                lower,
                upper,
                mid,
                range,
                interpolation,
            } => Self::blend(
                alpha.map_all(visitor)?,
                lower.map_all(visitor)?,
                upper.map_all(visitor)?,
                *mid,
                *range,
                *interpolation,
            )?,
            Node::Alpha { input, alpha } => {
                Self::alpha(input.map_all(visitor)?, alpha.map_all(visitor)?)?
            }
            Node::Boost { input, iterations } => Self::boost(input.map_all(visitor)?, *iterations)?,
            Node::Steps {
                input,
                steps,
                slope_min,
                slope_max,
                curve,
            } => Self::steps(
                input.map_all(visitor)?,
                steps.map_all(visitor)?,
                slope_min.map_all(visitor)?,
                slope_max.map_all(visitor)?,
                *curve,
            )?,
            Node::Threshold {
                input,
                lower,
                upper,
                threshold,
            } => Self::threshold(
                input.map_all(visitor)?,
                lower.map_all(visitor)?,
                upper.map_all(visitor)?,
                threshold.map_all(visitor)?,
            )?,
            Node::Add(a, b) => Self::add(a.map_all(visitor)?, b.map_all(visitor)?)?,
            Node::Multiply(a, b) => Self::multiply(a.map_all(visitor)?, b.map_all(visitor)?)?,
            Node::Frequency {
                input,
                x_freq,
                z_freq,
            } => Self::frequency(
                input.map_all(visitor)?,
                x_freq.map_all(visitor)?,
                z_freq.map_all(visitor)?,
            )?,
            Node::ShiftSeed { input, shift } => Self::shift_seed(input.map_all(visitor)?, *shift)?,
            Node::Abs(input) => input.map_all(visitor)?,
            Node::Invert(input) => Self::invert(input.map_all(visitor)?)?,
            Node::Min(a, b) => return Self::minimum(a.map_all(visitor)?, b.map_all(visitor)?),
            Node::Max(a, b) => Self::maximum(a.map_all(visitor)?, b.map_all(visitor)?)?,
            Node::Clamp { input, min, max } => Self::clamp(
                input.map_all(visitor)?,
                min.map_all(visitor)?,
                max.map_all(visitor)?,
            )?,
            Node::Map { alpha, from, to } => Self::map(
                alpha.map_all(visitor)?,
                from.map_all(visitor)?,
                to.map_all(visitor)?,
            )?,
        };
        visitor(mapped)
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
