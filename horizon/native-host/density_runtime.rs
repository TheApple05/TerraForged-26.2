// Owned procedural density bindings. This module alone does not enable a world.
use super::graph::{CompileError, DensityCompiler, DensityGraph, FunctionId};
use super::{DensitySampler, Interval, SampleError, SamplerContext, check_buffer};
use crate::noise::DensityVolume;
use kernel::{density_primitives::CellField, graph::Noise, kernel_sampler::KernelSampler};
use std::sync::{Arc, Mutex};
pub use terraforged_kernel as kernel;

#[derive(Debug)]
struct CellDensity {
    owner: Arc<KernelSampler>,
    field: CellField,
}
impl DensitySampler for CellDensity {
    fn sample_value(&self, _: &mut SamplerContext, p: [i32; 3]) -> Result<f32, SampleError> {
        let cell = self
            .owner
            .sample(p[0], p[2])
            .map_err(|_| SampleError::External("TerraForged tile generation failed"))?;
        let h = self.owner.heightmap();
        self.field
            .read(&cell, h.levels, h.control_points, &h.terrains)
            .map_err(|_| SampleError::External("TerraForged cell field conversion failed"))
    }
}
pub fn bind_cell(
    graph: &mut DensityGraph,
    owner: Arc<KernelSampler>,
    field: CellField,
) -> Result<FunctionId, CompileError> {
    // Legacy min/max metadata is not a proof. Keep optimizer intervals conservative.
    graph.bind(
        Arc::new(CellDensity { owner, field }),
        Interval::INFINITE,
        5,
    )
}

#[derive(Debug)]
struct NoiseDensity {
    noise: Noise,
    seed: i32,
}
impl DensitySampler for NoiseDensity {
    fn sample_value(&self, _: &mut SamplerContext, p: [i32; 3]) -> Result<f32, SampleError> {
        Ok(kernel::density_primitives::noise_value(&self.noise, p, self.seed) as f32)
    }
}
pub fn bind_noise(
    graph: &mut DensityGraph,
    noise: Noise,
    seed: i32,
) -> Result<FunctionId, CompileError> {
    graph.bind(
        Arc::new(NoiseDensity { noise, seed }),
        Interval::INFINITE,
        5,
    )
}

