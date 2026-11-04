// Copyright (c) 2026 TerraForged Revival contributors. MIT.
fn terraforged_open(
    data: &std::sync::Arc<GeneratorData>,
    seed: i64,
    dimension: &str,
) -> std::ops::ControlFlow<Result<OverworldGenerator, GeneratorError>> {
    if dimension != "minecraft:overworld" {
        return std::ops::ControlFlow::Continue(());
    }
    let Some(directory) = std::env::var_os("TERRAFORGED_HORIZON_PREPARED") else {
        return std::ops::ControlFlow::Continue(());
    };
    std::ops::ControlFlow::Break(terraforged_prepared::open(
        data.clone(),
        seed,
        std::path::Path::new(&directory),
    ))
}
