use terraforged_native_adapter::noise_registry::NoiseRegistry;
fn bits(f: f32) -> String {
    if f.is_nan() {
        "NaN".into()
    } else if f == f32::INFINITY {
        "+Infinity".into()
    } else if f == f32::NEG_INFINITY {
        "-Infinity".into()
    } else {
        format!("{:08x}", f.to_bits())
    }
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file = std::env::args().nth(1).ok_or("Input JSON file required")?;
    let input = std::fs::read_to_string(file)?;
    let mut documents: serde_json::Value = serde_json::from_str(&input)?;
    for id in [
        "qa:linear_spline",
        "qa:linear_spline_numeric",
        "qa:linear_spline_empty",
    ] {
        let value = documents
            .as_object_mut()
            .ok_or("Expected input dictionary")?
            .remove(id)
            .ok_or("Expected explicit source rejection case")?;
        let mut isolated = serde_json::Map::new();
        isolated.insert(id.to_owned(), value);
        assert!(
            NoiseRegistry::from_json(&serde_json::Value::Object(isolated).to_string()).is_err()
        );
        println!("reject/{id}\ttrue");
    }
    let registry = NoiseRegistry::from_json(&documents.to_string())?;
    let seeds = [0, 1, -1, 42, i32::MIN, i32::MAX];
    let coords: [f32; 17] = [
        -30_000_000.,
        -17.,
        -16.25,
        -16.,
        -1.5,
        -1.,
        -0.25,
        -0.,
        0.,
        0.25,
        1.,
        15.,
        15.75,
        16.,
        16.25,
        17.,
        30_000_000.,
    ];
    for (id, n) in registry.iter() {
        println!(
            "graph/{id}\t{},{},{}",
            n.kind(),
            bits(n.min_value()),
            bits(n.max_value())
        );
        for seed in seeds {
            for (i, x) in coords.into_iter().enumerate() {
                for (j, z) in coords.into_iter().enumerate() {
                    println!(
                        "sample/{id}/{seed}/{i}/{j}\t{}",
                        bits(n.compute(x, z, seed))
                    );
                }
            }
        }
    }
    let rejects = [
        (
            "missing-reference",
            r#"{"qa:a":{"type":"reterraforged:add","input1":"qa:missing","input2":0}}"#,
        ),
        (
            "cycle",
            r#"{"qa:a":{"type":"reterraforged:add","input1":"qa:b","input2":0},"qa:b":{"type":"reterraforged:add","input1":"qa:a","input2":0}}"#,
        ),
        ("unknown-type", r#"{"qa:a":{"type":"qa:unknown"}}"#),
        (
            "missing-field",
            r#"{"qa:a":{"type":"reterraforged:sin","frequency":1}}"#,
        ),
        (
            "unknown-enum",
            r#"{"qa:a":{"type":"reterraforged:worley_edge","frequency":1,"distance":1,"edge_function":"MISSING","distance_function":"EUCLIDEAN"}}"#,
        ),
        ("invalid-id", r#"{"QA:A":0}"#),
        ("constant-limit", r#"{"qa:a":1000001}"#),
        (
            "integer-limit",
            r#"{"qa:a":{"type":"reterraforged:shift","input":0,"shift":2147483648}}"#,
        ),
        (
            "power-limit",
            r#"{"qa:a":{"type":"reterraforged:legacy_temperature","frequency":1,"power":1025}}"#,
        ),
        (
            "octave-limit",
            r#"{"qa:a":{"type":"reterraforged:perlin","seed":0,"frequency":1,"octaves":257,"lacunarity":2,"gain":0.5,"interpolation":"CURVE3"}}"#,
        ),
    ];
    for (name, text) in rejects {
        assert!(
            NoiseRegistry::from_json(text).is_err(),
            "Missing reject: {name}"
        );
    }
    let mut nested = serde_json::json!(0);
    for _ in 0..40 {
        nested = serde_json::json!({"type":"reterraforged:abs","input":nested});
    }
    assert!(NoiseRegistry::from_json(&serde_json::json!({"qa:a":nested}).to_string()).is_err());
    assert!(
        NoiseRegistry::from_json(r#"{"qa:a":0}"#)?
            .get("qa:missing")
            .is_err()
    );
    let zeros = NoiseRegistry::from_json(
        r#"{"qa:-0":-0,"qa:encoded":"-0","qa:ref":{"type":"reterraforged:add","input1":"qa:-0","input2":-0}}"#,
    )?;
    for id in ["qa:-0", "qa:encoded", "qa:ref"] {
        assert_eq!(zeros.get(id)?.compute(0.0, 0.0, 0).to_bits(), 0x80000000);
    }
    eprintln!(
        "NATIVE_REGISTRY_GUARDS_OK missing cycle type field enum identifier numeric octave power depth lookup"
    );
    Ok(())
}
