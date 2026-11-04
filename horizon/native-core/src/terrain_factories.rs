//! MIT ReTerraForged Populators.java: original terrain graph construction and seed order.
use crate::{
    domain::Domain,
    erosion::{BlendMode, Erosion},
    graph::{CurveFunction, GraphError, Noise as N},
    layout::Levels,
    noise_util::round_java as round,
    noises as n,
    perlin::Interpolation,
    populator::{Populator, TerrainPopulator as T},
    preset::TerrainSettingsEntry as Settings,
    seed::Seed,
    worley::{DistanceFunction, EdgeFunction},
};
type R<T> = Result<T, GraphError>;
trait Arg {
    fn noise(self) -> R<N>;
}
impl Arg for f32 {
    fn noise(self) -> R<N> {
        N::constant(self)
    }
}
impl Arg for N {
    fn noise(self) -> R<N> {
        Ok(self)
    }
}
impl Arg for &N {
    fn noise(self) -> R<N> {
        Ok(self.clone())
    }
}
fn c(value: f32) -> R<N> {
    N::constant(value)
}
fn mul(input: N, value: impl Arg) -> R<N> {
    N::multiply(input, value.noise()?)
}
fn add(input: N, value: impl Arg) -> R<N> {
    N::add(input, value.noise()?)
}
fn alpha(input: N, value: f32) -> R<N> {
    N::alpha(input, c(value)?)
}
fn clamp(input: N, min: f32, max: f32) -> R<N> {
    N::clamp(input, c(min)?, c(max)?)
}
fn map(input: N, min: f32, max: f32) -> R<N> {
    n::map_range(input, min, max)
}
fn warp(input: N, x: N, z: N, strength: f32) -> R<N> {
    N::warp(input, Domain::warp(x, z, c(strength)?)?)
}
fn blend(selector: N, a: N, b: N, mid: f32, range: f32) -> R<N> {
    N::blend(selector, a, b, mid, range, Interpolation::Linear)
}
fn threshold(selector: N, a: impl Arg, b: impl Arg, mid: f32) -> R<N> {
    N::threshold(selector, a.noise()?, b.noise()?, c(mid)?)
}
fn terrace(
    input: N,
    lower_curve: f32,
    upper_curve: f32,
    lower_height: f32,
    range: f32,
    steps: i32,
) -> R<N> {
    N::terrace(
        input,
        c(lower_curve)?,
        c(upper_curve)?,
        c(lower_height)?,
        range,
        steps,
    )
}
fn steps(input: N, count: i32, min: f32, max: f32, interpolation: Interpolation) -> R<N> {
    N::steps(
        input,
        c(count as f32)?,
        c(min)?,
        c(max)?,
        CurveFunction::Interpolation(interpolation),
    )
}
fn perlin_lac(seed: i32, scale: i32, octaves: i32, lacunarity: f32) -> R<N> {
    n::perlin_with(seed, scale, octaves, lacunarity, 0.5)
}
fn ridge_lac(seed: i32, scale: i32, octaves: i32, lacunarity: f32) -> R<N> {
    n::perlin_ridge_with(seed, scale, octaves, lacunarity, 0.975)
}
pub fn erosion_mid(level: usize) -> f32 {
    let (min, max): (f32, f32) = [
        (-1.0, -0.78),
        (-0.78, -0.375),
        (-0.375, -0.2225),
        (-0.2225, 0.05),
        (0.05, 0.45),
        (0.45, 0.55),
        (0.55, 1.0),
    ][level];
    (min + max) / 2.0
}
pub fn weirdness_mid() -> f32 {
    (-0.26666668f32 + -0.05f32) / 2.0
}
fn default_erosion() -> R<N> {
    c(erosion_mid(4))
}
fn default_weirdness() -> R<N> {
    c(weirdness_mid())
}
fn make(terrain: usize, base: N, height: N, erosion: N, weirdness: N, s: &Settings) -> T {
    T {
        terrain,
        base,
        height,
        erosion,
        weirdness,
        base_scale: s.base_scale,
        height_scale: s.vertical_scale,
        weight: s.weight,
    }
}

