//! MIT ReTerraForged Climate / ClimateModule, bound to the qualified advanced continent.
use crate::{
    biome,
    cell::Cell,
    continent::AdvancedContinent,
    graph::{GraphError, Noise as N, java_max, java_min},
    layout::Levels,
    noise_util as u, noises as n,
    preset::{ClimateSettings, ControlPoints, WorldSettings},
    seed::Seed,
    terrain::TerrainRegistry,
    worley::{DistanceFunction, EdgeFunction},
};
use std::sync::Arc;
type R<T> = Result<T, GraphError>;
#[derive(Clone, Debug)]
pub struct ClimateModule {
    seed: i32,
    biome_freq: f32,
    warp_strength: f32,
    warp_x: N,
    warp_z: N,
    moisture: N,
    temperature: N,
    macro_biome_noise: N,
    continent: Arc<AdvancedContinent>,
    control_points: ControlPoints,
    levels: Levels,
}
#[derive(Clone, Debug)]
pub struct Climate {
    pub random_seed: i32,
    offset_x: N,
    offset_z: N,
    offset_distance: i32,
    levels: Levels,
    pub biome_noise: ClimateModule,
}
impl ClimateModule {
    pub fn new(
        seed: &mut Seed,
        continent: Arc<AdvancedContinent>,
        world: &WorldSettings,
        settings: &ClimateSettings,
        levels: Levels,
    ) -> R<Self> {
        let size = settings.biome_shape.biome_size;
        let temp_scaler = settings.temperature.scale as f32;
        let moist_scaler = settings.moisture.scale as f32 * 2.5;
        let biome_freq = 1.0 / size as f32;
        let moisture_size = moist_scaler * size as f32;
        let temperature_size = temp_scaler * size as f32;
        let moist_scale = u::round_legacy(moisture_size * biome_freq);
        let temp_scale = u::round_legacy(temperature_size * biome_freq);
        let warp_scale = settings.biome_shape.biome_warp_scale;
        let cell_seed = seed.next_seed();
        let warp_x = N::add(
            n::simplex(seed.next_seed(), warp_scale, 2)?,
            N::constant(-0.5)?,
        )?;
        let warp_z = N::add(
            n::simplex(seed.next_seed(), warp_scale, 2)?,
            N::constant(-0.5)?,
        )?;
        let mut moisture_seed = seed.offset(settings.moisture.seed_offset);
        let source = n::simplex(moisture_seed.next_seed(), moist_scale, 1)?;
        let source = N::clamp(source, N::constant(0.125)?, N::constant(0.875)?)?;
        let source = n::map_range(source, 0.0, 1.0)?;
        let source = n::frequency(source, 0.5, 1.0)?;
        let moisture = N::legacy_moisture(source, settings.moisture.falloff)?;
        let moisture = settings.moisture.apply(moisture)?;
        let moisture = n::warp_perlin(
            moisture,
            moisture_seed.next_seed(),
            1.max(moist_scale / 2),
            1,
            moist_scale as f32 / 4.0,
        )?;
        let moisture = n::warp_perlin(
            moisture,
            moisture_seed.next_seed(),
            1.max(moist_scale / 6),
            2,
            moist_scale as f32 / 12.0,
        )?;
        let mut temp_seed = seed.offset(settings.temperature.seed_offset);
        let temperature =
            N::legacy_temperature(1.0 / temp_scale as f32, settings.temperature.falloff)?;
        let temperature = settings.temperature.apply(temperature)?;
        let temperature = n::warp_perlin(
            temperature,
            temp_seed.next_seed(),
            temp_scale.wrapping_mul(4),
            2,
            temp_scale.wrapping_mul(4) as f32,
        )?;
        let temperature = n::warp_perlin(
            temperature,
            temp_seed.next_seed(),
            temp_scale,
            1,
            temp_scale as f32,
        )?;
        let macro_biome_noise = n::worley(seed.next_seed(), settings.biome_shape.macro_noise_size)?;
        Ok(Self {
            seed: cell_seed,
            biome_freq,
            warp_strength: settings.biome_shape.biome_warp_strength as f32,
            warp_x,
            warp_z,
            moisture,
            temperature,
            macro_biome_noise,
            continent,
            control_points: world.control_points.make(),
            levels,
        })
    }
    pub fn apply(
        &self,
        cell: &mut Cell,
        mut x: f32,
        mut z: f32,
        _original_x: f32,
        _original_z: f32,
        mask: bool,
        terrains: &TerrainRegistry,
    ) -> R<()> {
        let ox = self.warp_x.compute(x, z, 0) * self.warp_strength;
        let oz = self.warp_z.compute(x, z, 0) * self.warp_strength;
        x += ox;
        z += oz;
        x *= self.biome_freq;
        z *= self.biome_freq;
        let xr = u::floor_legacy(x);
        let zr = u::floor_legacy(z);
        let mut cell_x = xr;
        let mut cell_z = zr;
        let mut center_x = x;
        let mut center_z = z;
        let mut nearest = 999999.0f32;
        let mut second = 999999.0f32;
        for dz in -1i32..=1 {
            for dx in -1i32..=1 {
                let cx = xr.wrapping_add(dx);
                let cz = zr.wrapping_add(dz);
                let (vx, vz) = u::cell(self.seed, cx, cz);
                let px = cx as f32 + vx;
                let pz = cz as f32 + vz;
                let distance = DistanceFunction::Euclidean.apply(px - x, pz - z);
                if distance < nearest {
                    second = nearest;
                    nearest = distance;
                    center_x = px;
                    center_z = pz;
                    cell_x = cx;
                    cell_z = cz;
                } else if distance < second {
                    second = distance;
                }
            }
        }
        cell.biome_region_id =
            u::map_clamped_range(u::val_coord_2d(self.seed, cell_x, cell_z), -1.0, 1.0, 2.0);
        cell.region_moisture = self.moisture.compute(center_x, center_z, 0);
        cell.region_temperature = self.temperature.compute(center_x, center_z, 0);
        cell.macro_biome_id = self.macro_biome_noise.compute(center_x, center_z, 0);
        let px = u::floor_legacy(center_x / self.biome_freq);
        let pz = u::floor_legacy(center_z / self.biome_freq);
        let continent_edge = self.continent.edge_value(px as f32, pz as f32)?;
        if mask {
            let edge = EdgeFunction::Distance2Div;
            let value = edge.apply(nearest, second);
            cell.biome_region_edge =
                1.0 - u::map_clamped_range(value, edge.min(), edge.max(), edge.range());
            let terrain = terrains
                .get(cell.terrain)
                .ok_or(GraphError::InvalidSourceParameters(
                    "owned climate terrain ID",
                ))?;
            if terrain.is_overground()
                && !terrain.overrides_coast()
                && continent_edge <= self.control_points.coast_marker
            {
                cell.terrain = 3;
            }
        }
        cell.region_moisture = self.modify_moisture(cell.region_moisture, continent_edge);
        cell.biome =
            biome::get_curve(cell.region_temperature, cell.region_moisture).map_err(|_| {
                GraphError::InvalidSourceParameters(
                    "climate category lookup outside source palette",
                )
            })?;
        cell.region_temperature = self.modify_temperature(cell.height, cell.region_temperature);
        (cell.temperature, cell.moisture) = cell
            .biome
            .temperature_moisture(cell.biome_region_id)
            .map_err(|_| {
            GraphError::InvalidSourceParameters("climate identity outside source pairs")
        })?;
        Ok(())
    }
    fn modify_temperature(&self, mut height: f32, temp: f32) -> f32 {
        if height > 0.75 {
            return java_max(0.0, temp - 0.05);
        }
        if height > 0.45 {
            let delta = (height - 0.45) / 0.3;
            return java_max(0.0, temp - delta * 0.05);
        }
        height = java_max(self.levels.ground, height);
        if height >= self.levels.ground {
            let delta = 1.0 - (height - self.levels.ground) / (0.45 - self.levels.ground);
            return java_min(1.0, temp + delta * 0.05);
        }
        temp
    }
    fn modify_moisture(&self, moisture: f32, edge: f32) -> f32 {
        let limit = 0.75;
        let range = 1.0 - limit;
        if edge < limit {
            let alpha = (limit - edge) / range;
            let multiplier = 1.0 + alpha * range;
            u::clamp_f32(moisture * multiplier, 0.0, 1.0)
        } else {
            let alpha = (edge - limit) / range;
            let multiplier = 1.0 - alpha * range;
            moisture * multiplier
        }
    }
}
impl Climate {
    pub fn new(
        seed: &mut Seed,
        continent: Arc<AdvancedContinent>,
        world: &WorldSettings,
        settings: &ClimateSettings,
        levels: Levels,
        biome_edge_shape: N,
    ) -> R<Self> {
        let biome_noise = ClimateModule::new(seed, continent, world, settings, levels)?;
        let random_seed = seed.next_seed();
        let offset_x = N::shift_seed(biome_edge_shape.clone(), seed.next_seed())?;
        let offset_z = N::shift_seed(biome_edge_shape, seed.next_seed())?;
        Ok(Self {
            random_seed,
            offset_x,
            offset_z,
            offset_distance: settings.biome_edge_shape.strength,
            levels,
            biome_noise,
        })
    }
    pub fn apply(
        &self,
        cell: &mut Cell,
        mut x: f32,
        mut z: f32,
        apply_climate: bool,
        terrains: &TerrainRegistry,
    ) -> R<()> {
        if apply_climate {
            self.biome_noise.apply(cell, x, z, x, z, true, terrains)?;
        }
        let edge_blend = 0.4;
        if cell.height <= self.levels.water {
            if cell.terrain == 3 {
                cell.terrain = 2;
            }
        } else if apply_climate && (cell.biome_region_edge < edge_blend || cell.terrain == 15) {
            let modifier =
                1.0 - u::map_clamped_range(cell.biome_region_edge, 0.0, edge_blend, edge_blend);
            let distance = self.offset_distance as f32 * modifier;
            let dx = self.offset_x.compute(x, z, 0) * distance;
            let dz = self.offset_z.compute(x, z, 0) * distance;
            let ox = x;
            let oz = z;
            x += dx;
            z += dz;
            self.biome_noise
                .apply(cell, x, z, ox, oz, false, terrains)?;
        }
        Ok(())
    }
}
