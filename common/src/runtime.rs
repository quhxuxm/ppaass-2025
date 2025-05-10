use crate::Error;
use crate::config::ServerRuntimeConfig;
use tokio::runtime::{Builder, Runtime};
/// Generate the server runtime.
/// * config: The server configuration
pub fn build_server_runtime<C: ServerRuntimeConfig>(config: &C) -> Result<Runtime, Error> {
    Builder::new_multi_thread()
        .worker_threads(config.worker_threads())
        .enable_all()
        .build()
        .map_err(Into::into)
}
