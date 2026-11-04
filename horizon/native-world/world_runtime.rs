// Native procedural pipeline. No snapshots or Java processes participate.
use super::*;
use horizon_core::density::terraforged_runtime::kernel::{
    kernel_sampler::KernelSampler, preset_input::CellPresetInput, router::default_router,
};
use std::sync::Weak;

pub struct OpenedNative {
    pub generator: OverworldGenerator,
    /// Non-owning diagnostic handle for quiescent cache/release checks.
    pub kernel: Weak<KernelSampler>,
}

pub fn open(
    data: Arc<GeneratorData>,
    seed: i64,
    preset_json: &str,
    noises_json: &str,
    with_structures: bool,
) -> Result<OpenedNative, GeneratorError> {
    let preset = CellPresetInput::from_json(preset_json).map_err(|e| resource(e.to_string()))?;
    let plan = default_router(&preset.kernel, &preset.original["caves"])
        .map_err(|e| resource(e.to_string()))?;
    let mut state =
        terraforged_registry::RegistryState::from_json(preset_json, noises_json, seed, false)
            .map_err(resource)?;
    Arc::get_mut(&mut state)
        .ok_or_else(|| resource("new TerraForged state unexpectedly shared"))?
        .density_definitions = plan
        .materialize(&data.densities)
        .map_err(|e| resource(e.to_string()))?;
    let normal: Value = serde_json::from_str(
        data.world_presets
            .get("minecraft:normal")
            .ok_or_else(|| resource("26.3 normal world template missing"))?,
    )?;
    let stem = &normal["dimensions"]["minecraft:overworld"];
    let generator = stem["generator"].clone();
    if generator["type"].as_str() != Some("minecraft:noise") {
        return Err(resource("26.3 overworld template is not a noise generator"));
    }
    let mut settings: Value = match &generator["settings"] {
        Value::String(name) => serde_json::from_str(
            data.settings
                .get(name)
                .ok_or_else(|| resource(format!("missing native base settings {name}")))?,
        )?,
        value => value.clone(),
    };
    let level = overrides::LevelHeight::of_type(&stem["type"], &data.dimension_types)
        .map(|h| (h.min_y, h.height))
        .ok_or_else(|| resource("native overworld dimension height missing"))?;
    if level != (-64, 384)
        || settings["noise"]["min_y"].as_i64() != Some(-64)
        || settings["noise"]["height"].as_i64() != Some(384)
        || settings["sea_level"].as_i64() != Some(63)
    {
        return Err(resource(
            "native default pipeline requires 26.3 overworld bounds -64..320 and sea level 63",
        ));
    }
    let legacy = settings["legacy_random_source"].as_bool().unwrap_or(false);
    Arc::get_mut(&mut state)
        .ok_or_else(|| resource("new native state unexpectedly shared"))?
        .legacy_random = legacy;
    settings["noise_router"] = plan.router;
    let kernel = state.sampler.clone();
    let weak = Arc::downgrade(&kernel);
    let inner = terraforged_registry::with_state(
        state,
        || -> Result<Arc<dyn SeededPipeline>, GeneratorError> {
            if legacy {
                Ok(Arc::new(Pipeline::new(
                    data,
                    seed,
                    LegacyRandom::new(seed),
                    &generator,
                    &settings,
                    with_structures,
                    Some(level),
                )?))
            } else {
                Ok(Arc::new(Pipeline::new(
                    data,
                    seed,
                    XoroshiroRandom::new(seed),
                    &generator,
                    &settings,
                    with_structures,
                    Some(level),
                )?))
            }
        },
    )
    .map_err(resource)??;
    Ok(OpenedNative {
        generator: OverworldGenerator {
            pipeline: Arc::new(NativePipeline { inner, kernel }),
            noise_clamped: None,
        },
        kernel: weak,
    })
}

struct NativePipeline {
    inner: Arc<dyn SeededPipeline>,
    kernel: Arc<KernelSampler>,
}

