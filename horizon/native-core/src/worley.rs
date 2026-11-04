// ReTerraForged MIT Worley/WorleyEdge and distance/edge functions, baseline
// 29239f1f164d19dbfcccaca4a8277d64e784207c. No codec or graph substitution.
use crate::noise_util::{cell, floor_legacy, map_clamped_range};

#[derive(Clone, Copy, Debug)]
pub enum DistanceFunction {
    Euclidean,
    Manhattan,
    Natural,
}

impl DistanceFunction {
    pub fn apply(self, x: f32, y: f32) -> f32 {
        match self {
            DistanceFunction::Euclidean => x * x + y * y,
            DistanceFunction::Manhattan => x.abs() + y.abs(),
            DistanceFunction::Natural => x.abs() + y.abs() + (x * x + y * y),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum EdgeFunction {
    Distance2,
    Distance2Add,
    Distance2Sub,
    Distance2Mul,
    Distance2Div,
}

impl EdgeFunction {
    pub fn apply(self, distance: f32, distance2: f32) -> f32 {
        match self {
            EdgeFunction::Distance2 => distance2 - 1.0f32,
            EdgeFunction::Distance2Add => distance2 + distance - 1.0f32,
            EdgeFunction::Distance2Sub => distance2 - distance - 1.0f32,
            EdgeFunction::Distance2Mul => distance2 * distance - 1.0f32,
            EdgeFunction::Distance2Div => distance / distance2 - 1.0f32,
        }
    }

    pub fn max(self) -> f32 {
        match self {
            EdgeFunction::Distance2 => 1.0f32,
            EdgeFunction::Distance2Add => 1.6f32,
            EdgeFunction::Distance2Sub => 0.8f32,
            EdgeFunction::Distance2Mul => 0.7f32,
            EdgeFunction::Distance2Div => 0.0f32,
        }
    }

    pub fn min(self) -> f32 {
        match self {
            EdgeFunction::Distance2 => -1.0f32,
            EdgeFunction::Distance2Add => -1.0f32,
            EdgeFunction::Distance2Sub => -1.0f32,
            EdgeFunction::Distance2Mul => -1.0f32,
            EdgeFunction::Distance2Div => -1.0f32,
        }
    }

    pub fn range(self) -> f32 {
        match self {
            EdgeFunction::Distance2 => 2.0f32,
            EdgeFunction::Distance2Add => 2.6f32,
            EdgeFunction::Distance2Sub => 1.8f32,
            EdgeFunction::Distance2Mul => 1.7f32,
            EdgeFunction::Distance2Div => 1.0f32,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NoClosestCell;

#[derive(Clone, Copy, Debug)]
pub struct Closest {
    pub x: i32,
    pub z: i32,
    pub distance: f32,
    pub vector: (f32, f32),
}

pub fn closest(
    x: f32,
    z: f32,
    seed: i32,
    jitter: f32,
    metric: DistanceFunction,
) -> Result<Closest, NoClosestCell> {
    let xi = floor_legacy(x);
    let zi = floor_legacy(z);
    let mut cell_x = xi;
    let mut cell_z = zi;
    let mut nearest = f32::MAX;
    let mut found: Option<(f32, f32)> = None;

    let mut dy: i32 = -1;
    while dy <= 1 {
        let mut dx: i32 = -1;
        while dx <= 1 {
            let cx = xi.wrapping_add(dx);
            let cz = zi.wrapping_add(dy);
            let vector = cell(seed, cx, cz);
            let delta_x = cx as f32 + vector.0 * jitter - x;
            let delta_z = cz as f32 + vector.1 * jitter - z;
            let dist = metric.apply(delta_x, delta_z);
            if dist < nearest {
                nearest = dist;
                found = Some(vector);
                cell_x = cx;
                cell_z = cz;
            }
            dx += 1;
        }
        dy += 1;
    }

    match found {
        Some(vector) => Ok(Closest {
            x: cell_x,
            z: cell_z,
            distance: nearest,
            vector,
        }),
        None => Err(NoClosestCell),
    }
}

pub fn edge_sample(
    x: f32,
    z: f32,
    seed: i32,
    jitter: f32,
    edge: EdgeFunction,
    metric: DistanceFunction,
) -> f32 {
    let xi = floor_legacy(x);
    let zi = floor_legacy(z);
    let mut nearest1 = f32::MAX;
    let mut nearest2 = f32::MAX;

    let mut dy: i32 = -1;
    while dy <= 1 {
        let mut dx: i32 = -1;
        while dx <= 1 {
            let cx = xi.wrapping_add(dx);
            let cz = zi.wrapping_add(dy);
            let vector = cell(seed, cx, cz);
            let delta_x = cx as f32 + vector.0 * jitter - x;
            let delta_z = cz as f32 + vector.1 * jitter - z;
            let dist = metric.apply(delta_x, delta_z);
            if dist < nearest1 {
                nearest2 = nearest1;
                nearest1 = dist;
            } else if dist < nearest2 {
                nearest2 = dist;
            }
            dx += 1;
        }
        dy += 1;
    }

    edge.apply(nearest1, nearest2)
}

pub fn edge_compute(
    x: f32,
    z: f32,
    seed: i32,
    frequency: f32,
    jitter: f32,
    edge: EdgeFunction,
    metric: DistanceFunction,
) -> f32 {
    let x = x * frequency;
    let z = z * frequency;
    let value = edge_sample(x, z, seed, jitter, edge, metric);
    map_clamped_range(value, edge.min(), edge.max(), edge.range())
}
