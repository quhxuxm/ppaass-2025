use crate::config::CoreRuntimeConfig;
use crate::CoreError;
use tokio::runtime::{Builder, Runtime};
pub fn generate_core_runtime<C: CoreRuntimeConfig>(config: &C) -> Result<Runtime, CoreError> {
    let runtime = Builder::new_multi_thread().worker_threads(config.worker_threads()).enable_all().build()?;
    Ok(runtime)
}