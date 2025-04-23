use crate::config::ProxyConfig;
use crate::error::ProxyError;
use ppaass_2025_core::{generate_core_runtime, CoreError, CoreServer, CoreServerState};
use std::fs::read_to_string;
use std::sync::Arc;
use tokio::signal;
use tracing::{error, info};
mod config;
mod error;
mod user;
const PROXY_CONFIG_FILE: &str = "proxy.toml";
async fn handle_connection(core_server_state: CoreServerState<ProxyConfig>) -> Result<(), CoreError> {
    Ok(())
}
fn main() -> Result<(), ProxyError> {
    let proxy_config_content = read_to_string(PROXY_CONFIG_FILE)?;
    let proxy_config = Arc::new(toml::from_str::<ProxyConfig>(&proxy_config_content)?);
    let proxy_runtime = generate_core_runtime(proxy_config.as_ref())?;
    let _runtime_guard = proxy_runtime.enter();
    let core_server = CoreServer::new(proxy_config);
    let _server_guard = core_server.start(handle_connection);
    proxy_runtime.block_on(async {
        if let Err(e) = signal::ctrl_c().await {
            error!("Failed to listen to shutdown signal: {}", e);
        }
        info!("Ppaass proxy shutdown.");
    });
    Ok(())
}
