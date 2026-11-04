//! Original Heightmap terrain, river and climate stages. Tile filters remain separate.
use crate::{
    cell::Cell,
    climate::Climate,
    continent::AdvancedContinent,
    graph::{CurveFunction, GraphError, Noise as N},
    layout::Levels,
    noise_util::round_java,
    noises as n,
    perlin::Interpolation,
    populator::Populator as P,
    preset::{ContinentType, ControlPoints, KernelPreset},
    region::{RegionConfig, RegionModule},
    river_cache::RiverCache,
    river_generator::RiverGenerator,
    rivermap::Rivermap,
    seed::Seed,
    terrain::TerrainRegistry,
    terrain_factories as f, terrain_provider,
    worley::{DistanceFunction, EdgeFunction},
};
use std::sync::Arc;
type R<T> = Result<T, GraphError>;
#[derive(Clone, Debug)]
pub struct Heightmap {
    terrain: P,
    region: RegionModule,
    pub continent: Arc<AdvancedContinent>,
    pub climate: Climate,
    pub levels: Levels,
    pub control_points: ControlPoints,
    terrain_frequency: f32,
    beach_noise: N,
    pub terrains: TerrainRegistry,
    pub seed_cursor_after: i32,
    rivers: Arc<RiverCache>,
}
impl Heightmap {
    /// Arguments are resolved registry values; source HolderHolder bindings are
    /// constructed here in the same places as PresetNoiseData.getNoise.
    pub fn new(preset: &KernelPreset, kernel_seed: i32, ground: N, biome_edge_shape: N) -> R<Self> {
        if preset.world.continent.continent_type != ContinentType::MultiImproved {
            return Err(GraphError::InvalidSourceParameters(
                "native Heightmap currently supports MULTI_IMPROVED only",
            ));
        }
        let settings = &preset.terrain;
        let general = &settings.general;
        let climate = &preset.climate;
        if !general.global_horizontal_scale.is_finite()
            || general.global_horizontal_scale <= 0.0
            || general.terrain_region_size < 4
            || round_java(250.0 * settings.steppe.horizontal_scale) / 4 < 1
            || round_java(250.0 * settings.plains.horizontal_scale) / 4 < 1
            || round_java(410.0 * settings.mountains.horizontal_scale) < 1
            || climate.biome_shape.biome_size < 1
            || climate.biome_shape.biome_warp_scale < 1
            || climate.biome_shape.macro_noise_size < 1
            || climate.temperature.scale < 1
            || climate.moisture.scale < 1
        {
            return Err(GraphError::InvalidSourceParameters(
                "positive finite native terrain/climate scales with nonzero derived warp scales",
            ));
        }
        let mut seed = Seed::new(kernel_seed);
        let mut terrains = TerrainRegistry::new();
        let world = &preset.world;
        let control_points = world.control_points.make();
        let levels = Levels::new(
            world.properties.terrain_scaler(),
            world.properties.sea_level,
        );
        let mut region_seed = seed.offset(8934);
        let config = RegionConfig {
            seed: seed.root().wrapping_add(789124),
            scale: general.terrain_region_size,
            warp_x: n::simplex(region_seed.next_seed(), 400, 1)?,
            warp_z: n::simplex(region_seed.next_seed(), 400, 1)?,
            warp_strength: 200.0,
        };
        let region = RegionModule::new(
            config.seed,
            config.scale,
            config.warp_x.clone(),
            config.warp_z.clone(),
            config.warp_strength,
        )?;
        let terrain_frequency = 1.0 / general.global_horizontal_scale;
        let mut mountain_seed = seed.offset(general.terrain_seed_offset);
        let mountain_shape = n::worley_edge_with(
            mountain_seed.next_seed(),
            1000,
            EdgeFunction::Distance2Add,
            DistanceFunction::Euclidean,
        )?;
        let mountain_shape =
            n::warp_perlin(mountain_shape, mountain_seed.next_seed(), 333, 2, 250.0)?;
        let mountain_shape = N::curve(
            mountain_shape,
            CurveFunction::Interpolation(Interpolation::Curve3),
        )?;
        let mountain_shape = N::clamp(mountain_shape, N::constant(0.0)?, N::constant(0.9)?)?;
        let mountain_shape = n::map_range(mountain_shape, 0.0, 1.0)?;
        let ground = N::holder(ground)?;
        let regions = P::selector(terrain_provider::generate(
            &seed,
            settings,
            &config,
            &levels,
            ground.clone(),
            &mut terrains,
        )?)?;
        let borders = P::terrain(f::make_border(
            &mut seed,
            ground.clone(),
            &settings.plains,
            &settings.steppe,
            general.global_vertical_scale,
        )?)?;
        let terrain_blend = P::region_lerp(borders, regions)?;
        let mountains = P::terrain(f::make_mountain_chain(
            &mut mountain_seed,
            ground,
            &settings.mountains,
            general.global_vertical_scale,
            general.fancy_mountains,
        )?)?;
        let continent = Arc::new(AdvancedContinent::new(&mut seed, world)?);
        let rivers = Arc::new(RiverCache::new(
            RiverGenerator::new(continent.clone(), preset, levels, seed.root())?,
            32,
        )?);
        let climate = Climate::new(
            &mut seed,
            continent.clone(),
            world,
            climate,
            levels,
            N::holder(biome_edge_shape)?,
        )?;
        let land = P::blend(mountain_shape, terrain_blend, mountains, 0.3, 0.8, 0.575)?;
        let deep = f::make_deep_ocean(seed.next_seed(), levels.water)?;
        let shallow = f::make_shallow_ocean(&levels)?;
        let coast = f::make_coast(&levels)?;
        let oceans = P::continent3(
            deep,
            shallow,
            coast,
            control_points.deep_ocean,
            control_points.shallow_ocean,
            control_points.coast,
            Interpolation::Curve3,
        )?;
        let terrain = P::continent2(
            oceans,
            land,
            control_points.shallow_ocean,
            control_points.inland,
            Interpolation::Linear,
        )?;
        let beach_noise = N::multiply(
            n::perlin2(seed.next_seed(), 20, 1)?,
            N::constant(levels.scale_i32(5))?,
        )?;
        Ok(Self {
            terrain,
            region,
            continent,
            climate,
            levels,
            control_points,
            terrain_frequency,
            beach_noise,
            terrains,
            seed_cursor_after: seed.get(),
            rivers,
        })
    }
    pub fn get_rivermap(&self, x: i32, z: i32) -> R<Arc<Rivermap>> {
        self.rivers.get(x, z)
    }
    pub fn river_cache_len(&self) -> R<usize> {
        self.rivers.len()
    }
    pub fn clear_river_cache(&self) -> R<()> {
        self.rivers.clear()
    }
    pub fn apply_rivers(&self, cell: &mut Cell, x: f32, z: f32, map: &Rivermap) -> R<()> {
        if !x.is_finite() || !z.is_finite() {
            return Err(GraphError::InvalidSourceParameters(
                "finite native river coordinates",
            ));
        }
        map.apply(cell, x, z, &self.terrains)?;
        crate::volcano::Volcano::modify_type(cell, &self.levels);
        Ok(())
    }
    pub fn apply(&self, cell: &mut Cell, x: f32, z: f32, apply_climate: bool) -> R<()> {
        self.apply_terrain(cell, x, z)?;
        let map = self.get_rivermap(cell.continent_x, cell.continent_z)?;
        self.apply_rivers(cell, x, z, &map)?;
        self.apply_climate(cell, x, z, apply_climate)
    }
    pub fn apply_terrain(&self, cell: &mut Cell, x: f32, z: f32) -> R<()> {
        if !x.is_finite() || !z.is_finite() {
            return Err(GraphError::InvalidSourceParameters(
                "finite native cell coordinates",
            ));
        }
        cell.terrain = 8;
        cell.beach_noise = self.beach_noise.compute(x, z, 0);
        self.continent.apply(cell, x, z)?;
        self.region.apply(cell, x, z);
        self.terrain
            .apply(cell, x * self.terrain_frequency, z * self.terrain_frequency)
    }
    pub fn apply_climate(&self, cell: &mut Cell, x: f32, z: f32, apply_climate: bool) -> R<()> {
        if cell.river_mask < 0.675 {
            cell.erosion = 0.445;
            cell.weirdness = 0.34;
        }
        let terrain =
            self.terrains
                .get(cell.terrain)
                .ok_or(GraphError::InvalidSourceParameters(
                    "owned heightmap terrain ID",
                ))?;
        if terrain.is_river() {
            cell.erosion = -0.05;
            cell.weirdness = -0.03;
        }
        if terrain.is_lake() && cell.height < self.levels.water {
            cell.erosion = f::erosion_mid(4);
            cell.weirdness = -0.03;
        }
        if terrain.is_wetland() {
            cell.erosion = f::erosion_mid(6);
            cell.weirdness = (-0.05f32 + 0.05f32) / 2.0;
        }
        self.climate
            .apply(cell, x, z, apply_climate, &self.terrains)?;
        if cell.river_mask >= 0.675 && cell.macro_biome_id > 0.5 {
            cell.weirdness = -cell.weirdness;
        }
        Ok(())
    }
}
