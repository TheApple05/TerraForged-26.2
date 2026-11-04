//! MIT ReTerraForged immutable Network traversal. See river-networks.md.
use crate::{
    bounds::Bounds, cell::Cell, graph::GraphError, lake::Lake, line::distance_on_line,
    river::River, river_carver::RiverCarver, terrain::TerrainRegistry, wetland::Wetland,
};

#[derive(Clone, Debug)]
pub struct Network {
    pub river_carver: RiverCarver,
    pub lakes: Vec<Lake>,
    pub wetlands: Vec<Wetland>,
    pub children: Vec<Network>,
    pub bounds: Bounds,
}

impl Network {
    pub fn contains(&self, x: f32, z: f32) -> bool {
        self.bounds.contains(x, z)
    }

    pub fn carve(
        &self,
        cell: &mut Cell,
        x: f32,
        z: f32,
        nx: f32,
        nz: f32,
        terrains: &TerrainRegistry,
    ) -> Result<(), GraphError> {
        let river = &self.river_carver.river;
        let warp = &self.river_carver.warp;
        let mut t = distance_on_line(x, z, river.x1, river.z1, river.x2, river.z2);
        let px = x;
        let pz = z;
        let pt = t;
        let mut x = x;
        let mut z = z;
        if warp.test(t) {
            let (ox, oz) = warp.offset(x, z, t, river);
            x += ox;
            z += oz;
            t = distance_on_line(x, z, river.x1, river.z1, river.x2, river.z2);
        }
        self.river_carver
            .carve(cell, px, pz, pt, x, z, t, terrains)?;
        self.carve_wetlands(cell, x, z, nx, nz);
        self.carve_lakes(cell, x, z, nx, nz);
        for network in &self.children {
            network.carve(cell, x, z, nx, nz, terrains)?;
        }
        Ok(())
    }

    pub fn overlaps(&self, river: &River, extend: f32) -> bool {
        Self::overlaps_river_carver(river, &self.river_carver, extend)
            || Self::overlaps_networks(river, &self.children, extend)
    }

    fn carve_wetlands(&self, cell: &mut Cell, x: f32, z: f32, nx: f32, nz: f32) {
        for wetland in &self.wetlands {
            wetland.apply(cell, x + nx, z + nz, x, z);
        }
    }

    fn carve_lakes(&self, cell: &mut Cell, x: f32, z: f32, nx: f32, nz: f32) {
        let lx = x + nx;
        let lz = z + nz;
        for lake in &self.lakes {
            lake.apply(cell, lx, lz);
        }
    }

    fn overlaps_river_carver(river: &River, river_carver: &RiverCarver, extend: f32) -> bool {
        river_carver.river.intersects_extended(river, extend)
    }

    fn overlaps_networks(river: &River, networks: &[Network], extend: f32) -> bool {
        for network in networks {
            if network.overlaps(river, extend) {
                return true;
            }
        }
        false
    }
}
