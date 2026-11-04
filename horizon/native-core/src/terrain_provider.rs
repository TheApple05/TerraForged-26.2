//! MIT ReTerraForged TerrainProvider: construction, composite registration and shuffle order.
use crate::{
    graph::{GraphError, Noise as N},
    java_random::JavaRandom,
    layout::Levels,
    noises as n,
    perlin::Interpolation,
    populator::{Populator as P, TerrainPopulator as T},
    preset::TerrainSettings,
    region::RegionConfig,
    seed::Seed,
    terrain::TerrainRegistry,
    terrain_factories as f,
    volcano::Volcano,
};
type R<T> = Result<T, GraphError>;
pub fn generate(
    seed: &Seed,
    settings: &TerrainSettings,
    region: &RegionConfig,
    levels: &Levels,
    ground: N,
    registry: &mut TerrainRegistry,
) -> R<Vec<P>> {
    let vertical = settings.general.global_vertical_scale;
    let fancy = settings.general.fancy_mountains;
    let mut terrain_seed = seed.offset(settings.general.terrain_seed_offset);
    let mut mixable = vec![
        f::make_steppe(&mut terrain_seed, ground.clone(), &settings.steppe)?,
        f::make_plains(
            &mut terrain_seed,
            ground.clone(),
            &settings.plains,
            vertical,
        )?,
        f::make_dales(&mut terrain_seed, ground.clone(), &settings.dales)?,
        f::make_hills1(&mut terrain_seed, ground.clone(), &settings.hills, vertical)?,
        f::make_hills2(&mut terrain_seed, ground.clone(), &settings.hills, vertical)?,
        f::make_torridonian(&mut terrain_seed, ground.clone(), &settings.torridonian)?,
        f::make_plateau(
            &mut terrain_seed,
            ground.clone(),
            &settings.plateau,
            vertical,
        )?,
        f::make_badlands(&mut terrain_seed, ground.clone(), &settings.badlands)?,
    ];
    mixable.retain(|p| p.weight > 0.0);
    let unmixable = vec![
        P::terrain(f::make_badlands(
            &mut terrain_seed,
            ground.clone(),
            &settings.badlands,
        )?)?,
        P::terrain(f::make_mountains1(
            &mut terrain_seed,
            ground.clone(),
            &settings.mountains,
            vertical,
            fancy,
        )?)?,
        P::terrain(f::make_mountains2(
            &mut terrain_seed,
            ground.clone(),
            &settings.mountains,
            vertical,
            fancy,
        )?)?,
        P::terrain(f::make_mountains3(
            &mut terrain_seed,
            ground,
            &settings.mountains,
            vertical,
            fancy,
        )?)?,
        P::volcano(Volcano::new(
            &mut terrain_seed,
            region,
            levels,
            settings.volcano.weight,
        )?)?,
    ];
    let mut mixed: Vec<P> = mixable.iter().cloned().map(P::terrain).collect::<R<_>>()?;
    for (i, a) in mixable.iter().enumerate() {
        for b in &mixable[i + 1..] {
            mixed.push(P::terrain(combine(
                a,
                b,
                &mut terrain_seed,
                levels,
                region.scale / 2,
                registry,
            )?)?);
        }
    }
    mixed.extend(unmixable);
    JavaRandom::new(terrain_seed.next_seed() as i64)
        .shuffle(&mut mixed)
        .map_err(|_| GraphError::InvalidSourceParameters("terrain provider shuffle size"))?;
    Ok(mixed)
}
fn combine(
    a: &T,
    b: &T,
    seed: &mut Seed,
    levels: &Levels,
    scale: i32,
    registry: &mut TerrainRegistry,
) -> R<T> {
    let terrain = registry
        .composite(a.terrain, b.terrain)
        .map_err(|_| GraphError::InvalidSourceParameters("owned composite terrain registry"))?;
    let selector = n::perlin(seed.next_seed(), scale, 1)?;
    let selector = n::warp_perlin(selector, seed.next_seed(), scale / 2, 2, scale as f32 / 2.0)?;
    let height = N::blend(
        selector.clone(),
        a.height.clone(),
        b.height.clone(),
        0.5,
        0.25,
        Interpolation::Linear,
    )?;
    let height = N::maximum(height, N::constant(0.0)?)?;
    let erosion = N::blend(
        selector.clone(),
        a.erosion.clone(),
        b.erosion.clone(),
        0.5,
        0.25,
        Interpolation::Linear,
    )?;
    let weirdness = N::threshold(
        selector,
        a.weirdness.clone(),
        b.weirdness.clone(),
        N::constant(0.5)?,
    )?;
    let weight = (a.weight + b.weight) / 2.0;
    Ok(T {
        terrain,
        base: N::constant(levels.ground)?,
        height,
        erosion,
        weirdness,
        base_scale: 1.0,
        height_scale: 1.0,
        weight,
    })
}