pub fn make_deep_ocean(mut seed: i32, sea_level: f32) -> R<Populator> {
    seed = seed.wrapping_add(1);
    let hills = n::perlin(seed, 150, 3)?;
    let hills = mul(hills, sea_level * 0.7)?;
    seed = seed.wrapping_add(1);
    let hill_bias = mul(n::perlin(seed, 200, 1)?, sea_level * 0.2)?;
    let hills = add(hills, hill_bias)?;
    seed = seed.wrapping_add(1);
    let canyons = n::perlin(seed, 150, 4)?;
    let canyons = N::power_curve(canyons, 0.2)?;
    let canyons = mul(N::invert(canyons)?, sea_level * 0.7)?;
    seed = seed.wrapping_add(1);
    let canyon_bias = mul(n::perlin(seed, 170, 1)?, sea_level * 0.15)?;
    let canyons = add(canyons, canyon_bias)?;
    seed = seed.wrapping_add(1);
    let selector = n::perlin(seed, 500, 1)?;
    let height = blend(selector, hills, canyons, 0.6, 0.65)?;
    seed = seed.wrapping_add(1);
    let height = n::warp_perlin(height, seed, 50, 2, 50.0)?;
    Populator::ocean(1, height)
}
pub fn make_shallow_ocean(levels: &Levels) -> R<Populator> {
    Populator::ocean(2, c(levels.water(-7))?)
}
pub fn make_coast(levels: &Levels) -> R<Populator> {
    Populator::ocean(3, c(levels.water)?)
}

