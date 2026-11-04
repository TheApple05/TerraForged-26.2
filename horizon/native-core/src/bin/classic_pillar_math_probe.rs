use std::{error::Error, fs};
use terraforged_native_core::classic_pillars::{added_height, mask, terrace};
fn f32_bits(text: &str) -> Result<f32, Box<dyn Error>> {
    Ok(f32::from_bits(u32::from_str_radix(text, 16)?))
}
fn bits(value: f32) -> String {
    if value.is_nan() {
        "nan".into()
    } else {
        format!("{:08x}", value.to_bits())
    }
}
fn main() -> Result<(), Box<dyn Error>> {
    let path = std::env::args_os().nth(1).ok_or("input TSV required")?;
    for line in fs::read_to_string(path)?.lines() {
        let fields: Vec<_> = line.split('\t').collect();
        let output = match fields[1] {
            "terrace" => bits(terrace(f32_bits(fields[2])?, f32_bits(fields[3])?)),
            "mask" => bits(mask(
                f32_bits(fields[2])?,
                f32_bits(fields[3])?,
                f32_bits(fields[4])?,
            )),
            "height" => added_height(f32_bits(fields[2])?, f32_bits(fields[3])?, fields[4] == "1")
                .to_string(),
            _ => return Err("unknown input operation".into()),
        };
        println!("{}\t{}", fields[0], output);
    }
    Ok(())
}
