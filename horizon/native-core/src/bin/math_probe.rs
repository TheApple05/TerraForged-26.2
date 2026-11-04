use terraforged_native_core::noise_util as n;
fn bits(value: f32) -> String {
    format!("{:08x}", value.to_bits())
}

fn main() {
    let count = 4096;
    for i in 0..count {
        let center = (i as f32 + 0.5f32) / count as f32 * n::PI2;
        let angles = [
            center.next_down(),
            center,
            center.next_up(),
            -center.next_down(),
            -center,
            -center.next_up(),
        ];
        for (j, angle) in angles.iter().enumerate() {
            println!(
                "trig/{i}/{j}\t{},{},{}",
                bits(*angle),
                bits(n::sin(*angle)),
                bits(n::cos(*angle))
            );
        }
        let input = i as f32 / (count - 1) as f32;
        println!("sqrt/{i}\t{},{}", bits(input), bits(n::sqrt(input)));
    }
    for (i, raw) in [
        0u32, 0x80000000, 1, 0x007fffff, 0x00800000, 0x3f800000, 0x40000000, 0x7f7fffff,
    ]
    .iter()
    .enumerate()
    {
        let input = f32::from_bits(*raw);
        println!(
            "extra/{i}\t{},{},{},{}",
            bits(input),
            bits(n::sin(input)),
            bits(n::cos(input)),
            bits(n::sqrt(input))
        );
    }
}
