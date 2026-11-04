use terraforged_native_core::noise_util::pow_f32;

fn hex(v: f32) -> String {
    format!("{:08x}", v.to_bits())
}

fn emit_power(bi: usize, ei: usize, base: f32, exp: f32) {
    let r = pow_f32(base, exp);
    println!("power/{}/{}\t{}", bi, ei, hex(r));
}

fn emit_negative(bi: usize, ei: usize, base: f32, exp: f32) {
    let r = pow_f32(base, exp);
    println!("negative/{}/{}\t{}", bi, ei, hex(r));
}

fn main() {
    let base_bits: [u32; 12] = [
        0x00000000, 0x80000000, 0x00000001, 0x00800000, 0x3a83126f, 0x3e800000, 0x3f000000,
        0x3f800000, 0x3fc00000, 0x40000000, 0x41200000, 0x7f7fffff,
    ];
    let exp_bits: [u32; 15] = [
        0xc0400000, 0xc0000000, 0xbf800000, 0xbf000000, 0x80000000, 0x00000000, 0x3e000000,
        0x3f000000, 0x3f400000, 0x3f800000, 0x3fa00000, 0x3fc00000, 0x40000000, 0x40400000,
        0x41200000,
    ];

    for bi in 0..base_bits.len() {
        let base = f32::from_bits(base_bits[bi]);
        for ei in 0..exp_bits.len() {
            let exp = f32::from_bits(exp_bits[ei]);
            emit_power(bi, ei, base, exp);
        }
    }

    let neg_bits: [u32; 4] = [0xbe800000, 0xbf800000, 0xc0000000, 0xc1200000];
    let neg_exp_bits: [u32; 9] = [
        0xc0400000, 0xc0000000, 0xbf800000, 0x80000000, 0x00000000, 0x3f800000, 0x40000000,
        0x40400000, 0x41200000,
    ];

    for bi in 0..neg_bits.len() {
        let base = f32::from_bits(neg_bits[bi]);
        for ei in 0..neg_exp_bits.len() {
            let exp = f32::from_bits(neg_exp_bits[ei]);
            emit_negative(bi, ei, base, exp);
        }
    }

    let lac_bits: [u32; 5] = [0x3f8ccccd, 0x3fc00000, 0x40000000, 0x40200000, 0x40400000];

    for li in 0..lac_bits.len() {
        let lacunarity = f32::from_bits(lac_bits[li]);
        let mut frequency: f32 = 1.0;
        for step in 0..32 {
            let r = pow_f32(frequency, -1.0f32);
            println!("spectral/{}/{}\t{}", li, step, hex(r));
            frequency *= lacunarity;
        }
    }
}
