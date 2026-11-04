//! Bounded completed-value cache and deduplicated concurrent tile construction.
use crate::{graph::GraphError, tile::TileBuffer};
use std::{
    collections::{HashMap, VecDeque},
    sync::{Arc, Condvar, Mutex},
};
type R<T> = Result<T, GraphError>;
type Key = (i32, i32);
#[derive(Debug, Default)]
struct Flight {
    outcome: Mutex<Option<R<Arc<TileBuffer>>>>,
    ready: Condvar,
}
impl Flight {
    fn wait(&self) -> R<Arc<TileBuffer>> {
        let mut outcome = self
            .outcome
            .lock()
            .map_err(|_| GraphError::InvalidSourceParameters("tile flight poisoned"))?;
        while outcome.is_none() {
            outcome = self
                .ready
                .wait(outcome)
                .map_err(|_| GraphError::InvalidSourceParameters("tile flight poisoned"))?;
        }
        outcome.as_ref().expect("flight complete").clone()
    }
}
#[derive(Debug, Default)]
struct State {
    tiles: HashMap<Key, Arc<TileBuffer>>,
    order: VecDeque<Key>,
    flights: HashMap<Key, Arc<Flight>>,
    generation: u64,
}
#[derive(Debug)]
pub(crate) struct TileWorkCache {
    capacity: usize,
    max_in_flight: usize,
    state: Mutex<State>,
    space: Condvar,
}
struct BuildGuard<'a> {
    owner: &'a TileWorkCache,
    key: Key,
    generation: u64,
    flight: Arc<Flight>,
    finished: bool,
}
impl BuildGuard<'_> {
    fn finish(mut self, outcome: R<Arc<TileBuffer>>) -> R<Arc<TileBuffer>> {
        self.finished = true;
        self.owner
            .publish(self.key, self.generation, &self.flight, outcome)
    }
}
impl Drop for BuildGuard<'_> {
    fn drop(&mut self) {
        if !self.finished {
            let _ = self.owner.publish(
                self.key,
                self.generation,
                &self.flight,
                Err(GraphError::InvalidSourceParameters(
                    "native tile construction unwound",
                )),
            );
        }
    }
}
impl TileWorkCache {
    pub fn new(capacity: usize, max_in_flight: usize) -> Self {
        Self {
            capacity,
            max_in_flight,
            state: Mutex::new(State::default()),
            space: Condvar::new(),
        }
    }
    pub fn get(&self, key: Key, build: impl FnOnce() -> R<Arc<TileBuffer>>) -> R<Arc<TileBuffer>> {
        let (flight, generation) = loop {
            let mut state = self
                .state
                .lock()
                .map_err(|_| GraphError::InvalidSourceParameters("native tile cache poisoned"))?;
            if let Some(tile) = state.tiles.get(&key).cloned() {
                state.order.retain(|k| *k != key);
                state.order.push_back(key);
                return Ok(tile);
            }
            if let Some(flight) = state.flights.get(&key).cloned() {
                drop(state);
                return flight.wait();
            }
            if state.flights.len() >= self.max_in_flight {
                state = self.space.wait(state).map_err(|_| {
                    GraphError::InvalidSourceParameters("native tile cache poisoned")
                })?;
                drop(state);
                continue;
            }
            let flight = Arc::new(Flight::default());
            let generation = state.generation;
            state.flights.insert(key, flight.clone());
            break (flight, generation);
        };
        let guard = BuildGuard {
            owner: self,
            key,
            generation,
            flight,
            finished: false,
        };
        guard.finish(build())
    }
    fn publish(
        &self,
        key: Key,
        generation: u64,
        flight: &Arc<Flight>,
        outcome: R<Arc<TileBuffer>>,
    ) -> R<Arc<TileBuffer>> {
        let outcome = match self.state.lock() {
            Ok(mut state) => {
                if state
                    .flights
                    .get(&key)
                    .is_some_and(|value| Arc::ptr_eq(value, flight))
                {
                    state.flights.remove(&key);
                }
                if state.generation == generation {
                    if let Ok(tile) = &outcome {
                        if state.tiles.len() == self.capacity {
                            if let Some(old) = state.order.pop_front() {
                                state.tiles.remove(&old);
                            }
                        }
                        state.tiles.insert(key, tile.clone());
                        state.order.push_back(key);
                    }
                }
                outcome
            }
            Err(_) => Err(GraphError::InvalidSourceParameters(
                "native tile cache poisoned",
            )),
        };
        *flight
            .outcome
            .lock()
            .unwrap_or_else(|poison| poison.into_inner()) = Some(outcome.clone());
        flight.ready.notify_all();
        self.space.notify_all();
        outcome
    }
    pub fn clear(&self) -> R<()> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| GraphError::InvalidSourceParameters("native tile cache poisoned"))?;
        state.tiles.clear();
        state.order.clear();
        state.generation = state.generation.wrapping_add(1);
        Ok(())
    }
    pub fn counts(&self) -> R<(usize, usize)> {
        let state = self
            .state
            .lock()
            .map_err(|_| GraphError::InvalidSourceParameters("native tile cache poisoned"))?;
        Ok((state.tiles.len(), state.flights.len()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        panic::{AssertUnwindSafe, catch_unwind},
        sync::{
            Barrier,
            atomic::{AtomicUsize, Ordering},
            mpsc,
        },
        time::Duration,
    };
    fn tile() -> R<Arc<TileBuffer>> {
        Ok(Arc::new(TileBuffer::new(0, 0, 0, 0)?))
    }
    #[test]
    fn failed_or_unwound_work_can_retry_without_cache_publication() {
        let cache = TileWorkCache::new(1, 1);
        let error = GraphError::NodeBudget;
        assert_eq!(cache.get((0, 0), || Err(error)).unwrap_err(), error);
        assert_eq!(cache.counts().unwrap(), (0, 0));
        assert!(
            catch_unwind(AssertUnwindSafe(
                || cache.get((0, 0), || panic!("controlled builder unwind"))
            ))
            .is_err()
        );
        assert_eq!(cache.counts().unwrap(), (0, 0));
        let first = cache.get((0, 0), tile).unwrap();
        assert!(Arc::ptr_eq(
            &first,
            &cache
                .get((0, 0), || panic!("cache hit must not rebuild"))
                .unwrap()
        ));
    }
    #[test]
    fn lru_touch_eviction_and_clear_release_owned_values() {
        let cache = TileWorkCache::new(2, 1);
        let first = cache.get((0, 0), tile).unwrap();
        let weak_first = Arc::downgrade(&first);
        let second = cache.get((1, 0), tile).unwrap();
        let weak_second = Arc::downgrade(&second);
        drop(second);
        cache.get((0, 0), tile).unwrap();
        cache.get((2, 0), tile).unwrap();
        assert!(weak_second.upgrade().is_none());
        assert!(weak_first.upgrade().is_some());
        cache.clear().unwrap();
        drop(first);
        assert!(weak_first.upgrade().is_none());
        assert_eq!(cache.counts().unwrap(), (0, 0));
    }
    #[test]
    fn concurrent_readers_share_one_build_and_one_arc() {
        let cache = TileWorkCache::new(1, 2);
        let gate = Barrier::new(8);
        let builds = AtomicUsize::new(0);
        let results = std::thread::scope(|scope| {
            let threads: Vec<_> = (0..8)
                .map(|_| {
                    scope.spawn(|| {
                        gate.wait();
                        cache
                            .get((0, 0), || {
                                builds.fetch_add(1, Ordering::SeqCst);
                                tile()
                            })
                            .unwrap()
                    })
                })
                .collect();
            threads
                .into_iter()
                .map(|t| t.join().unwrap())
                .collect::<Vec<_>>()
        });
        assert_eq!(builds.load(Ordering::SeqCst), 1);
        assert!(results.iter().all(|v| Arc::ptr_eq(v, &results[0])));
    }
    #[test]
    fn clear_during_build_does_not_republish_old_generation() {
        let cache = TileWorkCache::new(1, 1);
        let (started_tx, started_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        let old = std::thread::scope(|scope| {
            let cache = &cache;
            let worker = scope.spawn(move || {
                cache.get((0, 0), || {
                    started_tx.send(()).unwrap();
                    release_rx.recv_timeout(Duration::from_secs(5)).unwrap();
                    tile()
                })
            });
            started_rx.recv_timeout(Duration::from_secs(5)).unwrap();
            cache.clear().unwrap();
            release_tx.send(()).unwrap();
            worker.join().unwrap().unwrap()
        });
        assert_eq!(cache.counts().unwrap(), (0, 0));
        let new = cache.get((0, 0), tile).unwrap();
        assert!(!Arc::ptr_eq(&old, &new));
    }
    #[test]
    fn distinct_builds_run_in_parallel_with_bounded_slots() {
        let cache = TileWorkCache::new(3, 2);
        let (started_tx, started_rx) = mpsc::channel();
        let gate = (Mutex::new(false), Condvar::new());
        let active = AtomicUsize::new(0);
        let peak = AtomicUsize::new(0);
        std::thread::scope(|scope| {
            let mut workers = vec![];
            for x in 0..3 {
                let started = &started_tx;
                let gate = &gate;
                let cache = &cache;
                let active = &active;
                let peak = &peak;
                workers.push(scope.spawn(move || {
                    cache.get((x, 0), || {
                        let current = active.fetch_add(1, Ordering::SeqCst) + 1;
                        peak.fetch_max(current, Ordering::SeqCst);
                        started.send(x).unwrap();
                        let mut open = gate.0.lock().unwrap();
                        while !*open {
                            open = gate.1.wait(open).unwrap();
                        }
                        drop(open);
                        active.fetch_sub(1, Ordering::SeqCst);
                        tile()
                    })
                }));
            }
            let first = started_rx.recv_timeout(Duration::from_secs(5));
            let second = started_rx.recv_timeout(Duration::from_secs(5));
            let third = started_rx.recv_timeout(Duration::from_millis(20));
            *gate.0.lock().unwrap() = true;
            gate.1.notify_all();
            for worker in workers {
                worker.join().unwrap().unwrap();
            }
            assert!(first.is_ok() && second.is_ok());
            assert!(third.is_err());
        });
        assert_eq!(peak.load(Ordering::SeqCst), 2);
        assert_eq!(cache.counts().unwrap(), (3, 0));
    }
}
