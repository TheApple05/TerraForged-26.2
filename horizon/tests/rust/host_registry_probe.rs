use horizon_core::density::graph::DensityCompiler;
use horizon_core::density::terraforged_runtime::kernel::{
    density_primitives::CellField, kernel_sampler::SamplerOptions, preset_input::CellPresetInput,
};
use horizon_core::density::{DensitySampler, SamplerContext};
use horizon_core::noise::DensityVolume;
use horizon_world::worldgen::{
    density::DensityRegistry,
    generator::terraforged_registry::{self as scope, RegistryState},
};
use serde_json::{Value, json};
use std::{collections::BTreeMap, sync::Arc};
fn bits(v: f32) -> String {
    if v.is_nan() {
        "NaN".into()
    } else {
        format!("{:08x}", v.to_bits())
    }
}
fn triple(v: &Value) -> [i32; 3] {
    std::array::from_fn(|i| v[i].as_i64().unwrap() as i32)
}
fn registry_key(label: &str) -> String {
    format!("terraforged:qa/{}", label.replace(':', "/"))
}
fn definitions(input: &Value) -> BTreeMap<String, String> {
    let mut result = BTreeMap::new();
    for field in CellField::ALL {
        result.insert(registry_key(field.name()),json!({"type":"minecraft:cache","input":{"type":"reterraforged:cell","field":field.name()}}).to_string());
    }
    for res in input["resolutions"].as_array().unwrap() {
        result.insert(registry_key(&format!("clamp_{}",res.as_i64().unwrap())),json!({"type":"reterraforged:clamp_to_nearest_unit","function":registry_key("height"),"resolution":res}).to_string());
    }
    result.insert(
        registry_key("height_plus_half"),
        json!({"type":"minecraft:add","left":registry_key("height"),"right":0.5})
            .to_string(),
    );
    for name in input["noiseRegistry"].as_object().unwrap().keys() {
        result.insert(
            registry_key(&format!("noise_{name}")),
            json!({"type":"reterraforged:noise","noise":name}).to_string(),
        );
    }
    result
}
fn depth_limit(state: &Arc<RegistryState>, level: usize) {
    if level == 8 {
        assert!(scope::with_state(state.clone(), || ()).is_err());
    } else {
        scope::with_state(state.clone(), || depth_limit(state, level + 1)).unwrap();
    }
}
fn lifecycle(state: &Arc<RegistryState>) {
    assert!(scope::active().is_none());
    scope::with_state(state.clone(), || {
        assert!(Arc::ptr_eq(&scope::active().unwrap(), state));
        std::thread::scope(|s| {
            s.spawn(|| assert!(scope::active().is_none()));
        });
        scope::with_state(state.clone(), || {
            assert!(Arc::ptr_eq(&scope::active().unwrap(), state))
        })
        .unwrap();
        assert!(Arc::ptr_eq(&scope::active().unwrap(), state));
    })
    .unwrap();
    assert!(scope::active().is_none());
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _ = scope::with_state(state.clone(), || panic!("controlled registry unwind"));
    }));
    assert!(result.is_err());
    assert!(scope::active().is_none());
    depth_limit(state, 0);
    assert!(scope::active().is_none());
}
fn run() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args().nth(1).ok_or("configuration required")?;
    let input: Value = serde_json::from_str(&std::fs::read_to_string(path)?)?;
    let preset = CellPresetInput::from_json(&input["preset"].to_string())?.kernel;
    let mut options = SamplerOptions::for_preset(&preset);
    options.cache_capacity = 32;
    let defs = definitions(&input);
    let empty = BTreeMap::new();
    let mut records = 0;
    assert!(DensityRegistry::from_json(&empty, &defs).is_err());
    for seed in input["seeds"].as_array().unwrap() {
        let seed = seed.as_i64().unwrap();
        let state = RegistryState::from_json_with_options(
            &input["preset"].to_string(),
            &input["noiseRegistry"].to_string(),
            seed,
            false,
            Some(options),
        )?;
        lifecycle(&state);
        let weak = Arc::downgrade(&state.sampler);
        {
            let registry =
                scope::with_state(state.clone(), || DensityRegistry::from_json(&empty, &defs))??;
            assert!(scope::active().is_none());
            let roots: BTreeMap<_, _> = registry
                .entries()
                .map(|(n, id)| (n.to_owned(), id))
                .collect();
            let mut compiler = DensityCompiler::new_seeded(registry.into_graph(), seed, false);
            let mut samplers: BTreeMap<String, Arc<dyn DensitySampler>> = BTreeMap::new();
            let mut labels: Vec<String> =
                CellField::ALL.iter().map(|f| f.name().to_owned()).collect();
            labels.extend(
                input["resolutions"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|r| format!("clamp_{}", r.as_i64().unwrap())),
            );
            labels.push("height_plus_half".into());
            labels.extend(
                input["noiseRegistry"]
                    .as_object()
                    .unwrap()
                    .keys()
                    .map(|n| format!("noise_{n}")),
            );
            for label in labels {
                samplers.insert(
                    label.clone(),
                    compiler.get_sampler(roots[&registry_key(&label)])?,
                );
            }
            let mut ctx = SamplerContext::cached().with_planes();
            for (name, sampler) in &samplers {
                ctx.reset();
                let mut index = 0;
                for x in input["coordinates"].as_array().unwrap() {
                    for z in input["coordinates"].as_array().unwrap() {
                        for y in input["heights"].as_array().unwrap() {
                            println!(
                                "scalar/{seed}/{name}/{index}\t{}",
                                bits(sampler.sample_value(
                                    &mut ctx,
                                    [
                                        x.as_i64().unwrap() as i32,
                                        y.as_i64().unwrap() as i32,
                                        z.as_i64().unwrap() as i32
                                    ]
                                )?)
                            );
                            index += 1;
                            records += 1;
                        }
                    }
                }
                for (vi, v) in input["volumes"].as_array().unwrap().iter().enumerate() {
                    ctx.reset();
                    let volume = DensityVolume::new(
                        triple(&v["sizes"]),
                        triple(&v["minimum"]),
                        triple(&v["steps"]),
                    )?;
                    let mut buffer = vec![f32::NAN; volume.size()];
                    sampler.sample_volume(&mut ctx, &mut buffer, volume)?;
                    for (i, value) in buffer.iter().enumerate() {
                        println!("volume/{seed}/{name}/{vi}/{i}\t{}", bits(*value));
                        records += 1;
                    }
                }
            }
            let bad = BTreeMap::from([(
                "terraforged:bad".into(),
                r#"{"type":"reterraforged:cell","field":"invalid"}"#.into(),
            )]);
            assert!(
                scope::with_state(state.clone(), || DensityRegistry::from_json(&empty, &bad))?
                    .is_err()
            );
            assert!(scope::active().is_none());
            assert!(
                scope::with_state(state.clone(), || DensityRegistry::from_json(&empty, &defs))?
                    .is_ok()
            );
        }
        state.sampler.release_caches()?;
        drop(state);
        assert!(weak.upgrade().is_none());
        assert!(DensityRegistry::from_json(&empty, &defs).is_err());
    }
    let vanilla = DensityRegistry::from_json(
        &empty,
        &BTreeMap::from([("minecraft:test".into(), "3.0".into())]),
    )?;
    let value = vanilla.value("minecraft:test").unwrap();
    let sampler = DensityCompiler::new(vanilla.into_graph()).get_sampler(value)?;
    assert_eq!(
        sampler.sample_value(&mut SamplerContext::default(), [0; 3])?,
        3.0
    );
    eprintln!(
        "NATIVE_HOST_REGISTRY_OK {records} records realHorizonRegistry scalar bulk isolatedScopes boundedNesting threadIsolation panicCleanup errorsRecover vanillaUnchanged releasedWorldOwner"
    );
    Ok(())
}
fn main() {
    if let Err(e) = run() {
        eprintln!("{e}");
        std::process::exit(1)
    }
}
