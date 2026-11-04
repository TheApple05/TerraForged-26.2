// MIT ReTerraForged NoiseUtil math constants, baseline29239f1f164d19dbfcccaca4a8277d64e784207c.
// The 4096-entry table is source-owned numerical lookup data, not game assets
// or terrain samples. See the immutable independent math-v1 metadata and license.
const SINE: &[u8; 16384] = include_bytes!("data/noiseutil-sine.bin");
const RAD_TO_INDEX: f32 = f32::from_bits(0x4422f983);
pub const PI2: f32 = f32::from_bits(0x40c90fdb);
pub const SQRT2: f32 = f32::from_bits(0x3fb504f3);

#[inline]
pub fn sin(radians: f32) -> f32 {
    let index = ((radians * RAD_TO_INDEX) as i32 & 4095) as usize;
    let offset = index * 4;
    f32::from_bits(u32::from_be_bytes([
        SINE[offset],
        SINE[offset + 1],
        SINE[offset + 2],
        SINE[offset + 3],
    ]))
}

#[inline]
pub fn cos(radians: f32) -> f32 {
    sin(radians + 1.5708f32)
}

#[inline]
pub fn sqrt(value: f32) -> f32 {
    (value as f64).sqrt() as f32
}
