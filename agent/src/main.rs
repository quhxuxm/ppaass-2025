mod command;
mod config;
mod error;
mod tunnel;
mod user;
use crate::config::get_agent_config;
use crate::error::AgentError;
use ppaass_2025_common::{build_server_runtime, init_log, Server, ServerState};
use tokio::signal;
use tracing::{error, info};
async fn handle_connection(server_state: ServerState) -> Result<(), AgentError> {
    tunnel::process(server_state).await?;
    Ok(())
}
fn main() -> Result<(), AgentError> {
    let _log_guard = init_log(get_agent_config())?;
    let server_runtime = build_server_runtime(get_agent_config())?;
    server_runtime.block_on(async move {
        let server = Server::new(get_agent_config());
        let server_guard = server.start(handle_connection);
        if let Err(e) = signal::ctrl_c().await {
            error!("Failed to listen to shutdown signal: {}", e);
        }
        info!("Begin to stop Ppaass proxy.");
        server_guard.stop_single.cancel();
    });
    Ok(())
}
