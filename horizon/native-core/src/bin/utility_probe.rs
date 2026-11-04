use std::fmt::Write;
use terraforged_native_core::{fast_random::FastRandom, pos_util as p, seed::Seed};

fn bits(value: f32) -> String {
    format!("{:08x}", value.to_bits())
}

fn main() {
    let ints = [
        i32::MIN,
        -30_000_000,
        -17,
        -16,
        -1,
        0,
        1,
        15,
        16,
        30_000_000,
        i32::MAX,
    ];
    for x in ints {
        for z in ints {
            let packed = p::pack(x, z);
            assert_eq!(p::unpack_left(packed), x);
            assert_eq!(p::unpack_right(packed), z);
            println!("pack/{x}/{z}\t{packed}");
        }
    }
    let coords: [f32; 17] = [
        -30_000_000.0,
        -17.0,
        -16.25,
        -16.0,
        -1.5,
        -1.0,
        -0.25,
        -0.0,
        0.0,
        0.25,
        1.0,
        15.0,
        15.75,
        16.0,
        16.25,
        17.0,
        30_000_000.0,
    ];
    for (i, x) in coords.iter().enumerate() {
        for (j, z) in coords.iter().enumerate() {
            let packed = p::pack_floats(*x, *z);
            assert_eq!(p::unpack_left_float(packed).to_bits(), x.to_bits());
            assert_eq!(p::unpack_right_float(packed).to_bits(), z.to_bits());
            let mix_a = p::pack_int_float(i as i32 - 17, *z);
            let mix_b = p::pack_float_int(*x, j as i32 - 17);
            assert_eq!(p::unpack_left(mix_a), i as i32 - 17);
            assert_eq!(p::unpack_right(mix_b), j as i32 - 17);
            println!("packf/{i}/{j}\t{packed},{mix_a},{mix_b}");
        }
    }
    assert!(p::contains_i32(-16, -16, -16, -16, 0, 0));
    assert!(!p::contains_i32(0, -1, -16, -16, 0, 0));
    assert!(p::contains_f32(-0.5, -0.5, -1.0, -1.0, 0.0, 0.0));
    for seed in [0, 1, -1, 42, i32::MIN, i32::MAX] {
        let mut sequence = Seed::new(seed);
        let mut values = String::new();
        for _ in 0..16 {
            write!(values, "{},", sequence.next_seed()).unwrap();
        }
        assert_eq!(sequence.root(), seed);
        assert_eq!(sequence.split().get(), seed);
        assert_eq!(sequence.offset(17).get(), seed.wrapping_add(17));
        println!("seed/{seed}\t{values}/{}", sequence.get());
        let mut random = FastRandom::new(seed as i64);
        let mut repeated = FastRandom::new(seed as i64);
        let mut values = String::new();
        for _ in 0..64 {
            let value = random.next_int();
            assert_eq!(value, repeated.next_int());
            write!(values, "{value},").unwrap();
        }
        for bound in [1, 2, 3, 31, 32, 1024, i32::MAX] {
            for _ in 0..16 {
                let value = random.next_int_bound(bound).unwrap();
                assert!(value >= 0 && value < bound);
                write!(values, "{value},").unwrap();
            }
        }
        for _ in 0..32 {
            let value = random.next_float();
            assert!((0.0..1.0).contains(&value));
            write!(values, "{},{},", bits(value), random.next_boolean()).unwrap();
        }
        random
            .seed_with_gamma(seed as i64, 19)
            .set_gamma(-7046029254386353131);
        write!(values, "{}", random.next_int()).unwrap();
        println!("random/{seed}\t{values}");
    }
}
