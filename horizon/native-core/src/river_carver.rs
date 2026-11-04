//! MIT ReTerraForged RiverCarver. Cell mutations retain the original order.
use crate::{
    cell::Cell,
    graph::{CurveFunction, GraphError, java_min},
    java_random::JavaRandom,
    layout::Levels,
    line::dist_sq,
    noise_util as u,
    river::{MAIN_VALLEY, River},
    river_config::{Range, RiverConfig},
    river_warp::RiverWarp,
    terrain::TerrainRegistry,
};
#[derive(Clone, Debug)]
pub struct CarverSettings {
    pub valley_size: f32,
    pub fade_in: f32,
    pub connecting: bool,
    pub valley_curve: CurveFunction,
}
impl Default for CarverSettings {
    fn default() -> Self {
        Self {
            valley_size: 275.0,
            fade_in: 0.7,
            connecting: false,
            valley_curve: CurveFunction::scurve(2.0, -0.5),
        }
    }
}
#[derive(Clone, Debug)]
pub struct RiverCarver {
    pub main: bool,
    pub river: River,
    pub warp: RiverWarp,
    pub config: RiverConfig,
    pub valley_curve: CurveFunction,
    pub connecting: bool,
    fade: f32,
    fade_inv: f32,
    bed_width: Range,
    banks_width: Range,
    valley_width: Range,
    bed_depth: Range,
    banks_depth: Range,
    water_line: f32,
}
impl RiverCarver {
    pub fn new(
        river: River,
        warp: RiverWarp,
        config: RiverConfig,
        settings: CarverSettings,
        levels: Levels,
    ) -> Self {
        let fade = settings.fade_in;
        let fade_inv = 1.0 / settings.fade_in;
        let bed_width = Range {
            min: 0.25,
            max: config.bed_width.wrapping_mul(config.bed_width) as f32,
        };
        let banks_width = Range {
            min: 1.5625,
            max: config.bank_width.wrapping_mul(config.bank_width) as f32,
        };
        let valley_width = Range {
            min: settings.valley_size * settings.valley_size,
            max: settings.valley_size * settings.valley_size,
        };
        let main = config.main;
        let bed_depth = Range {
            min: levels.water,
            max: config.bed_height,
        };
        let banks_depth = Range {
            min: config.min_bank_height,
            max: config.max_bank_height,
        };
        Self {
            main,
            river,
            warp,
            config,
            valley_curve: settings.valley_curve,
            connecting: settings.connecting,
            fade,
            fade_inv,
            bed_width,
            banks_width,
            valley_width,
            bed_depth,
            banks_depth,
            water_line: levels.water,
        }
    }
    pub fn compare_order(&self, other: &Self) -> std::cmp::Ordering {
        self.config.order.cmp(&other.config.order)
    }
    pub fn carve(
        &self,
        cell: &mut Cell,
        px: f32,
        pz: f32,
        pt: f32,
        x: f32,
        z: f32,
        t: f32,
        terrains: &TerrainRegistry,
    ) -> Result<(), GraphError> {
        let d2 = self.distance2(x, z, t);
        let pd2 = self.distance2(px, pz, pt);
        let mut valley_alpha = self.distance_alpha(pt, java_min(d2, pd2), self.valley_width);
        if valley_alpha == 0.0 {
            return Ok(());
        }
        let bank_height = self.scaled_size(t, self.banks_depth);
        valley_alpha = self.valley_curve.apply(valley_alpha);
        cell.river_mask = java_min(cell.river_mask, 1.0 - valley_alpha);
        cell.height = java_min(
            u::lerp_f32(cell.height, bank_height, valley_alpha),
            cell.height,
        );
        // Source's connecting/t>1 block is empty and has no carving effect.
        let mut mouth = u::map_clamped_range(cell.continent_edge, 0.0, 0.5, 0.5);
        mouth *= mouth;
        let bed_height = self.scaled_size(t, self.bed_depth);
        let banks_alpha = self.distance_alpha(t, d2 * mouth, self.banks_width);
        if banks_alpha == 0.0 {
            return Ok(());
        }
        if cell.height > bed_height {
            cell.height = java_min(
                u::lerp_f32(cell.height, bed_height, banks_alpha),
                cell.height,
            );
            self.tag(cell, bed_height, terrains)?;
        }
        let bed_alpha = self.distance_alpha(t, d2, self.bed_width);
        if bed_alpha != 0.0 && cell.height > bed_height {
            cell.height = u::lerp_f32(cell.height, bed_height, bed_alpha);
            self.tag(cell, bed_height, terrains)?;
        }
        Ok(())
    }
    pub fn create_fork_config(&self, t: f32, levels: Levels) -> RiverConfig {
        let bed_height = levels.scale_f32(self.scaled_size(t, self.bed_depth));
        let bed_width =
            u::round_java_f64((self.scaled_size(t, self.bed_width) as f64).sqrt() * 0.75) as i32;
        let bank_width =
            u::round_java_f64((self.scaled_size(t, self.banks_width) as f64).sqrt() * 0.75) as i32;
        let bed_width = 1.max(bed_width);
        let bank_width = bed_width.wrapping_add(1).max(bank_width);
        self.config
            .create_fork_dimensions(bed_height, bed_width, bank_width, levels)
    }
    fn distance2(&self, x: f32, z: f32, t: f32) -> f32 {
        if t <= 0.0 {
            return dist_sq(x, z, self.river.x1, self.river.z1);
        }
        if t >= 1.0 {
            return dist_sq(x, z, self.river.x2, self.river.z2);
        }
        dist_sq(
            x,
            z,
            self.river.x1 + t * self.river.dx,
            self.river.z1 + t * self.river.dz,
        )
    }
    fn distance_alpha(&self, t: f32, distance2: f32, range: Range) -> f32 {
        let size2 = self.scaled_size(t, range);
        if distance2 >= size2 {
            0.0
        } else {
            1.0 - distance2 / size2
        }
    }
    fn scaled_size(&self, t: f32, range: Range) -> f32 {
        if t < 0.0 {
            return range.min;
        }
        if t > 1.0 {
            return range.max;
        }
        if range.min == range.max {
            return range.min;
        }
        if t >= self.fade {
            return range.max;
        }
        u::lerp_f32(range.min, range.max, t * self.fade_inv)
    }
    fn tag(
        &self,
        cell: &mut Cell,
        bed_height: f32,
        terrains: &TerrainRegistry,
    ) -> Result<(), GraphError> {
        let terrain = terrains
            .get(cell.terrain)
            .ok_or(GraphError::InvalidSourceParameters(
                "owned river-carver terrain ID",
            ))?;
        if terrain.overrides_river() && (cell.height < bed_height || cell.height > self.water_line)
        {
            return Ok(());
        }
        cell.erosion_mask = true;
        if cell.height <= self.water_line {
            cell.terrain = 5;
        }
        Ok(())
    }
    pub fn valley_type(random: &mut JavaRandom) -> CurveFunction {
        let value = random
            .bounded_i32(100)
            .expect("fixed positive JavaRandom bound");
        if value < 5 {
            CurveFunction::scurve(0.4, 1.0)
        } else if value < 30 {
            CurveFunction::scurve(4.0, 5.0)
        } else if value < 50 {
            CurveFunction::scurve(3.0, 0.25)
        } else {
            CurveFunction::scurve(2.0, -0.5)
        }
    }
    pub fn create(
        x1: f32,
        z1: f32,
        x2: f32,
        z2: f32,
        config: RiverConfig,
        levels: Levels,
        random: &mut JavaRandom,
    ) -> Self {
        let river = River::new(x1, z1, x2, z2);
        let warp = RiverWarp::create_with_fade(0.35, random);
        let valley_size = 275.0 * MAIN_VALLEY.next(random);
        let valley_curve = Self::valley_type(random);
        let settings = CarverSettings {
            valley_size,
            fade_in: config.fade,
            connecting: false,
            valley_curve,
        };
        Self::new(river, warp, config, settings, levels)
    }
}
