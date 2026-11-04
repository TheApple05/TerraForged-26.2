mod support;
use support::{cell, snapshot};
use terraforged_native_core::{
    graph::{GraphError, Noise as N},
    noises as n,
    perlin::Interpolation,
    populator::{Populator as P, TerrainPopulator as T},
    region::RegionModule,
};
fn terrain(id: usize, weight: f32) -> Result<P, GraphError> {
    let (base, height, erosion, weirdness, base_scale, height_scale) = match id {
        8 => (
            N::constant(0.25)?,
            n::simplex(111, 80, 3)?,
            0.445,
            0.34,
            0.75,
            0.1,
        ),
        11 => (
            N::constant(0.5)?,
            n::perlin(222, 120, 2)?,
            0.6,
            -0.2,
            1.25,
            0.3,
        ),
        10 => (N::constant(0.0)?, N::constant(-2.0)?, -0.1, -0.75, 1.0, 1.0),
        9 => (N::constant(0.5)?, N::constant(0.125)?, 0.7, -0.5, 1.0, 1.0),
        _ => unreachable!(),
    };
    P::terrain(T {
        terrain: id,
        base,
        height,
        erosion: N::constant(erosion)?,
        weirdness: N::constant(weirdness)?,
        base_scale,
        height_scale,
        weight,
    })
}
fn floats(root: &serde_json::Value, key: &str) -> Vec<f32> {
    root[key]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_f64().unwrap() as f32)
        .collect()
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(
        std::env::args().nth(1).ok_or("input required")?,
    )?)?;
    let coords = floats(&root, "coordinates");
    let edges = floats(&root, "edges");
    let ids = floats(&root, "identities");
    for seed in root["seeds"].as_array().unwrap() {
        for scale in root["regionScales"].as_array().unwrap() {
            let seed = seed.as_i64().unwrap() as i32;
            let scale = scale.as_i64().unwrap() as i32;
            let warp_seed = seed.wrapping_add(8934);
            let region = RegionModule::new(
                seed.wrapping_add(789124),
                scale,
                n::simplex(warp_seed, 400, 1)?,
                n::simplex(warp_seed.wrapping_add(1), 400, 1)?,
                200.0,
            )?;
            for (i, &x) in coords.iter().enumerate() {
                for (j, &z) in coords.iter().enumerate() {
                    for dirty in 0..2 {
                        let mut c = cell(dirty == 1);
                        region.apply(&mut c, x, z);
                        println!("region/{seed}/{scale}/{i}/{j}/{dirty}\t{}", snapshot(&c));
                    }
                }
            }
        }
    }
    let a = terrain(8, 1.0)?;
    let b = terrain(11, 2.5)?;
    let c = terrain(10, 0.0)?;
    let ocean = P::ocean(1, N::multiply(n::simplex2(333, 64, 2)?, N::constant(0.3)?)?)?;
    let mut graphs = vec![
        a.clone(),
        b.clone(),
        c.clone(),
        ocean.clone(),
        P::region_lerp(a.clone(), b.clone())?,
    ];
    for &edge in &edges {
        graphs.push(P::blend(
            N::constant(edge)?,
            a.clone(),
            b.clone(),
            0.3,
            0.8,
            0.575,
        )?);
    }
    graphs.push(P::blend(
        n::simplex(444, 80, 2)?,
        a.clone(),
        b.clone(),
        0.3,
        0.8,
        0.575,
    )?);
    for interpolation in [
        Interpolation::Linear,
        Interpolation::Curve3,
        Interpolation::Curve4,
    ] {
        graphs.push(P::continent2(
            a.clone(),
            b.clone(),
            0.3,
            0.8,
            interpolation,
        )?);
    }
    for interpolation in [
        Interpolation::Linear,
        Interpolation::Curve3,
        Interpolation::Curve4,
    ] {
        graphs.push(P::continent3(
            ocean.clone(),
            a.clone(),
            b.clone(),
            0.15,
            0.3,
            0.8,
            interpolation,
        )?);
    }
    let selector = P::selector(vec![a.clone(), b.clone(), c.clone(), ocean.clone()])?;
    graphs.push(selector.clone());
    graphs.push(P::selector(vec![c, terrain(9, 0.0)?])?);
    graphs.push(P::selector(vec![
        ocean.clone(),
        P::ocean(3, N::constant(0.25)?)?,
    ])?);
    graphs.push(P::region_lerp(
        ocean.clone(),
        P::blend(
            n::simplex(555, 64, 2)?,
            selector.clone(),
            b.clone(),
            0.3,
            0.8,
            0.575,
        )?,
    )?);
    assert_eq!(graphs.len(), 27);
    for (g, p) in graphs.iter().enumerate() {
        for (i, &x) in coords.iter().enumerate() {
            for (j, &z) in coords.iter().enumerate() {
                for (e, &edge) in edges.iter().enumerate() {
                    let mut c = cell(true);
                    c.continent_edge = edge;
                    c.terrain_region_edge = edge;
                    c.terrain_region_id = ids[(i + j + e) % ids.len()];
                    p.apply(&mut c, x, z)?;
                    println!("populator/{g}/{i}/{j}/{e}\t{}", snapshot(&c));
                }
            }
        }
    }
    assert!(P::selector(vec![]).is_err());
    assert!(P::selector(vec![terrain(8, -1.0)?]).is_err());
    assert!(P::selector(vec![terrain(8, 1e-20)?, b.clone()]).is_err());
    assert!(selector.selected(2.0).is_err());
    let mut deep = a.clone();
    let mut rejected = false;
    for _ in 0..256 {
        match P::region_lerp(deep.clone(), a.clone()) {
            Ok(next) => deep = next,
            Err(GraphError::DepthLimit) => {
                rejected = true;
                break;
            }
            Err(e) => return Err(e.into()),
        }
    }
    assert!(rejected);
    let mut wide = a;
    let mut rejected = false;
    for _ in 0..32 {
        match P::region_lerp(wide.clone(), wide.clone()) {
            Ok(next) => wide = next,
            Err(GraphError::NodeBudget) => {
                rejected = true;
                break;
            }
            Err(e) => return Err(e.into()),
        }
    }
    assert!(rejected);
    for p in &graphs {
        let first = {
            let mut c = cell(true);
            c.terrain_region_id = 0.5;
            c.terrain_region_edge = 0.5;
            p.apply(&mut c, 17.0, -17.0)?;
            snapshot(&c)
        };
        std::thread::scope(|s| {
            let handles: Vec<_> = (0..4)
                .map(|_| {
                    s.spawn(|| {
                        let mut c = cell(true);
                        c.terrain_region_id = 0.5;
                        c.terrain_region_edge = 0.5;
                        p.apply(&mut c, 17.0, -17.0).unwrap();
                        snapshot(&c)
                    })
                })
                .collect();
            for h in handles {
                assert_eq!(h.join().unwrap(), first);
            }
        });
    }
    eprintln!(
        "NATIVE_POPULATOR_GUARDS_OK weights selectorBounds depthBudget nodeBudget parallelOwnership"
    );
    Ok(())
}
