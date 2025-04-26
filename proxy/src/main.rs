use crate::config::ProxyConfig;
use crate::error::ProxyError;
use ppaass_2025_core::{generate_core_runtime, init_log, CoreError, CoreServer, CoreServerState};
use std::fs::read_to_string;
use std::sync::Arc;
use tokio::signal;
use tracing::{debug, error, info};
mod config;
mod error;
mod user;
const PROXY_CONFIG_FILE: &str = "resource/proxy.toml";
async fn handle_connection(core_server_state: CoreServerState<ProxyConfig>) -> Result<(), CoreError> {
    debug!("Handle connection: {:?}", core_server_state.client_addr);
    Ok(())
}
fn main() -> Result<(), ProxyError> {
    let proxy_config_content = read_to_string(PROXY_CONFIG_FILE)?;
    let proxy_config = Arc::new(toml::from_str::<ProxyConfig>(&proxy_config_content)?);
    let _log_guard = init_log(proxy_config.as_ref())?;
    let proxy_runtime = generate_core_runtime(proxy_config.as_ref())?;
    let _runtime_guard = proxy_runtime.enter();
    let core_server = CoreServer::new(proxy_config);
    let server_guard = core_server.start(handle_connection);
    proxy_runtime.block_on(async {
        if let Err(e) = signal::ctrl_c().await {
            error!("Failed to listen to shutdown signal: {}", e);
        }
        info!("Begin to stop Ppaass proxy.");
        server_guard.stop_single.cancel();
    });
    Ok(())
}
