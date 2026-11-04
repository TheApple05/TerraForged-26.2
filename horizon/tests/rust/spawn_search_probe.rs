use horizon_core::density::terraforged_runtime::kernel;
use horizon_world::{registry::BlockRegistry, worldgen::{generator::{GeneratorData, terraforged_world}, overrides::WorldgenOverrides}};
use std::{fs, io::{self, Write}, path::Path, time::Instant};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let arguments: Vec<_> = std::env::args().skip(1).collect();
    let resources = Path::new(&arguments[0]);
    let registry = Box::leak(Box::new(BlockRegistry::from_facts_json(fs::File::open(resources.join("facts.json"))?)?));
    let data = GeneratorData::load_with(&resources.join("worldgen"), registry, &WorldgenOverrides::default())?;
    let opened = terraforged_world::open(data, 8675309, kernel::builtin_preset_json(), kernel::builtin_noises_json(), true)?;
    let center = opened.kernel.upgrade().ok_or("missing kernel")?.heightmap().continent.nearest_center(0.0, 0.0)?;
    println!("CORE_CONTINENT_CENTER {} {}", center.0, center.1);
    io::stdout().flush()?;
    let started = Instant::now();
    let actual = opened.generator.spawn_position()?;
    println!("NATIVE_SPAWN_RESULT {} {} elapsedSeconds={}", actual[0], actual[1], started.elapsed().as_secs_f64());
    Ok(())
}
