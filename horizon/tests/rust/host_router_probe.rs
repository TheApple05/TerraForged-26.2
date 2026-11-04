use horizon_core::density::graph::DensityCompiler;
use horizon_core::density::terraforged_runtime::kernel::{
    kernel_sampler::SamplerOptions, preset_input::CellPresetInput, router::default_router,
};
use horizon_core::density::{DensitySampler, SamplerContext};
use horizon_world::worldgen::{
    density::DensityRegistry,
    generator::terraforged_registry::{self as scope, RegistryState},
    router::{NoiseRouter, RouterField},
};
use serde_json::Value;
use std::{collections::BTreeMap, path::Path, sync::Arc};
fn registry(dir: &Path) -> Result<BTreeMap<String, String>, Box<dyn std::error::Error>> {
    fn visit(
        base: &Path,
        path: &Path,
        out: &mut BTreeMap<String, String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        for entry in std::fs::read_dir(path)? {
            let path = entry?.path();
            if path.is_dir() {
                visit(base, &path, out)?;
            } else if path.extension().is_some_and(|e| e == "json") {
                let key = format!(
                    "minecraft:{}",
                    path.strip_prefix(base)?
                        .with_extension("")
                        .to_string_lossy()
                        .replace('\\', "/")
                );
                out.insert(key, std::fs::read_to_string(path)?);
            }
        }
        Ok(())
    }
    let mut out = BTreeMap::new();
    visit(dir, dir, &mut out)?;
    Ok(out)
}
fn sampler(
    noises: &BTreeMap<String, String>,
    defs: &BTreeMap<String, String>,
    seed: i64,
    name: &str,
) -> Result<Arc<dyn DensitySampler>, Box<dyn std::error::Error>> {
    let registry = DensityRegistry::from_json(noises, defs)?;
    let id = registry.value(name).ok_or("baseline function missing")?;
    Ok(DensityCompiler::new_seeded(registry.into_graph(), seed, false).get_sampler(id)?)
}
fn bits(v: f32) -> String {
    format!("{:08x}", v.to_bits())
}
fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 2 {
        return Err("configuration and own prepared resource root required".into());
    }
    let input: Value = serde_json::from_str(&std::fs::read_to_string(&args[0])?)?;
    let preset = CellPresetInput::from_json(&input["preset"].to_string())?;
    let base = Path::new(&args[1]).join("data/minecraft/worldgen");
    let noises = registry(&base.join("noise"))?;
    let densities = registry(&base.join("density_function"))?;
    let original = densities.clone();
    let plan = default_router(&preset.kernel, &preset.original["caves"])?;
    assert!(plan.materialize(&BTreeMap::new()).is_err());
    let mut bad = preset.kernel.clone();
    bad.world.properties.world_height = 384;
    assert!(default_router(&bad, &preset.original["caves"]).is_err());
    let mut records = 0usize;
    for seed in input["seeds"].as_array().unwrap() {
        let seed = seed.as_i64().unwrap();
        let mut options = SamplerOptions::for_preset(&preset.kernel);
        options.cache_capacity = 32;
        let mut state = RegistryState::from_json_with_options(
            &input["preset"].to_string(),
            &input["noiseRegistry"].to_string(),
            seed,
            false,
            Some(options),
        )?;
        Arc::get_mut(&mut state)
            .ok_or("fresh state unexpectedly shared")?
            .density_definitions = plan.materialize(&densities)?;
        let weak = Arc::downgrade(&state.sampler);
        let vanilla_before = sampler(&noises, &densities, seed, "minecraft:overworld/offset")?
            .sample_value(&mut SamplerContext::cached(), [0, 80, 0])?;
        {
            let compiled = scope::with_state(
                state.clone(),
                || -> Result<_, Box<dyn std::error::Error>> {
                    let mut registry = DensityRegistry::from_json(&noises, &densities)?;
                    let router = NoiseRouter::from_json(&plan.router.to_string(), &mut registry)?;
                    let mut compiler =
                        DensityCompiler::new_seeded(registry.into_graph(), seed, false);
                    Ok(router.compile(&mut compiler)?)
                },
            )??;
            assert!(scope::active().is_none());
            for field in RouterField::ALL {
                let root = compiled.get(field);
                let mut context = SamplerContext::cached().with_planes();
                for (xi, x) in input["coordinates"].as_array().unwrap().iter().enumerate() {
                    for (zi, z) in input["coordinates"].as_array().unwrap().iter().enumerate() {
                        let mut first = None;
                        for (yi, y) in [-64, -32, 0, 32, 64, 96, 128, 160, 192, 224, 256, 288, 319]
                            .into_iter()
                            .enumerate()
                        {
                            let p = [x.as_i64().unwrap() as i32, y, z.as_i64().unwrap() as i32];
                            let value = root.sample_value(&mut context, p)?;
                            assert!(value.is_finite(), "finite router value {field:?} {p:?}");
                            if field == RouterField::ChunkSurfaceLevel {
                                assert!((-64.0..=320.0).contains(&value));
                                if let Some(before) = first {
                                    assert_eq!(before, value.to_bits());
                                } else {
                                    first = Some(value.to_bits());
                                }
                            }
                            println!(
                                "router/{seed}/{}/{xi}/{zi}/{yi}\t{}",
                                field.name(),
                                bits(value)
                            );
                            records += 1;
                        }
                    }
                }
            }
        }
        state.sampler.release_caches()?;
        assert_eq!(state.sampler.cache_counts()?, (0, 0));
        drop(state);
        assert!(weak.upgrade().is_none());
        assert_eq!(original, densities);
        let after = sampler(&noises, &densities, seed, "minecraft:overworld/offset")?
            .sample_value(&mut SamplerContext::cached(), [0, 80, 0])?;
        assert_eq!(vanilla_before.to_bits(), after.to_bits());
    }
    eprintln!(
        "NATIVE_HOST_ROUTER_OK {records} records eightFields finiteValues surfaceHeightDomain surfaceIgnoresY worldLocalDefinitions vanillaUnchanged releasedWorldOwner"
    );
    Ok(())
}
fn main() {
    if let Err(e) = run() {
        eprintln!("{e}");
        std::process::exit(1)
    }
}