pub fn make_torridonian(seed: &mut Seed, ground: N, s: &Settings) -> R<T> {
    let plains = n::perlin(seed.next_seed(), 100, 3)?;
    let plains = n::warp_perlin(plains, seed.next_seed(), 300, 1, 150.0)?;
    let plains = n::warp_perlin(plains, seed.next_seed(), 20, 1, 40.0)?;
    let plains = mul(plains, 0.15)?;
    let hills = n::perlin(seed.next_seed(), 150, 4)?;
    let hills = n::warp_perlin(hills, seed.next_seed(), 300, 1, 200.0)?;
    let hills = n::warp_perlin(hills, seed.next_seed(), 20, 2, 20.0)?;
    let hills = N::boost(hills, 1)?;
    let selector = n::perlin(seed.next_seed(), 200, 3)?;
    let modulation = mul(n::perlin(seed.next_seed(), 120, 1)?, 0.25)?;
    let mask = add(mul(n::perlin(seed.next_seed(), 200, 1)?, 0.5)?, 0.5)?;
    let blended = blend(selector, plains, hills, 0.6, 0.6)?;
    let blended = N::advanced_terrace(blended, modulation, mask, c(0.5)?, 0.0, 0.3, 6, 1)?;
    let height = mul(N::boost(blended.clone(), 1)?, 0.5)?;
    let weirdness = N::minimum(mul(blended, -1.0)?, c(-0.05f32 - 0.01f32)?)?;
    Ok(make(11, ground, height, c(erosion_mid(5))?, weirdness, s))
}
fn mountains(
    terrain: usize,
    seed: &mut Seed,
    ground: N,
    s: &Settings,
    vertical_scale: f32,
    fancy: bool,
) -> R<T> {
    let scale_h = round(410.0 * s.horizontal_scale);
    let height = n::perlin_ridge_with(seed.next_seed(), scale_h, 4, 2.35, 1.15)?;
    let scaler = alpha(n::perlin(seed.next_seed(), 24, 4)?, 0.075)?;
    let height = mul(height, scaler)?;
    let mut height = n::warp_perlin(height, seed.next_seed(), 350, 1, 150.0)?;
    if fancy {
        height = make_fancy(seed, height)?;
    }
    let height = N::cache(height)?;
    let scaled = mul(height.clone(), 0.7 * vertical_scale)?;
    let weirdness = N::minimum(mul(height, -1.0)?, c(-0.08)?)?;
    Ok(make(
        terrain,
        ground,
        scaled,
        c(erosion_mid(1))?,
        weirdness,
        s,
    ))
}
pub fn make_mountains1(
    seed: &mut Seed,
    ground: N,
    s: &Settings,
    vertical_scale: f32,
    fancy: bool,
) -> R<T> {
    mountains(12, seed, ground, s, vertical_scale, fancy)
}
pub fn make_mountain_chain(
    seed: &mut Seed,
    ground: N,
    s: &Settings,
    vertical_scale: f32,
    fancy: bool,
) -> R<T> {
    mountains(15, seed, ground, s, vertical_scale, fancy)
}
pub fn make_mountains2(
    seed: &mut Seed,
    ground: N,
    s: &Settings,
    vertical_scale: f32,
    fancy: bool,
) -> R<T> {
    let cell = n::worley_edge_with(
        seed.next_seed(),
        360,
        EdgeFunction::Distance2,
        DistanceFunction::Euclidean,
    )?;
    let cell = clamp(mul(cell, 1.2)?, 0.0, 1.0)?;
    let cell = n::warp_perlin(cell, seed.next_seed(), 200, 2, 100.0)?;
    let blur = alpha(n::perlin(seed.next_seed(), 10, 1)?, 0.025)?;
    let surface = alpha(n::perlin_ridge(seed.next_seed(), 125, 4)?, 0.37)?;
    let height = clamp(cell, 0.0, 1.0)?;
    let mut height = N::power(mul(mul(height, blur)?, surface)?, 1.1)?;
    if fancy {
        height = make_fancy(seed, height)?;
    }
    let height = N::cache(height)?;
    let weirdness = N::minimum(mul(height.clone(), -1.0)?, c(-0.08)?)?;
    Ok(make(
        13,
        ground,
        mul(height, 0.645 * vertical_scale)?,
        c(erosion_mid(2))?,
        weirdness,
        s,
    ))
}
pub fn make_mountains3(
    seed: &mut Seed,
    ground: N,
    s: &Settings,
    vertical_scale: f32,
    fancy: bool,
) -> R<T> {
    let cell = n::worley_edge_with(
        seed.next_seed(),
        400,
        EdgeFunction::Distance2,
        DistanceFunction::Euclidean,
    )?;
    let cell = clamp(mul(cell, 1.2)?, 0.0, 1.0)?;
    let cell = n::warp_perlin(cell, seed.next_seed(), 200, 2, 100.0)?;
    let blur = alpha(n::perlin(seed.next_seed(), 10, 1)?, 0.025)?;
    let surface = alpha(n::perlin_ridge(seed.next_seed(), 125, 4)?, 0.37)?;
    let mountains = N::power(mul(mul(clamp(cell, 0.0, 1.0)?, blur)?, surface)?, 1.1)?;
    let modulation = mul(n::perlin(seed.next_seed(), 50, 1)?, 0.5)?;
    let mask = map(
        clamp(n::perlin(seed.next_seed(), 100, 1)?, 0.5, 0.95)?,
        0.0,
        1.0,
    )?;
    let mut height = N::advanced_terrace(mountains, modulation, mask, c(0.45)?, 0.2, 0.45, 24, 1)?;
    if fancy {
        height = make_fancy(seed, height)?;
    }
    let height = N::cache(height)?;
    let weirdness = N::minimum(mul(height.clone(), -1.0)?, c(-0.08)?)?;
    Ok(make(
        14,
        ground,
        mul(height, 0.645 * vertical_scale)?,
        c(erosion_mid(1))?,
        weirdness,
        s,
    ))
}
pub fn make_fancy(seed: &mut Seed, input: N) -> R<N> {
    let domain = Domain::direction(n::perlin(seed.next_seed(), 10, 1)?, c(2.0)?)?;
    let erosion = Erosion::new(
        seed.next_seed(),
        2,
        0.65,
        128.0,
        0.15,
        3.1,
        0.8,
        BlendMode::Constant,
    )
    .map_err(|_| GraphError::InvalidSourceParameters("fancy mountain erosion octaves"))?;
    N::warp(N::erosion(input, erosion)?, domain)
}
pub fn make_border(
    seed: &mut Seed,
    ground: N,
    plains: &Settings,
    steppe: &Settings,
    vertical_scale: f32,
) -> R<T> {
    plains_with_scaling(seed, ground, plains, steppe, vertical_scale)
}

