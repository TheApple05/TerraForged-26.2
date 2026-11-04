//! MIT ReTerraForged AdvancedContinentGenerator / AbstractContinent geometry.
//! The river cache is a separate, unfinished layer; this owns immutable geometry only.
use crate::{
    cell::Cell,
    domain::Domain,
    graph::{GraphError, Noise},
    line::dist_sq,
    noise_util as u, noises as n,
    preset::{ControlPoints, WorldSettings},
    seed::Seed,
};

#[derive(Clone, Debug)]
pub struct AdvancedContinent {
    seed: i32,
    skipping_seed: i32,
    continent_scale: i32,
    jitter: f32,
    skip_threshold: f32,
    control_points: ControlPoints,
    frequency: f32,
    variance: f32,
    variance_seed: i32,
    warp: Domain,
    cliff_noise: Noise,
    bay_noise: Noise,
}

type Result<T> = std::result::Result<T, GraphError>;
impl AdvancedContinent {
    /// Mirrors constructor seed draws. The source river-cache constructor consumes
    /// no Seed.next calls; river construction is deliberately not approximated here.
    pub fn new(seed: &mut Seed, world: &WorldSettings) -> Result<Self> {
        let c = &world.continent;
        if !(2..=i32::MAX / 4).contains(&c.continent_scale)
            || ![
                c.continent_jitter,
                c.continent_skipping,
                c.continent_size_variance,
                c.continent_noise_lacunarity,
                c.continent_noise_gain,
            ]
            .iter()
            .all(|v| v.is_finite())
        {
            return Err(GraphError::InvalidSourceParameters(
                "finite continent settings and scale 2..i32::MAX/4",
            ));
        }
        let cell_seed = seed.next_seed();
        let skipping_seed = seed.next_seed();
        let tectonic_scale = c.continent_scale.wrapping_mul(4);
        let frequency = 1.0 / tectonic_scale as f32;
        let variance_seed = seed.next_seed();
        let warp_scale = u::round_legacy(tectonic_scale as f32 * 0.225);
        let strength = u::round_legacy(tectonic_scale as f32 * 0.33) as f32;
        let warp = Domain::warp(
            n::perlin2_with(
                seed.next_seed(),
                warp_scale,
                c.continent_noise_octaves,
                c.continent_noise_lacunarity,
                c.continent_noise_gain,
            )?,
            n::perlin2_with(
                seed.next_seed(),
                warp_scale,
                c.continent_noise_octaves,
                c.continent_noise_lacunarity,
                c.continent_noise_gain,
            )?,
            Noise::constant(strength)?,
        )?;
        let cliff = n::simplex2(seed.next_seed(), c.continent_scale / 2, 2)?;
        let cliff = Noise::clamp(cliff, Noise::constant(0.1)?, Noise::constant(0.25)?)?;
        let cliff = n::map_range(cliff, 0.0, 1.0)?;
        let cliff_noise = n::frequency(cliff, 1.0 / frequency, 1.0 / frequency)?;
        let bay = n::simplex(seed.next_seed(), 100, 1)?;
        let bay = Noise::add(
            Noise::multiply(bay, Noise::constant(0.1)?)?,
            Noise::constant(0.9)?,
        )?;
        let bay_noise = n::frequency(bay, 1.0 / frequency, 1.0 / frequency)?;
        Ok(Self {
            seed: cell_seed,
            skipping_seed,
            continent_scale: c.continent_scale,
            jitter: c.continent_jitter,
            skip_threshold: c.continent_skipping,
            control_points: world.control_points.make(),
            frequency,
            variance: c.continent_size_variance,
            variance_seed,
            warp,
            cliff_noise,
            bay_noise,
        })
    }

