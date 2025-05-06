use crate::BaseError;
use crate::config::RuntimeConfig;
use tokio::runtime::{Builder, Runtime};
pub fn generate_base_runtime<C: RuntimeConfig>(config: &C) -> Result<Runtime, BaseError> {
    let runtime = Builder::new_multi_thread()
        .worker_threads(config.worker_threads())
        .enable_all()
        .build()?;
    Ok(runtime)
}
