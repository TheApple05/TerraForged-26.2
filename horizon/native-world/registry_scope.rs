// Constructor-local registry bindings; compiled samplers own their terrain data.
use horizon_core::density::terraforged_runtime::kernel::{
    kernel_sampler::{KernelSampler, SamplerOptions},
    noise_registry::NoiseRegistry,
    preset_input::CellPresetInput,
};
use std::{cell::RefCell, sync::Arc};

#[derive(Debug)]
pub struct RegistryState {
    pub sampler: Arc<KernelSampler>,
    pub noises: NoiseRegistry,
    pub world_seed: i64,
    pub legacy_random: bool,
}
impl RegistryState {
    pub fn from_json(
        preset: &str,
        noises: &str,
        seed: i64,
        legacy_random: bool,
    ) -> Result<Arc<Self>, String> {
        Self::from_json_with_options(preset, noises, seed, legacy_random, None)
    }
    pub fn from_json_with_options(
        preset: &str,
        noises: &str,
        seed: i64,
        legacy_random: bool,
        options: Option<SamplerOptions>,
    ) -> Result<Arc<Self>, String> {
        let preset = CellPresetInput::from_json(preset)
            .map_err(|e| e.to_string())?
            .kernel;
        let noises = NoiseRegistry::from_json(noises).map_err(|e| e.to_string())?;
        let ground = noises
            .get("reterraforged:terrain/ground")
            .map_err(|e| e.to_string())?;
        let edge = noises
            .get("reterraforged:climate/biome_edge_shape")
            .map_err(|e| e.to_string())?;
        let options = options.unwrap_or_else(|| SamplerOptions::for_preset(&preset));
        let sampler = KernelSampler::new(&preset, seed as i32, ground, edge, options)
            .map_err(|e| format!("TerraForged registry sampler: {e:?}"))?;
        Ok(Arc::new(Self {
            sampler: Arc::new(sampler),
            noises,
            world_seed: seed,
            legacy_random,
        }))
    }
}
thread_local! {
    static ACTIVE: RefCell<Vec<Arc<RegistryState>>> = const { RefCell::new(Vec::new()) };
}
struct Scope;
impl Drop for Scope {
    fn drop(&mut self) {
        ACTIVE.with(|stack| {
            stack.borrow_mut().pop();
        });
    }
}
pub fn active() -> Option<Arc<RegistryState>> {
    ACTIVE.with(|stack| stack.borrow().last().cloned())
}
pub fn with_state<T>(state: Arc<RegistryState>, run: impl FnOnce() -> T) -> Result<T, String> {
    ACTIVE.with(|stack| -> Result<(), String> {
        let mut stack = stack.borrow_mut();
        if stack.len() >= 8 {
            return Err("TerraForged registry scope nesting exceeds eight".into());
        }
        stack.push(state);
        Ok(())
    })?;
    let _scope = Scope;
    Ok(run())
}
