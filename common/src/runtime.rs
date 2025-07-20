use crate::{Error, WithServerConfig};

use tokio::runtime::{Builder, Runtime};
/// Generate the server runtime.
/// * config: The server configuration
pub fn build_server_runtime<C: WithServerConfig>(config: &C) -> Result<Runtime, Error> {
    Builder::new_multi_thread()
        .worker_threads(config.worker_threads())
        .enable_all()
        .build()
        .map_err(Into::into)
}
