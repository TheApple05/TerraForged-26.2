//! Owned bounded LRU for deterministic maps. No global manager or polling thread.
use crate::{graph::GraphError, river_generator::RiverGenerator, rivermap::Rivermap};
use std::{
    collections::{HashMap, VecDeque},
    sync::{Arc, Mutex},
};
#[derive(Debug, Default)]
struct State {
    maps: HashMap<(i32, i32), Arc<Rivermap>>,
    order: VecDeque<(i32, i32)>,
}
#[derive(Debug)]
pub struct RiverCache {
    generator: RiverGenerator,
    capacity: usize,
    state: Mutex<State>,
}
impl RiverCache {
    pub fn new(generator: RiverGenerator, capacity: usize) -> Result<Self, GraphError> {
        if !(1..=256).contains(&capacity) {
            return Err(GraphError::InvalidSourceParameters(
                "river cache capacity 1..256",
            ));
        }
        Ok(Self {
            generator,
            capacity,
            state: Mutex::new(State::default()),
        })
    }
    pub fn get(&self, x: i32, z: i32) -> Result<Arc<Rivermap>, GraphError> {
        let key = (x, z);
        let mut state = self
            .state
            .lock()
            .map_err(|_| GraphError::InvalidSourceParameters("river cache poisoned"))?;
        if let Some(map) = state.maps.get(&key).cloned() {
            state.order.retain(|k| *k != key);
            state.order.push_back(key);
            return Ok(map);
        }
        // Serialize misses to avoid publishing duplicate maps for concurrent readers.
        // Java PosUtil.pack: left signed coordinate occupies the upper 32 bits.
        let id = ((x as u32 as u64) << 32) | (z as u32 as u64);
        let map = Arc::new(self.generator.generate(x, z, id as i64)?);
        if state.maps.len() == self.capacity {
            if let Some(old) = state.order.pop_front() {
                state.maps.remove(&old);
            }
        }
        state.maps.insert(key, map.clone());
        state.order.push_back(key);
        Ok(map)
    }
    pub fn len(&self) -> Result<usize, GraphError> {
        Ok(self
            .state
            .lock()
            .map_err(|_| GraphError::InvalidSourceParameters("river cache poisoned"))?
            .maps
            .len())
    }
    pub fn clear(&self) -> Result<(), GraphError> {
        *self
            .state
            .lock()
            .map_err(|_| GraphError::InvalidSourceParameters("river cache poisoned"))? =
            State::default();
        Ok(())
    }
}
