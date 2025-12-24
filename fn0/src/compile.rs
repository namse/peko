/// input: wasm
/// output: cwasm
pub fn compile(wasm: &[u8]) -> anyhow::Result<Vec<u8>> {
    let engine = crate::engine::new_engine()?;
    let component = engine.precompile_component(wasm)?;
    Ok(component)
}
