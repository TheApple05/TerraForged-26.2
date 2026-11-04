use serde_json::Value;
use std::{
    fs,
    io::{self, Write},
};
use terraforged_native_core::{cell::Cell, classic_biomes, terrain::TerrainRegistry};
fn value(raw: &Value) -> f32 {
    f32::from_bits(
        u32::from_str_radix(raw.as_str().expect("hex float bits"), 16).expect("valid float bits"),
    )
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 2 {
        return Err("predefined independent input JSON required".into());
    }
    let input: Value = serde_json::from_str(&fs::read_to_string(&args[1])?)?;
    let mut terrains = TerrainRegistry::new();
    for name in ["lake", "wetland", "beach"] {
        let parent = terrains.named(name).ok_or("parent terrain")?.id();
        terrains
            .get_or_create(&format!("native_clone_{name}"), Some(parent))
            .map_err(|e| format!("{e:?}"))?;
    }
    let out = io::stdout();
    let mut out = io::BufWriter::new(out.lock());
    for (i, raw) in input["hashBits"]
        .as_array()
        .ok_or("hash inputs")?
        .iter()
        .enumerate()
    {
        writeln!(
            out,
            "hash/{i}\t{:08x}",
            classic_biomes::region_choice(value(raw)).to_bits()
        )?;
    }
    for (p, parent) in input["parents"]
        .as_array()
        .ok_or("parent inputs")?
        .iter()
        .enumerate()
    {
        let parent = parent.as_str().ok_or("parent string")?;
        for (t, terrain) in input["terrains"]
            .as_array()
            .ok_or("terrain inputs")?
            .iter()
            .enumerate()
        {
            let terrain = terrains
                .named(terrain.as_str().ok_or("terrain name")?)
                .ok_or("known terrain")?;
            for (r, region) in input["regionBits"]
                .as_array()
                .ok_or("region inputs")?
                .iter()
                .enumerate()
            {
                for (k, temperature) in input["temperatureBits"]
                    .as_array()
                    .ok_or("temperature inputs")?
                    .iter()
                    .enumerate()
                {
                    let cell = Cell {
                        terrain: terrain.id(),
                        biome_region_id: value(region),
                        region_temperature: value(temperature),
                        ..Cell::default()
                    };
                    writeln!(
                        out,
                        "select/{p}/{t}/{r}/{k}\t{}",
                        classic_biomes::select(parent, &cell, terrain)
                    )?;
                }
            }
        }
    }
    out.flush()?;
    eprintln!(
        "NATIVE_CLASSIC_BIOMES_OK exactRegionHash modernPolicy terrainIdentity eligibleFamilies nonfinitePassthrough"
    );
    Ok(())
}
