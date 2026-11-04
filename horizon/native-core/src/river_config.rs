// MIT ReTerraForged river/lake/wetland configurations and Variance.
// river_config.rs

use crate::java_random::JavaRandom;
use crate::layout::Levels;
use crate::noise_util::round_legacy;
use crate::preset::{LakeSettings, WetlandSettings};

#[derive(Clone, Copy, Debug)]
pub struct Range {
    pub min: f32,
    pub max: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct Variance {
    pub min: f32,
    pub range: f32,
}

impl Variance {
    pub fn new(base: f32, range: f32) -> Self {
        Self { min: base, range }
    }

    pub fn of(min: f32, range: f32) -> Self {
        Self { min, range }
    }

    pub fn min(min: f32) -> Self {
        Self {
            min,
            range: 1.0f32 - min,
        }
    }

    pub fn range(range: f32) -> Self {
        Self {
            min: 1.0f32 - range,
            range,
        }
    }

    pub fn apply(&self, value: f32) -> f32 {
        self.min + value * self.range
    }

    pub fn apply_scaled(&self, value: f32, scaler: f32) -> f32 {
        self.apply(value) * scaler
    }

    pub fn next(&self, random: &mut JavaRandom) -> f32 {
        self.apply(random.next_f32())
    }

    pub fn next_scaled(&self, random: &mut JavaRandom, scalar: f32) -> f32 {
        self.apply_scaled(random.next_f32(), scalar)
    }
}

#[derive(Clone, Debug)]
pub struct RiverConfig {
    pub order: i32,
    pub main: bool,
    pub bed_width: i32,
    pub bank_width: i32,
    pub bed_height: f32,
    pub min_bank_height: f32,
    pub max_bank_height: f32,
    pub length: i32,
    pub length2: i32,
    pub fade: f32,
}

impl RiverConfig {
    fn from_builder(builder: &RiverConfigBuilder) -> Self {
        Self {
            main: builder.main,
            order: builder.order,
            bed_width: builder.bed_width,
            bank_width: builder.bank_width,
            bed_height: builder.levels.water(builder.bed_depth.wrapping_neg()),
            min_bank_height: builder.levels.water(builder.min_bank_height),
            max_bank_height: builder.levels.water(builder.max_bank_height),
            length: builder.length,
            length2: builder.length.wrapping_mul(builder.length),
            fade: builder.fade,
        }
    }

    pub fn new(
        main: bool,
        order: i32,
        bed_width: i32,
        bank_width: i32,
        bed_height: f32,
        min_bank_height: f32,
        max_bank_height: f32,
        length: i32,
        length2: i32,
        fade: f32,
    ) -> Self {
        Self {
            main,
            order,
            bed_width,
            bank_width,
            bed_height,
            min_bank_height,
            max_bank_height,
            length,
            length2,
            fade,
        }
    }

    pub fn create_fork_width(&self, connect_width: f32, levels: Levels) -> RiverConfig {
        if (self.bank_width as f32) < connect_width {
            return self.clone();
        }
        let scale = self.bank_width as f32 / connect_width;
        self.create_fork_dimensions(
            levels.scale_f32(self.bed_height),
            round_legacy(self.bed_width as f32 / scale),
            round_legacy(self.bank_width as f32 / scale),
            levels,
        )
    }

    pub fn create_fork_dimensions(
        &self,
        bed_height: i32,
        bed_width: i32,
        bank_width: i32,
        levels: Levels,
    ) -> RiverConfig {
        let min_bank_height = i32::max(
            levels.ground_level,
            levels.scale_f32(self.min_bank_height).wrapping_sub(1),
        );
        let max_bank_height = i32::max(
            min_bank_height,
            levels.scale_f32(self.max_bank_height).wrapping_sub(1),
        );
        RiverConfig::new(
            false,
            self.order.wrapping_add(1),
            bed_width,
            bank_width,
            levels.scale_i32(bed_height),
            levels.scale_i32(min_bank_height),
            levels.scale_i32(max_bank_height),
            self.length,
            self.length2,
            self.fade,
        )
    }

