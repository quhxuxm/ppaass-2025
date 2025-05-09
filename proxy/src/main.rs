use crate::config::get_proxy_config;
use crate::error::ProxyError;
use crate::user::get_forward_user_repo;
use ppaass_2025_common::{build_server_runtime, init_log, Server, ServerState};
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
async fn handle_agent_connection(server_state: ServerState) -> Result<(), ProxyError> {
    tunnel::process(server_state, get_forward_user_repo()).await?;
    Ok(())
}
/// Start the proxy server
fn main() -> Result<(), ProxyError> {
    let _log_guard = init_log(get_proxy_config())?;
    let server_runtime = build_server_runtime(get_proxy_config())?;
    server_runtime.block_on(async move {
        let server = Server::new(get_proxy_config());
        let server_guard = server.start(handle_agent_connection);
        if let Err(e) = signal::ctrl_c().await {
            error!("Failed to listen to shutdown signal: {}", e);
        }
        info!("Begin to stop Ppaass proxy.");
        server_guard.stop_single.cancel();
    });
    Ok(())
}