// Pipeline forwarding follows below. Regenerate with the source-only helper.
// Trait source SHA256: ef3b3fc18893a495b7c07644f32e86b885e5c232a2e85bb9bca8e4f2b8c453e1
impl SeededPipeline for NativePipeline {
    fn generate(
        &self,
        worker: &mut WorkerState,
        pos: ChunkPos,
    ) -> Result<ProtoChunk, GeneratorError> {
        self.inner.generate(worker, pos)
    }
    fn decorate(
        &self,
        worker: &mut WorkerState,
        region: &mut [ProtoChunk; 9],
    ) -> Result<DecorateStats, GeneratorError> {
        self.inner.decorate(worker, region)
    }
    fn data(&self) -> &Arc<GeneratorData> {
        self.inner.data()
    }
    fn cache_census(&self) -> CacheCensus {
        self.inner.cache_census()
    }
    fn release_caches(&self) {
        self.inner.release_caches();
        if let Err(error) = self.kernel.release_caches() {
            log::error!("TerraForged cache release failed: {error:?}");
        }
    }
    fn end_island_origins(&self, chunk: ChunkPos) -> Vec<[i32; 3]> {
        self.inner.end_island_origins(chunk)
    }
    fn sea_level(&self) -> i32 {
        self.inner.sea_level()
    }
    fn level(&self) -> (i32, i32) {
        self.inner.level()
    }
    fn surface_sample(
        &self,
        worker: &mut WorkerState,
        x: i32,
        z: i32,
        spans: Option<&mut Vec<(i32, i32, BlockStateId)>>,
    ) -> Option<SurfaceSample> {
        self.inner.surface_sample(worker, x, z, spans)
    }
    fn lod_grid(
        &self,
        worker: &mut WorkerState,
        x: i32,
        z: i32,
        width: i32,
        side: usize,
        approximate: bool,
    ) -> Result<Vec<LodColumn>, GeneratorError> {
        self.inner.lod_grid(worker, x, z, width, side, approximate)
    }
    fn structure_starts(
        &self,
        worker: &mut WorkerState,
        pos: ChunkPos,
    ) -> Vec<Arc<crate::worldgen::structure::start::StructureStart>> {
        self.inner.structure_starts(worker, pos)
    }
    fn structure_start(
        &self,
        worker: &mut WorkerState,
        structure: &str,
        chunk: ChunkPos,
    ) -> Option<Arc<crate::worldgen::structure::start::StructureStart>> {
        self.inner.structure_start(worker, structure, chunk)
    }
    fn unsupported_features(&self) -> Vec<(String, String)> {
        self.inner.unsupported_features()
    }
    fn use_saved_start(
        &self,
        worker: &mut WorkerState,
        chunk: ChunkPos,
        structure: &str,
        pieces: &[serde_json::Value],
    ) -> Result<(), String> {
        self.inner.use_saved_start(worker, chunk, structure, pieces)
    }
    fn forget_structure_starts(&self) {
        self.inner.forget_structure_starts()
    }
    fn ring_positions(&self, worker: &mut WorkerState, set: &str) -> Option<Vec<ChunkPos>> {
        self.inner.ring_positions(worker, set)
    }
    fn spawn_position(&self) -> Result<[i32; 2], GeneratorError> {
        self.inner.spawn_position()
    }
    fn region_random(
        &self,
        center: ChunkPos,
    ) -> Box<dyn crate::worldgen::structure::legacy::PieceRandom> {
        self.inner.region_random(center)
    }
    fn locate(
        &self,
        worker: &mut WorkerState,
        names: &[String],
        from: [i32; 3],
        radius: i32,
        skip: &dyn Fn(&str, ChunkPos) -> bool,
    ) -> Option<(String, [i32; 3])> {
        self.inner.locate(worker, names, from, radius, skip)
    }
    fn set_old_world(&self, world: Arc<dyn crate::worldgen::blending::OldWorld>) {
        self.inner.set_old_world(world)
    }
    fn debug_density_line(&self, feet: [i32; 3]) -> Option<String> {
        self.inner.debug_density_line(feet)
    }
    fn find_closest_biome(
        &self,
        worker: &mut WorkerState,
        search: &BiomeSearch<'_>,
    ) -> Option<(String, [i32; 3])> {
        self.inner.find_closest_biome(worker, search)
    }
    fn noise_biomes(&self, worker: &mut WorkerState, quarts: &[[i32; 3]]) -> Vec<Option<String>> {
        self.inner.noise_biomes(worker, quarts)
    }
    fn jigsaw_pieces(
        &self,
        worker: &mut WorkerState,
        query: &JigsawQuery<'_>,
    ) -> Option<JigsawPieces> {
        self.inner.jigsaw_pieces(worker, query)
    }
}
