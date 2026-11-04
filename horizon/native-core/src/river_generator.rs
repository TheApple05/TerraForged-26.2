//! MIT ReTerraForged SimpleRiverGenerator / used BaseRiverGenerator path.
//! Construction IDs replace mutable Java builder identity; finished maps own trees.
use crate::{
    bounds::{Bounds, BoundsBuilder},
    continent::AdvancedContinent,
    graph::{GraphError, java_max},
    java_random::JavaRandom,
    lake::Lake,
    layout::Levels,
    line, noise_util as u,
    preset::{KernelPreset, RiverProfile},
    river::{FORK_ANGLE, FORK_SPACING, FORK_VALLEY, MAIN_SPACING, MAIN_VALLEY, River},
    river_carver::{CarverSettings, RiverCarver},
    river_config::{LakeConfig, RiverConfig, Variance, WetlandConfig},
    river_domain::GenWarp,
    river_network::Network,
    river_warp::RiverWarp,
    rivermap::Rivermap,
    wetland::Wetland,
};
use std::sync::Arc;
type R<T> = Result<T, GraphError>;
const MAX_NODES: usize = 65536;
#[derive(Clone, Debug)]
pub struct RiverGenerator {
    continent: Arc<AdvancedContinent>,
    levels: Levels,
    continent_scale: i32,
    min_edge_value: f32,
    seed: i32,
    count: i32,
    main: RiverConfig,
    fork: RiverConfig,
    lake: LakeConfig,
    wetland: WetlandConfig,
}
#[derive(Debug)]
struct Builder {
    carver: RiverCarver,
    lakes: Vec<Lake>,
    wetlands: Vec<Wetland>,
    children: Vec<usize>,
}
#[derive(Default, Debug)]
struct Arena {
    nodes: Vec<Builder>,
}
impl Arena {
    fn add(&mut self, carver: RiverCarver) -> R<usize> {
        if self.nodes.len() >= MAX_NODES {
            return Err(GraphError::InvalidSourceParameters(
                "river construction node budget",
            ));
        }
        let id = self.nodes.len();
        self.nodes.push(Builder {
            carver,
            lakes: vec![],
            wetlands: vec![],
            children: vec![],
        });
        Ok(id)
    }
    fn overlaps(&self, id: usize, river: &River, parent: usize, extend: f32) -> bool {
        let b = &self.nodes[id];
        let other = &b.carver.river;
        if parent == id {
            let x1 = river.x1 - river.ndx * extend;
            let z1 = river.z1 - river.ndz * extend;
            let x2 = river.x1 + river.dx * 0.5;
            let z2 = river.z1 + river.dz * 0.5;
            if line::intersect(x1, z1, x2, z2, other.x1, other.z1, other.x2, other.z2) {
                return true;
            }
        } else if other.intersects_extended(river, extend) {
            return true;
        }
        // The original repeats the same non-parent intersection check; no state changes.
        b.children
            .iter()
            .any(|child| self.overlaps(*child, river, parent, extend))
    }
    fn record_bounds(&self, id: usize, bounds: &mut BoundsBuilder) {
        let b = &self.nodes[id];
        let r = &b.carver.river;
        bounds.record(r.min_x, r.min_z);
        bounds.record(r.max_x, r.max_z);
        for child in &b.children {
            self.record_bounds(*child, bounds);
        }
        for lake in &b.lakes {
            lake.record_bounds(bounds);
        }
        for wetland in &b.wetlands {
            wetland.record_bounds(bounds);
        }
    }
    fn build(&self, id: usize, bounds: Bounds) -> Network {
        let b = &self.nodes[id];
        Network {
            river_carver: b.carver.clone(),
            lakes: b.lakes.clone(),
            wetlands: b.wetlands.clone(),
            children: b
                .children
                .iter()
                .map(|child| self.build(*child, Bounds::NONE))
                .collect(),
            bounds,
        }
    }
}
fn profile(p: RiverProfile, l: Levels, length: i32, main: bool, order: i32) -> RiverConfig {
    RiverConfig::builder(l)
        .bank_height(p.min_bank_height, p.max_bank_height)
        .bank_width(p.bank_width)
        .bed_width(p.bed_width)
        .bed_depth(p.bed_depth)
        .fade(p.fade)
        .length(length)
        .main(main)
        .order(order)
        .build()
}
impl RiverGenerator {
    /// The original constructor reads Seed.root(), never consuming Seed.next().
    pub fn new(
        continent: Arc<AdvancedContinent>,
        preset: &KernelPreset,
        levels: Levels,
        root_seed: i32,
    ) -> R<Self> {
        let r = &preset.rivers;
        if !(0..=256).contains(&r.river_count)
            || !preset.world.control_points.inland.is_finite()
            || ![
                r.main_rivers.fade,
                r.branch_rivers.fade,
                r.lakes.chance,
                r.wetlands.chance,
            ]
            .iter()
            .all(|f| f.is_finite())
        {
            return Err(GraphError::InvalidSourceParameters(
                "bounded finite river settings and count 0..256",
            ));
        }
        Ok(Self {
            continent,
            levels,
            continent_scale: preset.world.continent.continent_scale,
            min_edge_value: preset.world.control_points.inland,
            seed: root_seed.wrapping_add(r.seed_offset),
            count: r.river_count,
            main: profile(r.main_rivers, levels, 5000, true, 0),
            fork: profile(r.branch_rivers, levels, 4500, false, 1),
            lake: LakeConfig::from_settings(&r.lakes, levels),
            wetland: WetlandConfig::from_settings(&r.wetlands),
        })
    }
    pub fn generate(&self, x: i32, z: i32, id: i64) -> R<Rivermap> {
        let mut random = JavaRandom::new(id.wrapping_add(self.seed as i64));
        self.generate_with_random(x, z, id, &mut random)
    }
    /// Also exposes a caller-owned RNG for independent stage/checkpoint qualification.
    pub fn generate_with_random(
        &self,
        x: i32,
        z: i32,
        id: i64,
        random: &mut JavaRandom,
    ) -> R<Rivermap> {
        let warp = GenWarp::new(id as i32, self.continent_scale)?;
        let mut arena = Arena::default();
        let mut roots = self.roots(x, z, random, &mut arena)?;
        random.shuffle(&mut roots).expect("bounded root list");
        for root in &roots {
            self.forks(*root, MAIN_SPACING, random, &roots, &mut arena, 0)?;
        }
        for root in &roots {
            self.wetlands(*root, random, &mut arena)?;
        }
        let networks = roots
            .into_iter()
            .map(|root| {
                let mut bounds = BoundsBuilder::new();
                arena.record_bounds(root, &mut bounds);
                arena.build(root, bounds.build())
            })
            .collect();
        Ok(Rivermap {
            x,
            z,
            networks,
            warp,
        })
    }
    fn roots(&self, x: i32, z: i32, random: &mut JavaRandom, arena: &mut Arena) -> R<Vec<usize>> {
        let start = random.next_f32();
        let spacing = 6.2831855 / self.count as f32;
        let space_var = spacing * 0.75;
        let space_bias = -space_var / 2.0;
        let mut roots = Vec::with_capacity(self.count as usize);
        for i in 0..self.count {
            let variance = random.next_f32() * space_var + space_bias;
            let angle = start + spacing * i as f32 + variance;
            let dx = u::sin(angle);
            let dz = u::cos(angle);
            let start_mod = 0.05 + random.next_f32() * 0.45;
            let length = self.continent.distance_to_ocean(x, z, dx, dz)?;
            let start_dist = java_max(400.0, start_mod * length);
            let x2 = x as f32 + dx * start_dist;
            let z2 = z as f32 + dz * start_dist;
            let x3 = x as f32 + dx * length;
            let z3 = z as f32 + dz * length;
            let valley_size = 275.0 * MAIN_VALLEY.next(random);
            let river = River::new(
                x2 as i32 as f32,
                z2 as i32 as f32,
                x3 as i32 as f32,
                z3 as i32 as f32,
            );
            let settings = CarverSettings {
                valley_curve: RiverCarver::valley_type(random),
                fade_in: self.main.fade,
                valley_size,
                connecting: false,
            };
            let warp = RiverWarp::create(0.1, 0.85, random);
            let id = arena.add(RiverCarver::new(
                river,
                warp,
                self.main.clone(),
                settings,
                self.levels,
            ))?;
            roots.push(id);
            self.add_lake(id, random, arena);
        }
        Ok(roots)
    }
    fn forks(
        &self,
        parent: usize,
        spacing: Variance,
        random: &mut JavaRandom,
        roots: &[usize],
        arena: &mut Arena,
        depth: i32,
    ) -> R<()> {
        if depth > 2 {
            return Ok(());
        }
        let length = 0.44 * arena.nodes[parent].carver.river.length;
        if length < 300.0 {
            return Ok(());
        }
        let mut direction = if random.next_bool() { 1 } else { -1 };
        let mut offset = 0.25;
        while offset < 0.9 {
            direction = -direction;
            let parent_angle = arena.nodes[parent].carver.river.angle();
            let fork_angle = direction as f32 * 6.2831855 * FORK_ANGLE.next(random);
            let angle = parent_angle + fork_angle;
            let dx = u::sin(angle);
            let dz = u::cos(angle);
            let (x1, z1) = arena.nodes[parent].carver.river.pos(offset);
            if self.continent.edge_value(x1, z1)? >= self.min_edge_value {
                let x2 = x1 - dx * length;
                let z2 = z1 - dz * length;
                if self.continent.edge_value(x2, z2)? >= self.min_edge_value {
                    let config = arena.nodes[parent]
                        .carver
                        .create_fork_config(offset, self.levels);
                    let river = River::new(x2, z2, x1, z1);
                    if !roots
                        .iter()
                        .any(|id| arena.overlaps(*id, &river, parent, 250.0))
                    {
                        let valley_size = 275.0 * FORK_VALLEY.next(random);
                        let settings = CarverSettings {
                            valley_curve: RiverCarver::valley_type(random),
                            connecting: true,
                            fade_in: self.fork.fade,
                            valley_size,
                        };
                        let warp = arena.nodes[parent]
                            .carver
                            .warp
                            .create_child(0.15, 0.75, 0.65, random);
                        let child = arena.add(RiverCarver::new(
                            river,
                            warp,
                            config,
                            settings,
                            self.levels,
                        ))?;
                        arena.nodes[parent].children.push(child);
                        self.forks(child, FORK_SPACING, random, roots, arena, depth + 1)?;
                    }
                }
            }
            offset += spacing.next(random);
        }
        self.add_lake(parent, random, arena);
        Ok(())
    }
    fn add_lake(&self, id: usize, random: &mut JavaRandom, arena: &mut Arena) {
        if random.next_f32() <= self.lake.chance {
            let size = self.lake.size_min + random.next_f32() * self.lake.size_range;
            let r = &arena.nodes[id].carver.river;
            let (x, z) = (r.x1, r.z1);
            if !arena.nodes[id]
                .lakes
                .iter()
                .any(|lake| lake.overlaps(x, z, size * size))
            {
                arena.nodes[id]
                    .lakes
                    .push(Lake::new((x, z), size, 1.0, &self.lake));
            }
        }
    }
    fn wetlands(&self, id: usize, random: &mut JavaRandom, arena: &mut Arena) -> R<()> {
        if random
            .bounded_i32(self.wetland.skip_size)
            .expect("positive configured wetland skip")
            == 0
        {
            let width = self.wetland.width.next(random);
            let length = self.wetland.length.next(random);
            let river = &arena.nodes[id].carver.river;
            let river_length = river.length;
            let start_pos = random.next_f32() * 0.75;
            let end_pos = start_pos + random.next_f32() * (length / river_length);
            let start = river.pos(start_pos);
            let end = river.pos(end_pos);
            arena.nodes[id].wetlands.push(Wetland::new(
                random.next_i32(),
                start,
                end,
                width,
                self.levels,
            )?);
        }
        let children = arena.nodes[id].children.clone();
        for child in children {
            self.wetlands(child, random, arena)?;
        }
        Ok(())
    }
}
