//! MIT ReTerraForged Rivermap coordinate/domain and root traversal.
use crate::{
    cell::Cell, graph::GraphError, river_domain::GenWarp, river_network::Network,
    terrain::TerrainRegistry,
};
#[derive(Clone, Debug)]
pub struct Rivermap {
    pub x: i32,
    pub z: i32,
    pub networks: Vec<Network>,
    pub warp: GenWarp,
}
impl Rivermap {
    pub fn apply(
        &self,
        cell: &mut Cell,
        x: f32,
        z: f32,
        terrains: &TerrainRegistry,
    ) -> Result<(), GraphError> {
        let rx = self.warp.river.get_x(x, z, 0);
        let rz = self.warp.river.get_z(x, z, 0);
        let lx = self.warp.lake.offset_x(rx, rz, 0);
        let lz = self.warp.lake.offset_z(rx, rz, 0);
        for network in &self.networks {
            if network.contains(rx, rz) {
                network.carve(cell, rx, rz, lx, lz, terrains)?;
            }
        }
        Ok(())
    }
}
