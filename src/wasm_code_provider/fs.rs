use super::*;
use std::path::PathBuf;
use wasmtime::component::Component;

#[derive(Clone)]
pub struct FsCodeProvider {
    base_path: PathBuf,
}

impl FsCodeProvider {
    pub fn new(base_path: PathBuf) -> Self {
        Self { base_path }
    }
}

impl WasmCodeProvider for FsCodeProvider {
    async fn get_instance_pre(
        &self,
        id: &str,
        engine: &Engine,
        linker: &Linker<()>,
    ) -> Result<InstancePre<()>> {
        let path = self.base_path.join(id);
        match tokio::fs::read(path).await {
            Ok(code) => {
                let component = Component::new(engine, code)?;
                Ok(linker.instantiate_pre(&component)?)
            }
            Err(error) => {
                if error.kind() == std::io::ErrorKind::NotFound {
                    return Err(Error::NotFound);
                }
                Err(Error::IoError { error })
            }
        }
    }
}
