fn terraforged_decode(
    loader: &mut Loader,
    raw: &RawValue,
    depth: usize,
) -> std::ops::ControlFlow<Result<FunctionId, DecodeError>> {
    use horizon_core::density::terraforged_runtime::{
        self as native, kernel::density_primitives::CellField,
    };
    let Some(state) = super::generator::terraforged_registry::active() else {
        return std::ops::ControlFlow::Continue(());
    };
    if !raw.get().starts_with('{') {
        return std::ops::ControlFlow::Continue(());
    }
    let object: Object = match serde_json::from_str(raw.get()) {
        Ok(object) => object,
        Err(_) => return std::ops::ControlFlow::Continue(()),
    };
    let kind = object.get("type").and_then(|v| string(v).ok());
    let Some(kind) = kind else {
        return std::ops::ControlFlow::Continue(());
    };
    if !matches!(
        kind.as_str(),
        "reterraforged:cell" | "reterraforged:noise" | "reterraforged:clamp_to_nearest_unit"
    ) {
        return std::ops::ControlFlow::Continue(());
    }
    let result = (|| {
        if depth > 256 {
            return Err(DecodeError::Field("density nesting"));
        }
        match kind.as_str() {
            "reterraforged:cell" => {
                let name = string(required(&object, "field")?)?;
                let field = CellField::parse(&name)
                    .map_err(|_| DecodeError::Field("TerraForged cell field"))?;
                Ok(native::bind_cell(
                    &mut loader.graph,
                    state.sampler.clone(),
                    field,
                )?)
            }
            "reterraforged:noise" => {
                // Named source holders only. Inline graphs remain an explicit unsupported input.
                let name = identifier(&string(required(&object, "noise")?)?)?;
                let noise = state
                    .noises
                    .get(&name)
                    .map_err(|e| DecodeError::Registry(e.to_string()))?;
                Ok(native::bind_noise(
                    &mut loader.graph,
                    noise,
                    state.world_seed as i32,
                )?)
            }
            "reterraforged:clamp_to_nearest_unit" => {
                let resolution: i32 = serde_json::from_str(required(&object, "resolution")?.get())?;
                let input = loader.input(&object, "function", depth)?;
                Ok(native::bind_clamp(
                    &mut loader.graph,
                    input,
                    resolution,
                    state.world_seed,
                    state.legacy_random,
                )?)
            }
            _ => unreachable!(),
        }
    })();
    std::ops::ControlFlow::Break(result)
}
