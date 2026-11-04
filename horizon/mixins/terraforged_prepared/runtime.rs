// Copyright (c) 2026 TerraForged Revival contributors. MIT; see the repository LICENSE.
// Adapter for Horizon's MIT generator API at 9dd5c4093977c718c5c0349032fd3a5434e86afc.
// This imports prepared TerraForged terrain. It does not implement infinite native generation.
use super::*;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{path::PathBuf, sync::Mutex};

const MAX_FILE: u64 = 8 * 1024 * 1024;
const CACHE_CHUNKS: usize = 32;

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct Manifest {
    schema: u32,
    source_minecraft: String,
    target_minecraft: String,
    seed: i64,
    min_y: i32,
    height: i32,
    sea_level: i32,
    spawn: [i32; 2],
    source_mod_sha256: String,
    chunks: BTreeMap<String, String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct ChunkFile {
    schema: u32,
    position: [i32; 2],
    min_y: i32,
    height: i32,
    block_palette: Vec<State>,
    /// X fastest, then Z, then Y, bottom to top. Pairs are palette index/count.
    blocks: Vec<[u32; 2]>,
    biome_palette: Vec<String>,
    /// Quart X fastest, then Z, then Y, bottom to top.
    biomes: Vec<[u32; 2]>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct State {
    name: String,
    properties: BTreeMap<String, String>,
}

struct PreparedChunk {
    blocks: Vec<BlockStateId>,
    biomes: Vec<BiomeId>,
}

struct Prepared {
    data: Arc<GeneratorData>,
    root: PathBuf,
    manifest: Manifest,
    cache: Mutex<HashMap<(i32, i32), Arc<PreparedChunk>>>,
}

fn bounded(path: &Path) -> Result<Vec<u8>, GeneratorError> {
    let file = fs::File::open(path)?;
    if file.metadata()?.len() > MAX_FILE {
        return Err(resource("TerraForged prepared file exceeds 8 MiB"));
    }
    use std::io::Read;
    let mut bytes = Vec::new();
    file.take(MAX_FILE + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > MAX_FILE {
        return Err(resource("TerraForged prepared file exceeds 8 MiB"));
    }
    Ok(bytes)
}

fn valid_sha(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

fn expand<T: Copy>(
    runs: &[[u32; 2]],
    palette: &[T],
    count: usize,
) -> Result<Vec<T>, GeneratorError> {
    let mut result = Vec::with_capacity(count);
    for &[index, length] in runs {
        let item = *palette
            .get(index as usize)
            .ok_or_else(|| resource("prepared palette index"))?;
        if length == 0 || length as usize > count.saturating_sub(result.len()) {
            return Err(resource("prepared RLE length"));
        }
        result.resize(result.len() + length as usize, item);
    }
    if result.len() != count {
        return Err(resource("prepared RLE does not cover the entire chunk"));
    }
    Ok(result)
}

/// Open a finite prepared world. Every requested chunk must have a verified source snapshot.
/// An absent/malformed chunk or unknown block/biome is an error, never vanilla fallback.
pub fn open(
    data: Arc<GeneratorData>,
    seed: i64,
    root: &Path,
) -> Result<OverworldGenerator, GeneratorError> {
    let manifest: Manifest =
        serde_json::from_slice(&bounded(&root.join("terraforged-prepared.json"))?)?;
    if manifest.schema != 1
        || manifest.source_minecraft != "26.2"
        || manifest.target_minecraft != "26.3"
        || manifest.seed != seed
        || !valid_sha(&manifest.source_mod_sha256)
        || manifest.min_y % 16 != 0
        || manifest.height <= 0
        || manifest.height > 4096
        || manifest.height % 16 != 0
        || manifest.min_y.checked_add(manifest.height).is_none()
        || manifest.chunks.is_empty()
        || manifest.chunks.len() > 4096
        || !manifest.chunks.values().all(|value| valid_sha(value))
    {
        return Err(resource(
            "TerraForged prepared manifest/version/seed/bounds mismatch",
        ));
    }
    // Heightmap::new checks Horizon's supported vertical range before any chunk allocation.
    crate::heightmap::Heightmap::new(HeightmapType::WorldSurface, manifest.min_y, manifest.height)
        .map_err(stage("prepared bounds"))?;
    let prepared = Prepared {
        data,
        root: root.to_owned(),
        manifest,
        cache: Mutex::new(HashMap::new()),
    };
    let spawn = prepared.manifest.spawn;
    prepared.chunk(ChunkPos::new(spawn[0] >> 4, spawn[1] >> 4))?;
    Ok(OverworldGenerator {
        pipeline: Arc::new(prepared),
        noise_clamped: None,
    })
}

impl Prepared {
    fn chunk(&self, pos: ChunkPos) -> Result<Arc<PreparedChunk>, GeneratorError> {
        let key = (pos.x, pos.z);
        if let Some(chunk) = self
            .cache
            .lock()
            .map_err(|_| resource("prepared cache poisoned"))?
            .get(&key)
        {
            return Ok(chunk.clone());
        }
        let name = format!("{},{}", pos.x, pos.z);
        let expected = self.manifest.chunks.get(&name).ok_or_else(|| resource(format!(
            "TerraForged prepared coverage ends before chunk {name}; native infinite generation is not installed"
        )))?;
        let bytes = bounded(
            &self
                .root
                .join("chunks")
                .join(format!("{}.{}.json", pos.x, pos.z)),
        )?;
        let actual = format!("{:x}", Sha256::digest(&bytes));
        if &actual != expected {
            return Err(resource(format!(
                "prepared chunk checksum mismatch: {name}"
            )));
        }
        let raw: ChunkFile = serde_json::from_slice(&bytes)?;
        if raw.schema != 1
            || raw.position != [pos.x, pos.z]
            || raw.min_y != self.manifest.min_y
            || raw.height != self.manifest.height
            || raw.block_palette.is_empty()
            || raw.biome_palette.is_empty()
        {
            return Err(resource("prepared chunk position/bounds/palette mismatch"));
        }
        let palette = raw
            .block_palette
            .into_iter()
            .map(|state| {
                let state = crate::chunk::BlockState {
                    name: state.name,
                    properties: state.properties.into(),
                };
                self.data.registry.id(&state).ok_or_else(|| {
                    resource(format!(
                        "prepared block state absent from Horizon registry: {state:?}"
                    ))
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        let biomes = raw
            .biome_palette
            .iter()
            .map(|name| {
                self.data.biome_id(name).ok_or_else(|| {
                    resource(format!(
                        "prepared biome absent from Horizon registry: {name}"
                    ))
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        let chunk = Arc::new(PreparedChunk {
            blocks: expand(&raw.blocks, &palette, raw.height as usize * 256)?,
            biomes: expand(&raw.biomes, &biomes, raw.height as usize * 4)?,
        });
        let mut cache = self
            .cache
            .lock()
            .map_err(|_| resource("prepared cache poisoned"))?;
        if cache.len() >= CACHE_CHUNKS {
            cache.clear();
        }
        cache.insert(key, chunk.clone());
        Ok(chunk)
    }
    fn biome(&self, q: [i32; 3]) -> Option<BiomeId> {
        let chunk = self.chunk(ChunkPos::new(q[0] >> 2, q[2] >> 2)).ok()?;
        let bottom = self.manifest.min_y >> 2;
        let y = q[1].clamp(bottom, bottom + (self.manifest.height >> 2) - 1) - bottom;
        Some(chunk.biomes[(y * 16 + (q[2] & 3) * 4 + (q[0] & 3)) as usize])
    }
    fn column(&self, x: i32, z: i32) -> Option<LodColumn> {
        let chunk = self.chunk(ChunkPos::new(x >> 4, z >> 4)).ok()?;
        let mut spans: Vec<(i32, i32, BlockStateId)> = Vec::new();
        let (mut top, mut water) = (None, None);
        for y in self.manifest.min_y..self.manifest.min_y + self.manifest.height {
            let state =
                chunk.blocks[((y - self.manifest.min_y) * 256 + (z & 15) * 16 + (x & 15)) as usize];
            let facts = self.data.states.get(state);
            if facts.air {
                continue;
            }
            if let Some((_, max, previous)) = spans.last_mut() {
                if *max == y && *previous == state {
                    *max += 1;
                } else {
                    spans.push((y, y + 1, state));
                }
            } else {
                spans.push((y, y + 1, state));
            }
            if facts.fluid {
                water = Some((y + 1, state));
            } else {
                top = Some((y + 1, state));
            }
        }
        let biome = self.biome([
            x >> 2,
            top.map_or(self.manifest.min_y, |v| v.0 - 1) >> 2,
            z >> 2,
        ])?;
        spans.reverse();
        let water = water.filter(|(height, _)| top.is_none_or(|solid| *height > solid.0));
        Some((
            top.map(|(height, state)| SurfaceSample {
                height,
                state,
                water,
                biome,
            }),
            spans,
        ))
    }
}

impl SeededPipeline for Prepared {
    fn data(&self) -> &Arc<GeneratorData> {
        &self.data
    }
    fn level(&self) -> (i32, i32) {
        (self.manifest.min_y, self.manifest.height)
    }
    fn sea_level(&self) -> i32 {
        self.manifest.sea_level
    }
    fn generate(&self, _: &mut WorkerState, pos: ChunkPos) -> Result<ProtoChunk, GeneratorError> {
        let source = self.chunk(pos)?;
        let (min, height) = self.level();
        let mut terrain = TerrainChunk::new(
            pos,
            min,
            height,
            self.data.registry,
            self.data.states.clone(),
        )
        .map_err(stage("prepared terrain"))?;
        for (index, &state) in source.blocks.iter().enumerate() {
            terrain
                .set_block_raw(
                    (index & 15) as i32,
                    min + (index >> 8) as i32,
                    ((index >> 4) & 15) as i32,
                    state,
                )
                .map_err(stage("prepared blocks"))?;
        }
        terrain.set_generation_stage(PreLightChunkStage::Features);
        terrain
            .prime_heightmaps(&HeightmapType::ALL)
            .map_err(stage("prepared heightmaps"))?;
        let mut biomes = BiomeChunk::new(pos, min, height, source.biomes[0])
            .map_err(stage("prepared biomes"))?;
        biomes
            .fill(min, min + height - 1, |q| {
                let y = q[1] - (min >> 2);
                Ok::<_, String>(source.biomes[(y * 16 + (q[2] & 3) * 4 + (q[0] & 3)) as usize])
            })
            .map_err(stage("prepared biomes"))?;
        Ok(ProtoChunk {
            terrain,
            biomes,
            data: self.data.clone(),
            decorated: true,
            block_entities: Vec::new(),
            structure_starts: Vec::new(),
            own_starts: Arc::new(Vec::new()),
            entities: Vec::new(),
        })
    }
    fn decorate(
        &self,
        _: &mut WorkerState,
        region: &mut [ProtoChunk; 9],
    ) -> Result<DecorateStats, GeneratorError> {
        // Snapshots already include completed Java decoration. Never decorate them a second time.
        for chunk in region {
            if !chunk.decorated {
                return Err(resource("unprepared chunk in prepared decoration region"));
            }
        }
        Ok(DecorateStats::default())
    }
    fn cache_census(&self) -> CacheCensus {
        let cache = self.cache.lock().unwrap_or_else(|e| e.into_inner());
        CacheCensus {
            biome_chunks: cache.len(),
            biome_bytes: cache
                .values()
                .map(|c| c.blocks.len() * 4 + c.biomes.len() * 4)
                .sum(),
            ..CacheCensus::default()
        }
    }
    fn release_caches(&self) {
        self.cache.lock().unwrap_or_else(|e| e.into_inner()).clear();
    }
    fn spawn_position(&self) -> Result<[i32; 2], GeneratorError> {
        Ok(self.manifest.spawn)
    }
    fn surface_sample(
        &self,
        _: &mut WorkerState,
        x: i32,
        z: i32,
        spans: Option<&mut Vec<(i32, i32, BlockStateId)>>,
    ) -> Option<SurfaceSample> {
        let (surface, runs) = self.column(x, z)?;
        if let Some(spans) = spans {
            spans.extend(runs);
        }
        surface
    }
    fn lod_grid(
        &self,
        _: &mut WorkerState,
        x: i32,
        z: i32,
        width: i32,
        side: usize,
        _: bool,
    ) -> Result<Vec<LodColumn>, GeneratorError> {
        if width <= 0 || side == 0 || side > 1024 {
            return Err(resource("invalid prepared LOD grid"));
        }
        let mut result = Vec::with_capacity(side * side);
        for dz in 0..side {
            for dx in 0..side {
                let wx = i32::try_from(x as i64 + dx as i64 * width as i64)
                    .map_err(|_| resource("prepared LOD coordinate overflow"))?;
                let wz = i32::try_from(z as i64 + dz as i64 * width as i64)
                    .map_err(|_| resource("prepared LOD coordinate overflow"))?;
                result.push(
                    self.column(wx, wz)
                        .ok_or_else(|| resource("prepared LOD exceeds terrain coverage"))?,
                );
            }
        }
        Ok(result)
    }
    fn noise_biomes(&self, _: &mut WorkerState, quarts: &[[i32; 3]]) -> Vec<Option<String>> {
        quarts
            .iter()
            .map(|&q| self.biome(q).map(|id| self.data.biome_name(id).to_owned()))
            .collect()
    }
    fn find_closest_biome(
        &self,
        _: &mut WorkerState,
        search: &BiomeSearch<'_>,
    ) -> Option<(String, [i32; 3])> {
        if search.horizontal <= 0 || search.vertical <= 0 || search.radius < 0 {
            return None;
        }
        let ys = out_from_origin(
            search.origin[1],
            search.min_y + 1,
            search.max_y + 1,
            search.vertical,
        );
        for [sx, sz] in spiral_around(search.radius / search.horizontal) {
            let x = search.origin[0].wrapping_add(sx.wrapping_mul(search.horizontal));
            let z = search.origin[2].wrapping_add(sz.wrapping_mul(search.horizontal));
            for &y in &ys {
                if let Some(id) = self.biome([x >> 2, y >> 2, z >> 2]) {
                    let name = self.data.biome_name(id);
                    if (search.allowed)(name) {
                        return Some((name.to_owned(), [x, y, z]));
                    }
                }
            }
        }
        None
    }
    fn end_island_origins(&self, _: ChunkPos) -> Vec<[i32; 3]> {
        Vec::new()
    }
    fn structure_starts(
        &self,
        _: &mut WorkerState,
        _: ChunkPos,
    ) -> Vec<Arc<super::super::structure::start::StructureStart>> {
        Vec::new()
    }
    fn structure_start(
        &self,
        _: &mut WorkerState,
        _: &str,
        _: ChunkPos,
    ) -> Option<Arc<super::super::structure::start::StructureStart>> {
        None
    }
    fn unsupported_features(&self) -> Vec<(String, String)> {
        vec![("terraforged:prepared".into(), "Finite terrain snapshot only; live native terrain generation, structures, block entities and entities are not implemented".into())]
    }
    fn use_saved_start(
        &self,
        _: &mut WorkerState,
        _: ChunkPos,
        _: &str,
        _: &[serde_json::Value],
    ) -> Result<(), String> {
        Err("Prepared terrain does not restore structure metadata".into())
    }
    fn forget_structure_starts(&self) {}
    fn ring_positions(&self, _: &mut WorkerState, _: &str) -> Option<Vec<ChunkPos>> {
        None
    }
    fn region_random(
        &self,
        center: ChunkPos,
    ) -> Box<dyn super::super::structure::legacy::PieceRandom> {
        Box::new(
            XoroshiroRandom::new(self.manifest.seed)
                .fork_positional()
                .from_hash_of("minecraft:worldgen_region_random")
                .fork_positional()
                .at(BlockPos::new(
                    center.x.wrapping_shl(4),
                    0,
                    center.z.wrapping_shl(4),
                )),
        )
    }
    fn locate(
        &self,
        _: &mut WorkerState,
        _: &[String],
        _: [i32; 3],
        _: i32,
        _: &dyn Fn(&str, ChunkPos) -> bool,
    ) -> Option<(String, [i32; 3])> {
        None
    }
    fn set_old_world(&self, _: Arc<dyn super::super::blending::OldWorld>) {}
    fn debug_density_line(&self, _: [i32; 3]) -> Option<String> {
        None
    }
    fn jigsaw_pieces(&self, _: &mut WorkerState, _: &JigsawQuery<'_>) -> Option<JigsawPieces> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rle_fails_closed() {
        assert_eq!(
            expand(&[[0, 2], [1, 1]], &[7, 9], 3).unwrap(),
            vec![7, 7, 9]
        );
        for runs in [
            vec![[0, 0]],
            vec![[2, 3]],
            vec![[0, 2]],
            vec![[0, 4]],
            vec![[0, u32::MAX]],
        ] {
            assert!(expand(&runs, &[7, 9], 3).is_err());
        }
    }
}