    pub fn apply(&self, cell: &mut Cell, x: f32, z: f32) -> Result<()> {
        let wx = self.warp.get_x(x, z, 0);
        let wz = self.warp.get_z(x, z, 0);
        let x = wx * self.frequency;
        let z = wz * self.frequency;
        // Guard native iteration bounds, rather than emulate the source's possible
        // overflowing/nonterminating integer loops for invalid/nonfinite queries.
        if !x.is_finite()
            || !z.is_finite()
            || x.abs() >= 2_147_483_000.0
            || z.abs() >= 2_147_483_000.0
        {
            return Err(GraphError::InvalidSourceParameters(
                "finite continent grid coordinate within signed32 neighbor bounds",
            ));
        }
        let xi = u::floor_legacy(x);
        let zi = u::floor_legacy(z);
        let mut cell_x = xi;
        let mut cell_z = zi;
        let mut point_x = x;
        let mut point_z = z;
        let mut nearest = f32::MAX;
        for cz in zi - 1..=zi + 1 {
            for cx in xi - 1..=xi + 1 {
                let (vx, vz) = u::cell(self.seed, cx, cz);
                let px = cx as f32 + vx * self.jitter;
                let pz = cz as f32 + vz * self.jitter;
                let distance = dist_sq(x, z, px, pz);
                if distance < nearest {
                    point_x = px;
                    point_z = pz;
                    cell_x = cx;
                    cell_z = cz;
                    nearest = distance;
                }
            }
        }
        nearest = f32::MAX;
        let mut sum_x = 0.0;
        let mut sum_z = 0.0;
        for cz in cell_z - 1..=cell_z + 1 {
            for cx in cell_x - 1..=cell_x + 1 {
                if cx != cell_x || cz != cell_z {
                    let (vx, vz) = u::cell(self.seed, cx, cz);
                    let px = cx as f32 + vx * self.jitter;
                    let pz = cz as f32 + vz * self.jitter;
                    let distance = bisector_distance_sq(x, z, point_x, point_z, px, pz);
                    sum_x += px;
                    sum_z += pz;
                    if distance < nearest {
                        nearest = distance;
                    }
                }
            }
        }
        if self.should_skip(cell_x, cell_z) {
            return Ok(());
        }
        cell.continent_id = cell_value(self.seed, cell_x, cell_z);
        cell.continent_edge = self.distance_value(x, z, cell_x, cell_z, nearest);
        cell.continent_x = (u::lerp_f32(point_x, sum_x / 8.0, 0.35) / self.frequency) as i32;
        cell.continent_z = (u::lerp_f32(point_z, sum_z / 8.0, 0.35) / self.frequency) as i32;
        Ok(())
    }
    pub fn edge_value(&self, x: f32, z: f32) -> Result<f32> {
        let mut cell = Cell::default();
        self.apply(&mut cell, x, z)?;
        Ok(cell.continent_edge)
    }
    pub fn nearest_center(&self, x: f32, z: f32) -> Result<(i32, i32)> {
        let mut cell = Cell::default();
        self.apply(&mut cell, x, z)?;
        Ok((cell.continent_x, cell.continent_z))
    }
    pub fn distance_to_edge(&self, cx: i32, cz: i32, dx: f32, dz: f32) -> Result<f32> {
        let mut distance = self.continent_scale.wrapping_mul(4) as f32;
        for _ in 0..10 {
            let center =
                self.nearest_center(cx as f32 + dx * distance, cz as f32 + dz * distance)?;
            distance += distance;
            if center != (cx, cz) {
                let mut low = 0.0;
                let mut high = distance;
                for _ in 0..50 {
                    let mid = (low + high) / 2.0;
                    let center = self.nearest_center(cx as f32 + dx * mid, cz as f32 + dz * mid)?;
                    if center == (cx, cz) {
                        low = mid;
                    } else {
                        high = mid;
                    }
                    if high - low < 50.0 {
                        break;
                    }
                }
                return Ok(high);
            }
        }
        Ok(distance)
    }
    pub fn distance_to_ocean(&self, cx: i32, cz: i32, dx: f32, dz: f32) -> Result<f32> {
        let mut high = self.distance_to_edge(cx, cz, dx, dz)?;
        let mut low = 0.0;
        for _ in 0..50 {
            let mid = (low + high) / 2.0;
            let edge = self.edge_value(cx as f32 + dx * mid, cz as f32 + dz * mid)?;
            if edge > self.control_points.shallow_ocean {
                low = mid;
            } else {
                high = mid;
            }
            if high - low < 10.0 {
                break;
            }
        }
        Ok(high)
    }
    fn should_skip(&self, x: i32, z: i32) -> bool {
        self.skip_threshold > 0.0
            && (x != 0 || z != 0)
            && cell_value(self.skipping_seed, x, z) < self.skip_threshold
    }
    fn distance_value(&self, x: f32, z: f32, cx: i32, cz: i32, mut distance: f32) -> f32 {
        if self.variance > 0.0 && (cx != 0 || cz != 0) {
            let size = cell_value(self.variance_seed, cx, cz);
            distance *= u::map_clamped_range(size, 0.0, self.variance, self.variance);
        }
        distance = u::sqrt(distance);
        distance = u::map_clamped_range(distance, 0.05, 0.25, 0.2);
        distance = self.coastal_distance_value(x, z, distance);
        if distance < self.control_points.inland && distance >= self.control_points.shallow_ocean {
            distance = self.coastal_distance_value(x, z, distance);
        }
        distance
    }
    fn coastal_distance_value(&self, x: f32, z: f32, mut distance: f32) -> f32 {
        if distance > self.control_points.shallow_ocean && distance < self.control_points.inland {
            let alpha = distance / self.control_points.inland;
            let cliff = self.cliff_noise.compute(x, z, 0);
            distance = u::lerp_f32(distance * cliff, distance, alpha);
            if distance < self.control_points.shallow_ocean {
                distance = self.control_points.shallow_ocean * self.bay_noise.compute(x, z, 0);
            }
        }
        distance
    }
}
fn cell_value(seed: i32, x: i32, z: i32) -> f32 {
    0.5 + u::val_coord_2d(seed, x, z) * 0.5
}
pub fn bisector_distance_sq(x: f32, z: f32, ax: f32, az: f32, bx: f32, bz: f32) -> f32 {
    let mx = (ax + bx) * 0.5;
    let mz = (az + bz) * 0.5;
    let dx = bx - ax;
    let dz = bz - az;
    distance_to_line_sq(x, z, mx, mz, mx + (-dz), mz + dx)
}
pub fn distance_to_line_sq(x: f32, z: f32, ax: f32, az: f32, bx: f32, bz: f32) -> f32 {
    let dx = bx - ax;
    let dz = bz - az;
    let mut v = (x - ax) * dx + (z - az) * dz;
    v /= dx * dx + dz * dz;
    dist_sq(x, z, ax + dx * v, az + dz * v)
}
