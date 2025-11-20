pub mod fs;

pub enum Error {
    NotFound,
}

pub type Result<T> = std::result::Result<T, Error>;

pub trait WasmCodeProvider {
    async fn get_wasm_code(&self, id: &str) -> Result<Vec<u8>>;
}