/// A separately compiled child has its own cache IDs. Never borrow its caller's
/// cache namespace. Only immutable world bindings and evaluation flags cross.
#[derive(Debug, Default)]
struct ChildContexts(Mutex<Vec<SamplerContext>>);
impl ChildContexts {
    fn with<T>(
        &self,
        parent: &SamplerContext,
        run: impl FnOnce(&mut SamplerContext) -> Result<T, SampleError>,
    ) -> Result<T, SampleError> {
        let mut child = self
            .0
            .lock()
            .map_err(|_| SampleError::External("TerraForged child context lock poisoned"))?
            .pop()
            .unwrap_or_default();
        child.reset();
        // Match the caller's cache mode, using fresh IDs in the child's own
        // namespace. A pooled Default context otherwise silently stays uncached.
        child.caches = parent.caches.as_ref().map(|_| Vec::new());
        child.fields = parent.fields.clone();
        child.sparse = parent.sparse;
        child.planes = parent.planes.as_ref().map(|_| Default::default());
        let result = run(&mut child);
        child.reset();
        let mut pool = self
            .0
            .lock()
            .map_err(|_| SampleError::External("TerraForged child context lock poisoned"))?;
        if pool.len() < 8 {
            pool.push(child);
        }
        result
    }
}
#[derive(Debug)]
struct ClampDensity {
    input: Arc<dyn DensitySampler>,
    resolution: i32,
    contexts: ChildContexts,
}
impl DensitySampler for ClampDensity {
    fn sample_value(&self, parent: &mut SamplerContext, p: [i32; 3]) -> Result<f32, SampleError> {
        self.contexts.with(parent, |child| {
            let value = self.input.sample_value(child, p)?;
            Ok(
                kernel::density_primitives::clamp_to_nearest_unit(value as f64, self.resolution)
                    as f32,
            )
        })
    }
    fn sample_volume(
        &self,
        parent: &mut SamplerContext,
        buffer: &mut [f32],
        volume: DensityVolume,
    ) -> Result<(), SampleError> {
        check_buffer(buffer, volume)?;
        self.contexts.with(parent, |child| {
            self.input
                .sample_volume(child, &mut buffer[..volume.size()], volume)
        })?;
        for value in &mut buffer[..volume.size()] {
            *value =
                kernel::density_primitives::clamp_to_nearest_unit(*value as f64, self.resolution)
                    as f32;
        }
        Ok(())
    }
}
pub fn bind_clamp(
    graph: &mut DensityGraph,
    input: FunctionId,
    resolution: i32,
    world_seed: i64,
    legacy_random: bool,
) -> Result<FunctionId, CompileError> {
    let axes = graph.domain_axes(input)?;
    let mut compiler = DensityCompiler::new_seeded(graph.clone(), world_seed, legacy_random);
    let input = compiler.get_sampler(input)?;
    graph.bind(
        Arc::new(ClampDensity {
            input,
            resolution,
            contexts: ChildContexts::default(),
        }),
        Interval::INFINITE,
        axes,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::density::graph::GraphNode;
    use crate::density::{CachingDensitySampler, ConstantSampler, WorldField, WorldFields};

    fn field_context(value: f32) -> SamplerContext {
        SamplerContext::cached().with_world_fields(
            WorldFields::default()
                .with_field(WorldField::Beardifier, Arc::new(ConstantSampler(value))),
        )
    }
    fn clamped_field() -> Arc<dyn DensitySampler> {
        let mut graph = DensityGraph::new();
        let field = graph
            .insert(GraphNode::WorldField(WorldField::Beardifier))
            .unwrap();
        let cached = graph.insert(GraphNode::Cache(field)).unwrap();
        let bound = bind_clamp(&mut graph, cached, 16, 123, false).unwrap();
        assert_eq!(graph.domain_axes(bound).unwrap(), 7);
        assert_eq!(graph.range(bound).unwrap(), Interval::INFINITE);
        DensityCompiler::new(graph).get_sampler(bound).unwrap()
    }
    #[test]
    fn terraforged_child_cache_namespace_and_world_bindings_are_isolated() {
        let sampler = clamped_field();
        let mut parent = field_context(0.25);
        let other_graph = CachingDensitySampler::new(0, Arc::new(ConstantSampler(0.75))).unwrap();
        assert_eq!(other_graph.sample_value(&mut parent, [0; 3]).unwrap(), 0.75);
        assert_eq!(sampler.sample_value(&mut parent, [0; 3]).unwrap(), 0.3125);
        // The child's cache zero cannot overwrite the caller's cache zero.
        assert_eq!(other_graph.sample_value(&mut parent, [0; 3]).unwrap(), 0.75);
        parent = parent.with_world_fields(
            WorldFields::default()
                .with_field(WorldField::Beardifier, Arc::new(ConstantSampler(0.5))),
        );
        assert_eq!(sampler.sample_value(&mut parent, [0; 3]).unwrap(), 0.5625);
        let volume = DensityVolume::new([2, 3, 2], [-1, -64, -1], [1, 4, 1]).unwrap();
        let mut output = vec![-999.0; volume.size() + 1];
        sampler
            .sample_volume(&mut parent, &mut output, volume)
            .unwrap();
        assert!(output[..volume.size()].iter().all(|v| *v == 0.5625));
        assert_eq!(output[volume.size()], -999.0);
        let mut short = vec![-999.0; volume.size() - 1];
        assert!(
            sampler
                .sample_volume(&mut parent, &mut short, volume)
                .is_err()
        );
        assert!(short.iter().all(|v| *v == -999.0));
        std::thread::scope(|scope| {
            for n in 1..=4 {
                let sampler = &sampler;
                scope.spawn(move || {
                    let mut ctx = field_context(n as f32 / 16.0);
                    for _ in 0..32 {
                        assert_eq!(
                            sampler.sample_value(&mut ctx, [0; 3]).unwrap(),
                            (n + 1) as f32 / 16.0
                        );
                    }
                });
            }
        });
    }
    #[test]
    fn terraforged_child_contexts_release_world_bindings_after_errors() {
        let pool = ChildContexts::default();
        let parent = SamplerContext::sparse().with_planes();
        pool.with(&parent, |child| {
            assert!(child.caches.is_some());
            assert!(child.sparse);
            assert!(child.planes.as_ref().unwrap().is_empty());
            Ok(())
        })
        .unwrap();
        pool.with(&SamplerContext::default(), |child| {
            assert!(child.caches.is_none());
            assert!(!child.sparse);
            assert!(child.planes.is_none());
            Ok(())
        })
        .unwrap();
        let leaf = Arc::new(ConstantSampler(1.0));
        let weak = Arc::downgrade(&leaf);
        let parent = SamplerContext::default()
            .with_world_fields(WorldFields::default().with_field(WorldField::Beardifier, leaf));
        let result: Result<(), SampleError> = pool.with(&parent, |_| {
            Err(SampleError::External("controlled failure"))
        });
        assert_eq!(
            result.unwrap_err(),
            SampleError::External("controlled failure")
        );
        drop(parent);
        assert!(weak.upgrade().is_none());
        assert_eq!(pool.0.lock().unwrap().len(), 1);
        assert_eq!(
            pool.with(&SamplerContext::default(), |_| Ok(123)).unwrap(),
            123
        );
    }
}