pub fn make_steppe(seed: &mut Seed, ground: N, s: &Settings) -> R<T> {
    let scale_h = round(250.0 * s.horizontal_scale);
    let erosion = alpha(
        perlin_lac(seed.next_seed(), scale_h.wrapping_mul(2), 3, 3.75)?,
        0.45,
    )?;
    let wx = perlin_lac(seed.next_seed(), scale_h / 4, 3, 3.0)?;
    let wz = perlin_lac(seed.next_seed(), scale_h / 4, 3, 3.0)?;
    let height = mul(n::perlin(seed.next_seed(), scale_h, 1)?, erosion)?;
    let height = warp(height, wx, wz, scale_h as f32 / 4.0)?;
    let height = n::warp_perlin(height, seed.next_seed(), 256, 1, 200.0)?;
    let height = add(mul(height, 0.08)?, -0.02)?;
    Ok(make(
        8,
        ground,
        height,
        default_erosion()?,
        default_weirdness()?,
        s,
    ))
}
fn plains_with_scaling(
    seed: &mut Seed,
    ground: N,
    noise_s: &Settings,
    scale_s: &Settings,
    vertical_scale: f32,
) -> R<T> {
    let scale_h = round(250.0 * noise_s.horizontal_scale);
    let erosion = alpha(
        perlin_lac(seed.next_seed(), scale_h.wrapping_mul(2), 3, 3.75)?,
        0.45,
    )?;
    let wx = perlin_lac(seed.next_seed(), scale_h / 4, 3, 3.5)?;
    let wz = perlin_lac(seed.next_seed(), scale_h / 4, 3, 3.5)?;
    let height = mul(n::perlin(seed.next_seed(), scale_h, 1)?, erosion)?;
    let height = warp(height, wx, wz, scale_h as f32 / 4.0)?;
    let height = n::warp_perlin(height, seed.next_seed(), 256, 1, 256.0)?;
    let height = add(mul(height, 0.15 * vertical_scale)?, -0.02)?;
    Ok(make(
        8,
        ground,
        height,
        default_erosion()?,
        default_weirdness()?,
        scale_s,
    ))
}
pub fn make_plains(seed: &mut Seed, ground: N, s: &Settings, vertical_scale: f32) -> R<T> {
    plains_with_scaling(seed, ground, s, s, vertical_scale)
}
pub fn make_hills1(seed: &mut Seed, ground: N, s: &Settings, vertical_scale: f32) -> R<T> {
    let height = n::perlin(seed.next_seed(), 200, 3)?;
    let scaler = alpha(n::billow(seed.next_seed(), 400, 3)?, 0.5)?;
    let height = mul(height, scaler)?;
    let height = n::warp_perlin(height, seed.next_seed(), 30, 3, 20.0)?;
    let height = n::warp_perlin(height, seed.next_seed(), 400, 3, 200.0)?;
    Ok(make(
        11,
        ground,
        mul(height, 0.6 * vertical_scale)?,
        default_erosion()?,
        default_weirdness()?,
        s,
    ))
}
pub fn make_hills2(seed: &mut Seed, ground: N, s: &Settings, vertical_scale: f32) -> R<T> {
    let height = n::cubic(seed.next_seed(), 128, 2)?;
    let scaler = alpha(n::perlin(seed.next_seed(), 32, 4)?, 0.075)?;
    let height = mul(height, scaler)?;
    let height = n::warp_perlin(height, seed.next_seed(), 30, 3, 20.0)?;
    let height = n::warp_perlin(height, seed.next_seed(), 400, 3, 200.0)?;
    let scaler = alpha(n::perlin_ridge(seed.next_seed(), 512, 2)?, 0.8)?;
    let height = mul(height, scaler)?;
    Ok(make(
        11,
        ground,
        mul(height, 0.55 * vertical_scale)?,
        default_erosion()?,
        default_weirdness()?,
        s,
    ))
}
pub fn make_dales(seed: &mut Seed, ground: N, s: &Settings) -> R<T> {
    let hills1 = n::billow_with(seed.next_seed(), 300, 4, 4.0, 0.8)?;
    let hills1 = mul(N::power_curve(hills1, 0.5)?, 0.75)?;
    let hills2 = N::power(n::billow_with(seed.next_seed(), 350, 3, 4.0, 0.8)?, 1.25)?;
    let selector = map(
        clamp(n::perlin(seed.next_seed(), 400, 1)?, 0.3, 0.6)?,
        0.0,
        1.0,
    )?;
    let warp_seed = seed.next_seed();
    let height = N::power(blend(selector.clone(), hills1, hills2, 0.4, 0.75)?, 1.125)?;
    let height = n::warp_perlin(height, warp_seed, 300, 1, 100.0)?;
    let erosion = threshold(selector, erosion_mid(2), erosion_mid(4), 0.5)?;
    let weirdness = N::minimum(mul(height.clone(), -1.0)?, c(-0.06)?)?;
    Ok(make(11, ground, mul(height, 0.4)?, erosion, weirdness, s))
}
pub fn make_plateau(seed: &mut Seed, ground: N, s: &Settings, vertical_scale: f32) -> R<T> {
    let mut valley = n::perlin_ridge(seed.next_seed(), 500, 1)?;
    valley = N::invert(valley)?;
    valley = n::warp_perlin(valley, seed.next_seed(), 100, 1, 150.0f32)?;
    valley = n::warp_perlin(valley, seed.next_seed(), 20, 1, 15.0f32)?;

    let mut top = ridge_lac(seed.next_seed(), 150, 3, 2.45f32)?;
    top = n::warp_perlin(top, seed.next_seed(), 300, 1, 150.0f32)?;
    top = n::warp_perlin(top, seed.next_seed(), 40, 2, 20.0f32)?;
    top = mul(top, 0.15f32)?;

    let mut valley_scaler = clamp(valley.clone(), 0.02f32, 0.1f32)?;
    valley_scaler = map(valley_scaler, 0.0f32, 1.0f32)?;

    top = mul(top, &valley_scaler)?;

    let mut surface = n::perlin(seed.next_seed(), 20, 3)?;
    surface = mul(surface, 0.05f32)?;
    surface = n::warp_perlin(surface, seed.next_seed(), 40, 2, 20.0f32)?;

    let mut cubic = n::cubic(seed.next_seed(), 500, 1)?;
    cubic = mul(cubic, 0.6f32)?;
    cubic = add(cubic, 0.3f32)?;

    let mut valley_base = mul(valley.clone(), &cubic)?;
    valley_base = add(valley_base, &top)?;

    let mut height = terrace(valley_base.clone(), 0.9f32, 0.15f32, 0.35f32, 0.4f32, 4)?;
    height = add(height, &surface)?;
    height = mul(height, 0.475f32 * vertical_scale)?;

    let mut weirdness = clamp(valley_base.clone(), 0.0f32, 0.415f32)?;
    weirdness = map(weirdness, 0.0f32, 1.0f32)?;
    weirdness = map(weirdness, weirdness_mid(), -0.42f32)?;
    Ok(make(10, ground, height, c(erosion_mid(3))?, weirdness, s))
}

