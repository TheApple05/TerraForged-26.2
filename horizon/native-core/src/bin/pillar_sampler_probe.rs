use std::{collections::HashMap, error::Error, fs};
use terraforged_native_core::pillar_sampler::PillarSampler;
fn main() -> Result<(), Box<dyn Error>> {
    let input = fs::read_to_string(std::env::args_os().nth(1).ok_or("input TSV required")?)?;
    let mut cache = HashMap::new();
    for line in input.lines() {
        let f: Vec<_> = line.split('\t').collect();
        let result = if f[1] == "index" {
            PillarSampler::palette_index(f32::from_bits(u32::from_str_radix(f[2], 16)?)).to_string()
        } else if f[1] == "stone" {
            PillarSampler::stone_material(f[2].parse()?, f[3].parse()?)
                .id()
                .into()
        } else {
            let seed: i32 = f[2].parse()?;
            if let std::collections::hash_map::Entry::Vacant(entry) = cache.entry(seed) {
                entry.insert(PillarSampler::new(seed)?);
            }
            let sampler = &cache[&seed];
            match f[1] {
                "ends" => sampler
                    .palette(f[3].parse()?)
                    .ok_or("palette index")?
                    .ends()
                    .map(|n| format!("{:08x}", n.to_bits()))
                    .join(","),
                "material" => sampler
                    .palette(f[3].parse()?)
                    .ok_or("palette index")?
                    .at(f[4].parse()?, f[5].parse()?)?
                    .id()
                    .into(),
                "shape" => format!(
                    "{:08x}",
                    sampler
                        .shape(f[3].parse()?, f[4].parse()?, f[5] == "1")
                        .to_bits()
                ),
                "bands" => format!(
                    "{:08x}",
                    sampler.band_value(f[3].parse()?, f[4].parse()?).to_bits()
                ),
                "shift" => sampler.band_shift(f[3].parse()?, f[4].parse()?).to_string(),
                _ => return Err("unknown operation".into()),
            }
        };
        println!("{}\t{}", f[0], result);
    }
    Ok(())
}