    pub fn builder(levels: Levels) -> RiverConfigBuilder {
        RiverConfigBuilder::new(levels)
    }
}

#[derive(Clone, Debug)]
pub struct RiverConfigBuilder {
    main: bool,
    order: i32,
    bed_width: i32,
    bank_width: i32,
    bed_depth: i32,
    max_bank_height: i32,
    min_bank_height: i32,
    length: i32,
    fade: f32,
    levels: Levels,
}

impl RiverConfigBuilder {
    pub fn new(levels: Levels) -> Self {
        Self {
            main: false,
            order: 0,
            bed_width: 4,
            bank_width: 15,
            bed_depth: 5,
            max_bank_height: 1,
            min_bank_height: 1,
            length: 1000,
            fade: 0.2f32,
            levels,
        }
    }

    pub fn order(&mut self, order: i32) -> &mut Self {
        self.order = order;
        self
    }

    pub fn main(&mut self, value: bool) -> &mut Self {
        self.main = value;
        self
    }

    pub fn bed_width(&mut self, value: i32) -> &mut Self {
        self.bed_width = value;
        self
    }

    pub fn bank_width(&mut self, value: i32) -> &mut Self {
        self.bank_width = value;
        self
    }

    pub fn bed_depth(&mut self, depth: i32) -> &mut Self {
        self.bed_depth = depth;
        self
    }

    pub fn bank_height(&mut self, min: i32, max: i32) -> &mut Self {
        self.min_bank_height = i32::min(min, max);
        self.max_bank_height = i32::max(min, max);
        self
    }

    pub fn length(&mut self, value: i32) -> &mut Self {
        self.length = value;
        self
    }

    pub fn fade(&mut self, value: f32) -> &mut Self {
        self.fade = value;
        self
    }

    pub fn build(&self) -> RiverConfig {
        RiverConfig::from_builder(self)
    }
}

#[derive(Clone, Debug)]
pub struct LakeConfig {
    pub depth: f32,
    pub chance: f32,
    pub size_min: f32,
    pub size_max: f32,
    pub size_range: f32,
    pub bank_min: f32,
    pub bank_max: f32,
    pub distance_min: f32,
    pub distance_max: f32,
}

impl LakeConfig {
    fn from_builder(builder: &LakeConfigBuilder) -> Self {
        Self {
            depth: builder.depth,
            chance: builder.chance,
            size_min: builder.size_min,
            size_max: builder.size_max,
            size_range: builder.size_max - builder.size_min,
            bank_min: builder.bank_min,
            bank_max: builder.bank_max,
            distance_min: builder.distance_min,
            distance_max: builder.distance_max,
        }
    }

    pub fn from_settings(settings: &LakeSettings, levels: Levels) -> Self {
        let mut builder = LakeConfigBuilder::new();
        builder.chance = settings.chance;
        builder.size_min = settings.size_min as f32;
        builder.size_max = settings.size_max as f32;
        builder.depth = levels.water(settings.depth.wrapping_neg());
        builder.distance_min = settings.min_start_distance;
        builder.distance_max = settings.max_start_distance;
        builder.bank_min = levels.water(settings.min_bank_height);
        builder.bank_max = levels.water(settings.max_bank_height);
        LakeConfig::from_builder(&builder)
    }
}

#[derive(Clone, Debug)]
pub struct LakeConfigBuilder {
    pub chance: f32,
    pub depth: f32,
    pub size_min: f32,
    pub size_max: f32,
    pub bank_min: f32,
    pub bank_max: f32,
    pub distance_min: f32,
    pub distance_max: f32,
}

impl LakeConfigBuilder {
    pub fn new() -> Self {
        Self {
            chance: 0.0f32,
            depth: 10.0f32,
            size_min: 30.0f32,
            size_max: 100.0f32,
            bank_min: 1.0f32,
            bank_max: 8.0f32,
            distance_min: 0.025f32,
            distance_max: 0.05f32,
        }
    }
}

#[derive(Clone, Debug)]
pub struct WetlandConfig {
    pub skip_size: i32,
    pub length: Variance,
    pub width: Variance,
}

impl WetlandConfig {
    pub fn from_settings(settings: &WetlandSettings) -> Self {
        Self {
            skip_size: i32::max(1, round_legacy((1.0f32 - settings.chance) * 10.0f32)),
            length: Variance::of(settings.size_min as f32, settings.size_max as f32),
            width: Variance::of(50.0f32, 150.0f32),
        }
    }
}
