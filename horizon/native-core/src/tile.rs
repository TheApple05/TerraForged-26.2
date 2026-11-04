//! Owned MIT ReTerraForged tile backing and generation, including raw row wrapping.
use crate::{cell::Cell, graph::GraphError, heightmap::Heightmap, layout::Size};
#[derive(Clone, Debug)]
pub struct TileBuffer {
    pub x: i32,
    pub z: i32,
    pub factor: i32,
    pub border_chunks: i32,
    pub size: Size,
    pub cells: Vec<Cell>,
}
impl TileBuffer {
    pub fn new(x: i32, z: i32, factor: i32, border_chunks: i32) -> Result<Self, GraphError> {
        if !(0..=4).contains(&factor) || !(0..=4).contains(&border_chunks) {
            return Err(GraphError::InvalidSourceParameters(
                "native tile factor/border 0..4",
            ));
        }
        let size = Size::blocks(factor, border_chunks);
        Ok(Self {
            x,
            z,
            factor,
            border_chunks,
            size,
            cells: vec![Cell::default(); size.array_size as usize],
        })
    }
    pub fn block_x(&self) -> i32 {
        self.x.wrapping_shl(self.factor as u32).wrapping_shl(4)
    }
    pub fn block_z(&self) -> i32 {
        self.z.wrapping_shl(self.factor as u32).wrapping_shl(4)
    }
    /// Source checks only the flattened index, not each coordinate separately.
    pub fn raw_index(&self, x: i32, z: i32) -> Option<usize> {
        let index = self.size.index_of(x, z);
        if index >= 0 && index < self.size.array_size {
            Some(index as usize)
        } else {
            None
        }
    }
    pub fn get_raw(&self, x: i32, z: i32) -> Option<&Cell> {
        self.raw_index(x, z).map(|index| &self.cells[index])
    }
    pub fn lookup(&self, world_x: i32, world_z: i32) -> &Cell {
        let x = self.size.border + self.size.mask_index(world_x);
        let z = self.size.border + self.size.mask_index(world_z);
        &self.cells[self.size.index_of(x, z) as usize]
    }
    pub fn populate(&mut self, map: &Heightmap) -> Result<(), GraphError> {
        self.populate_coordinates(map, None)
    }
    pub fn populate_zoomed(
        &mut self,
        map: &Heightmap,
        center_x: f32,
        center_z: f32,
        zoom: f32,
    ) -> Result<(), GraphError> {
        if !center_x.is_finite() || !center_z.is_finite() || !zoom.is_finite() || zoom <= 0.0 {
            return Err(GraphError::InvalidSourceParameters(
                "finite positive native tile zoom",
            ));
        }
        self.populate_coordinates(map, Some((center_x, center_z, zoom)))
    }
    fn populate_coordinates(
        &mut self,
        map: &Heightmap,
        zoom: Option<(f32, f32, f32)>,
    ) -> Result<(), GraphError> {
        let chunks = Size::chunks(self.factor, self.border_chunks);
        let (translate_x, translate_z, scale) = zoom
            .map(|(x, z, zoom)| {
                (
                    x - self.size.size as f32 * zoom / 2.0,
                    z - self.size.size as f32 * zoom / 2.0,
                    zoom,
                )
            })
            .unwrap_or((0.0, 0.0, 1.0));
        for cz in 0..chunks.total {
            for cx in 0..chunks.total {
                let bx = self
                    .x
                    .wrapping_shl(self.factor as u32)
                    .wrapping_add(cx)
                    .wrapping_sub(self.border_chunks)
                    .wrapping_shl(4);
                let bz = self
                    .z
                    .wrapping_shl(self.factor as u32)
                    .wrapping_add(cz)
                    .wrapping_sub(self.border_chunks)
                    .wrapping_shl(4);
                let mut hint: Option<std::sync::Arc<crate::rivermap::Rivermap>> = None;
                for dz in 0..16 {
                    for dx in 0..16 {
                        let world_x = bx.wrapping_add(dx) as f32 * scale + translate_x;
                        let world_z = bz.wrapping_add(dz) as f32 * scale + translate_z;
                        let index = self.size.index_of((cx << 4) + dx, (cz << 4) + dz) as usize;
                        let cell = &mut self.cells[index];
                        map.apply_terrain(cell, world_x, world_z)?;
                        if !hint
                            .as_ref()
                            .is_some_and(|r| r.x == cell.continent_x && r.z == cell.continent_z)
                        {
                            hint = Some(map.get_rivermap(cell.continent_x, cell.continent_z)?);
                        }
                        map.apply_rivers(
                            cell,
                            world_x,
                            world_z,
                            hint.as_ref().expect("river hint initialized"),
                        )?;
                        map.apply_climate(cell, world_x, world_z, true)?;
                    }
                }
            }
        }
        Ok(())
    }
}
