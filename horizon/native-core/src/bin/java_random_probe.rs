use terraforged_native_core::java_random::JavaRandom;
fn main() {
    let seeds = [
        0,
        1,
        -1,
        i64::MIN,
        i64::MAX,
        i32::MIN as i64,
        i32::MAX as i64,
        8675309,
        8675309 + (1i64 << 48),
    ];
    let bounds = [1, 2, 3, 50, 100, 1073741824, 1073741825, i32::MAX];
    for seed in seeds {
        let mut r = JavaRandom::new(seed);
        for step in 0..256 {
            let mut fields = vec![
                r.next_i32().to_string(),
                r.next_bool().to_string(),
                format!("{:08x}", r.next_f32().to_bits()),
            ];
            for b in bounds {
                fields.push(r.bounded_i32(b).unwrap().to_string());
            }
            fields.push(r.next_i32().to_string());
            println!("trace/{seed}/{step}\t{}", fields.join(","));
        }
        for size in [0, 1, 2, 5, 32, 257, 4096] {
            let mut values: Vec<i32> = (0..size).collect();
            let mut shuffle = JavaRandom::new(seed);
            shuffle.shuffle(&mut values).unwrap();
            println!(
                "shuffle/{seed}/{size}\t{}",
                values
                    .iter()
                    .map(i32::to_string)
                    .collect::<Vec<_>>()
                    .join(",")
            );
            println!("shuffle-next/{seed}/{size}\t{}", shuffle.next_i32());
        }
        let mut r = JavaRandom::new(!seed);
        for _ in 0..32 {
            r.next_i32();
        }
        r.set_seed(seed);
        for i in 0..32 {
            println!("reset/{seed}/{i}\t{}", r.next_i32());
        }
    }
    for bound in [0, -1, i32::MIN] {
        assert!(JavaRandom::new(0).bounded_i32(bound).is_err());
    }
    eprintln!("NATIVE_JAVA_RANDOM_GUARDS_OK positiveBound");
}
