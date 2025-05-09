use crate::config::get_proxy_config;
use crate::error::ProxyError;
use crate::user::get_forward_user_repo;
use ppaass_2025_common::{generate_base_runtime, init_log, BaseServer, BaseServerState};
use tokio::signal;
use tracing::{error, info};
pub(crate) mod client;
mod command;
mod config;
pub(crate) mod destination;
mod error;
mod tunnel;
mod user;

/// Handle the incoming client connection
async fn handle_agent_connection(server_state: BaseServerState) -> Result<(), ProxyError> {
    tunnel::process(server_state, get_forward_user_repo()).await?;
    Ok(())
}
/// Start the proxy server
fn main() -> Result<(), ProxyError> {
    let _log_guard = init_log(get_proxy_config())?;
    let proxy_runtime = generate_base_runtime(get_proxy_config())?;
    proxy_runtime.block_on(async move {
        let base_server = BaseServer::new(get_proxy_config());
        let base_server_guard = base_server.start(handle_agent_connection);
        if let Err(e) = signal::ctrl_c().await {
            error!("Failed to listen to shutdown signal: {}", e);
        }
        info!("Begin to stop Ppaass proxy.");
        base_server_guard.stop_single.cancel();
    });
    Ok(())
}
