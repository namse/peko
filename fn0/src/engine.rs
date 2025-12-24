use wasmtime::{component::Linker, *};
use wasmtime_wasi::WasiView;
use wasmtime_wasi_http::WasiHttpView;

pub fn new_engine() -> Result<Engine> {
    let engine = Engine::new(&engine_config())?;
    Ok(engine)
}

pub fn new_linker<T: WasiView + WasiHttpView>(engine: &Engine) -> Result<Linker<T>> {
    let mut linker = Linker::new(engine);
    wasmtime_wasi::p2::add_to_linker_async(&mut linker)?;
    wasmtime_wasi_http::add_only_http_to_linker_async(&mut linker)?;
    Ok(linker)
}

fn engine_config() -> Config {
    const MB: usize = 1024 * 1024;

    let mut sys = sysinfo::System::new_all();
    sys.refresh_all();

    let total_memory_bytes = sys.total_memory() as usize;
    let total_memory_mb = total_memory_bytes / (1024 * 1024);
    const MAX_MEMORY_MB: usize = 128;
    let max_instance_count = total_memory_mb / MAX_MEMORY_MB;

    let mut pooling_allocation_config = PoolingAllocationConfig::new();
    pooling_allocation_config
        .max_memory_size(MB * MAX_MEMORY_MB)
        .linear_memory_keep_resident(MB * 16)
        .table_keep_resident(MB)
        .total_core_instances(max_instance_count as _)
        .total_memories(max_instance_count as _)
        .total_tables(max_instance_count as _)
        .pagemap_scan(wasmtime::Enabled::Auto)
        .memory_protection_keys(wasmtime::Enabled::Auto);

    let mut config = Config::new();

    config
        .async_support(true)
        .allocation_strategy(InstanceAllocationStrategy::Pooling(
            pooling_allocation_config,
        ))
        .epoch_interruption(true)
        .wasm_component_model(true)
        .cranelift_opt_level(wasmtime::OptLevel::None)
        .cache(Some(
            wasmtime::Cache::new(wasmtime::CacheConfig::new()).unwrap(),
        ))
        .parallel_compilation(true)
        .debug_info(true);

    config
}
