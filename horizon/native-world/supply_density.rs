fn terraforged_supply_density(
    loader: &mut Loader,
    name: &str,
) -> std::ops::ControlFlow<Result<FunctionId, DecodeError>> {
    let Some(state) = super::generator::terraforged_registry::active() else {
        return std::ops::ControlFlow::Continue(());
    };
    if loader.graph.function(name).is_some() {
        return std::ops::ControlFlow::Continue(());
    }
    if let Some(json) = state.density_definitions.get(name) {
        let raw: Box<RawValue> = match serde_json::from_str(json) {
            Ok(raw) => raw,
            Err(error) => return std::ops::ControlFlow::Break(Err(error.into())),
        };
        loader.definitions.insert(name.to_owned(), raw);
    }
    std::ops::ControlFlow::Continue(())
}