pub fn make_badlands(seed: &mut Seed, ground: N, s: &Settings) -> R<T> {
    let mut mask = n::perlin(seed.next_seed(), 270, 3)?;
    mask = clamp(mask, 0.35f32, 0.65f32)?;
    mask = map(mask, 0.0f32, 1.0f32)?;

    let mut hills = n::perlin_ridge(seed.next_seed(), 275, 4)?;
    hills = n::warp_perlin(hills, seed.next_seed(), 400, 2, 100.0f32)?;
    hills = n::warp_perlin(hills, seed.next_seed(), 18, 1, 20.0f32)?;
    hills = mul(hills, &mask)?;

    let modulation = 0.4f32;
    let alpha_value = 1.0f32 - modulation;

    let mut mod1 = n::warp_perlin(hills.clone(), seed.next_seed(), 100, 1, 50.0f32)?;
    mod1 = mul(mod1, modulation)?;

    let mut low_freq = steps(hills.clone(), 4, 0.6f32, 0.7f32, Interpolation::Linear)?;
    low_freq = mul(low_freq, alpha_value)?;
    low_freq = add(low_freq, &mod1)?;

    let mut high_freq = steps(hills.clone(), 10, 0.6f32, 0.7f32, Interpolation::Linear)?;
    high_freq = mul(high_freq, alpha_value)?;
    high_freq = add(high_freq, &mod1)?;

    let mut detail = add(low_freq.clone(), &high_freq)?;
    detail = alpha(detail, 0.5f32)?;

    let mut scaler = n::perlin(seed.next_seed(), 200, 3)?;
    scaler = mul(scaler, modulation)?;

    let mod2 = mul(hills.clone(), &scaler)?;

    let mut shape = steps(hills.clone(), 4, 0.65f32, 0.75f32, Interpolation::Curve3)?;
    shape = mul(shape, alpha_value)?;
    shape = add(shape, &mod2)?;
    shape = mul(shape, alpha_value)?;

    let mut height = mul(shape.clone(), &detail)?;
    height = mul(height, 0.55f32)?;
    height = add(height, 0.025f32)?;
    Ok(make(
        9,
        ground,
        height,
        default_erosion()?,
        default_weirdness()?,
        s,
    ))
}
