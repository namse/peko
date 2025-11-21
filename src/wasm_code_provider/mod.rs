pub mod fs;
pub mod s3;

use std::future::Future;
use wasmtime::{
    Engine,
    component::{InstancePre, Linker},
};

pub type Result<T> = std::result::Result<T, Error>;
pub trait WasmCodeProvider: Clone + Send + Sync + 'static {
    fn get_instance_pre(
        &self,
        id: &str,
        engine: &Engine,
        linker: &Linker<()>,
    ) -> impl Future<Output = Result<InstancePre<()>>> + Send;
}

#[derive(Debug)]
pub enum Error {
    NotFound,
    IoError { error: std::io::Error },
    ProviderError(anyhow::Error),
    WasmTime(wasmtime::Error),
}
impl From<wasmtime::Error> for Error {
    fn from(value: wasmtime::Error) -> Self {
        Self::WasmTime(value)
    }
}
