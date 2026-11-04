//! MIT ReTerraForged droplet erosion; brush and update order retained.
use crate::{
    cell::Cell,
    fast_random::FastRandom,
    graph::{GraphError, java_max, java_min},
    noise_util as u,
    preset::ErosionSettings,
    terrain::TerrainRegistry,
    tile::TileBuffer,
    tile_filters_basic::Modifier,
};
type R<T> = Result<T, GraphError>;
#[derive(Debug)]
pub struct Erosion {
    seed: i32,
    size: i32,
    settings: ErosionSettings,
    modifier: Modifier,
    brushes: Vec<Vec<(usize, f32)>>,
}
#[derive(Default, Clone, Copy)]
struct TerrainPos {
    height: f32,
    gx: f32,
    gz: f32,
}
impl TerrainPos {
    fn at(cells: &[Cell], size: i32, px: f32, pz: f32) -> Self {
        let cx = px as i32;
        let cz = pz as i32;
        let x = px - cx as f32;
        let z = pz - cz as f32;
        let index = (cz * size + cx) as usize;
        let nw = cells[index].height;
        let ne = cells[index + 1].height;
        let sw = cells[index + size as usize].height;
        let se = cells[index + size as usize + 1].height;
        Self {
            gx: (ne - nw) * (1.0 - z) + (se - sw) * z,
            gz: (sw - nw) * (1.0 - x) + (se - ne) * x,
            height: nw * (1.0 - x) * (1.0 - z)
                + ne * x * (1.0 - z)
                + sw * (1.0 - x) * z
                + se * x * z,
        }
    }
}
impl Erosion {
    pub fn brush(&self, index: usize) -> Option<&[(usize, f32)]> {
        self.brushes.get(index).map(Vec::as_slice)
    }
    pub fn new(seed: i32, size: i32, settings: ErosionSettings, modifier: Modifier) -> R<Self> {
        if !(16..=512).contains(&size) {
            return Err(GraphError::InvalidSourceParameters(
                "native erosion size 16..512",
            ));
        }
        let mut brushes = Vec::with_capacity((size * size) as usize);
        let mut offsets = vec![];
        let mut sum = 0.0;
        for i in 0..size * size {
            let cx = i % size;
            let cz = i / size;
            if cz <= 4 || cz >= size - 4 || cx <= 5 || cx >= size - 4 {
                offsets.clear();
                sum = 0.0;
                for z in -4..=4 {
                    for x in -4..=4 {
                        let distance = (x * x + z * z) as f32;
                        if distance < 16.0
                            && cx + x >= 0
                            && cx + x < size
                            && cz + z >= 0
                            && cz + z < size
                        {
                            let weight = 1.0 - u::sqrt(distance) / 4.0;
                            sum += weight;
                            offsets.push((x, z, weight));
                        }
                    }
                }
            }
            brushes.push(
                offsets
                    .iter()
                    .map(|(x, z, w)| (((cz + z) * size + cx + x) as usize, *w / sum))
                    .collect(),
            );
        }
        Ok(Self {
            seed,
            size,
            settings,
            modifier,
            brushes,
        })
    }
    pub fn apply(
        &self,
        tile: &mut TileBuffer,
        iterations: i32,
        terrains: &TerrainRegistry,
    ) -> R<()> {
        if tile.size.total != self.size {
            return Err(GraphError::InvalidSourceParameters(
                "erosion brush/tile size mismatch",
            ));
        }
        let chunk_x = tile.block_x() >> 4;
        let chunk_z = tile.block_z() >> 4;
        let length_chunks = tile.size.total >> 4;
        let border_chunks = tile.size.border >> 4;
        let max_pos = (self.size - 2) as f32;
        let mut random = FastRandom::new(0);
        for i in 0..iterations {
            let iteration_seed = u::seed(self.seed, i);
            for cz in 0..length_chunks {
                for cx in 0..length_chunks {
                    let seed_z = chunk_z.wrapping_add(cz).wrapping_sub(border_chunks);
                    let seed_x = chunk_x.wrapping_add(cx).wrapping_sub(border_chunks);
                    random.seed_with_gamma(u::seed(seed_x, seed_z), iteration_seed);
                    let px = ((cx << 4)
                        + random.next_int_bound(16).expect("positive constant bound"))
                        as f32;
                    let pz = ((cz << 4)
                        + random.next_int_bound(16).expect("positive constant bound"))
                        as f32;
                    self.drop(
                        u::clamp_f32(px, 1.0, max_pos),
                        u::clamp_f32(pz, 1.0, max_pos),
                        &mut tile.cells,
                        terrains,
                    )?;
                }
            }
        }
        Ok(())
    }
    fn drop(
        &self,
        mut px: f32,
        mut pz: f32,
        cells: &mut [Cell],
        terrains: &TerrainRegistry,
    ) -> R<()> {
        let mut dx = 0.0;
        let mut dz = 0.0;
        let mut sediment = 0.0;
        let mut speed = self.settings.droplet_velocity;
        let mut water = self.settings.droplet_volume;
        for _ in 0..self.settings.droplet_lifetime {
            let nx = px as i32;
            let nz = pz as i32;
            let index = (nz * self.size + nx) as usize;
            let ox = px - nx as f32;
            let oz = pz - nz as f32;
            let first = TerrainPos::at(cells, self.size, px, pz);
            dx = dx * 0.05 - first.gx * 0.95;
            dz = dz * 0.05 - first.gz * 0.95;
            let mut length = u::sqrt(dx * dx + dz * dz);
            if length.is_nan() {
                length = 0.0;
            }
            if length != 0.0 {
                dx /= length;
                dz /= length;
            }
            px += dx;
            pz += dz;
            if (dx == 0.0 && dz == 0.0)
                || px < 0.0
                || px >= (self.size - 1) as f32
                || pz < 0.0
                || pz >= (self.size - 1) as f32
            {
                return Ok(());
            }
            let new_height = TerrainPos::at(cells, self.size, px, pz).height;
            let delta = new_height - first.height;
            let capacity = java_max(-delta * speed * water * 4.0, 0.01);
            if sediment > capacity || delta > 0.0 {
                let amount = if delta > 0.0 {
                    java_min(delta, sediment)
                } else {
                    (sediment - capacity) * self.settings.deposite_rate
                };
                sediment -= amount;
                self.deposit(
                    &mut cells[index],
                    amount * (1.0 - ox) * (1.0 - oz),
                    terrains,
                )?;
                self.deposit(&mut cells[index + 1], amount * ox * (1.0 - oz), terrains)?;
                self.deposit(
                    &mut cells[index + self.size as usize],
                    amount * (1.0 - ox) * oz,
                    terrains,
                )?;
                self.deposit(
                    &mut cells[index + self.size as usize + 1],
                    amount * ox * oz,
                    terrains,
                )?;
            } else {
                let amount = java_min((capacity - sediment) * self.settings.erosion_rate, -delta);
                for (node, weight) in &self.brushes[index] {
                    let cell = &mut cells[*node];
                    let delta_sediment = java_min(cell.height, amount * weight);
                    if !cell.erosion_mask {
                        let change = self.modifier.modify(cell, delta_sediment, terrains)?;
                        cell.height -= change;
                        cell.height_erosion -= change;
                    }
                    sediment += delta_sediment;
                }
            }
            speed = u::sqrt(speed * speed + delta * 3.0);
            water *= 0.99;
            if speed.is_nan() {
                speed = 0.0;
            }
        }
        Ok(())
    }
    fn deposit(&self, cell: &mut Cell, amount: f32, terrains: &TerrainRegistry) -> R<()> {
        if !cell.erosion_mask {
            let change = self.modifier.modify(cell, amount, terrains)?;
            cell.height += change;
            cell.sediment += change;
        }
        Ok(())
    }
}
