use horizon_core::density::graph::{DensityCompiler, DensityGraph, GraphBinary, GraphNode};
use horizon_core::density::terraforged_runtime::{self as runtime, kernel};
use horizon_core::density::{DensitySampler, SamplerContext};
use horizon_core::noise::DensityVolume;
use kernel::{
    density_primitives::CellField,
    kernel_sampler::{KernelSampler, SamplerOptions},
    noise_registry::NoiseRegistry,
    preset_input::CellPresetInput,
    serde_json::{self, Value},
};
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
fn run() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args()
        .nth(1)
        .ok_or("configuration path required")?;
    let text = std::fs::read_to_string(path)?;
    let input: Value = serde_json::from_str(&text)?;
    let preset = CellPresetInput::from_json(&input["preset"].to_string())?.kernel;
    let registry = NoiseRegistry::from_json(&input["noiseRegistry"].to_string())?;
    let ground = registry.get("reterraforged:terrain/ground")?;
    let edge = registry.get("reterraforged:climate/biome_edge_shape")?;
    let mut records = 0usize;
    for seed in input["seeds"].as_array().unwrap() {
        let seed = seed.as_i64().unwrap();
        let mut options = SamplerOptions::for_preset(&preset);
        // This oracle revisits 16 tiles for every field. Keep its fixed working
        // set resident; capacity 32 is an explicit bounded QA configuration.
        options.cache_capacity = 32;
        let owner = Arc::new(KernelSampler::new(
            &preset,
            seed as i32,
            ground.clone(),
            edge.clone(),
            options,
        )?);
        let weak = Arc::downgrade(&owner);
        let mut graph = DensityGraph::new();
        let mut roots = BTreeMap::new();
        for field in CellField::ALL {
            let id = runtime::bind_cell(&mut graph, owner.clone(), field)?;
            assert_eq!(graph.domain_axes(id)?, 5);
            assert_eq!(graph.range(id)?, horizon_core::density::Interval::INFINITE);
            roots.insert(field.name().to_owned(), graph.insert(GraphNode::Cache(id))?);
        }
        let height = roots["height"];
        for res in input["resolutions"].as_array().unwrap() {
            let resolution = res.as_i64().unwrap() as i32;
            roots.insert(
                format!("clamp_{resolution}"),
                runtime::bind_clamp(&mut graph, height, resolution, seed, false)?,
            );
        }
        let half = graph.insert(GraphNode::Constant(0.5))?;
        roots.insert(
            "height_plus_half".into(),
            graph.insert(GraphNode::Binary {
                kind: GraphBinary::Add,
                left: height,
                right: half,
            })?,
        );
        for (name, noise) in registry.iter() {
            roots.insert(
                format!("noise_{name}"),
                runtime::bind_noise(&mut graph, noise.clone(), seed as i32)?,
            );
        }
        {
            let mut compiler = DensityCompiler::new_seeded(graph, seed, false);
            let samplers: BTreeMap<String, Arc<dyn DensitySampler>> = roots
                .into_iter()
                .map(|(n, id)| Ok((n, compiler.get_sampler(id)?)))
                .collect::<Result<_, horizon_core::density::graph::CompileError>>()?;
            let mut ctx = SamplerContext::cached().with_planes();
            for (name, sampler) in &samplers {
                // Context cache IDs only belong to this compiler; begin an independent evaluation.
                ctx.reset();
                let mut index = 0;
                for x in input["coordinates"].as_array().unwrap() {
                    for z in input["coordinates"].as_array().unwrap() {
                        for y in input["heights"].as_array().unwrap() {
                            let p = [
                                x.as_i64().unwrap() as i32,
                                y.as_i64().unwrap() as i32,
                                z.as_i64().unwrap() as i32,
                            ];
                            println!(
                                "scalar/{seed}/{name}/{index}\t{}",
                                bits(sampler.sample_value(&mut ctx, p)?)
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
            // One shared compiled graph, independent worker contexts and reversed root order.
            std::thread::scope(|scope| {
                for _ in 0..4 {
                    let samplers = &samplers;
                    scope.spawn(move || {
                        let mut context = SamplerContext::cached();
                        for (_, sampler) in samplers.iter().rev() {
                            context.reset();
                            let first =
                                sampler.sample_value(&mut context, [-129, 80, 128]).unwrap();
                            context.reset();
                            assert_eq!(
                                first.to_bits(),
                                sampler
                                    .sample_value(&mut context, [-129, 80, 128])
                                    .unwrap()
                                    .to_bits()
                            );
                        }
                    });
                }
            });
        }
        owner.release_caches()?;
        assert_eq!(owner.cache_counts()?, (0, 0));
        drop(owner);
        assert!(weak.upgrade().is_none());
    }
    eprintln!(
        "NATIVE_HOST_DENSITY_OK {records} records realHorizonGraph scalar bulk conservativeBounds parallelContexts releasedWorldOwner"
    );
    Ok(())
}
fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1)
    }
}
