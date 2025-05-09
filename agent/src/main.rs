mod command;
mod config;
mod error;
mod tunnel;
mod user;
use crate::config::get_agent_config;
use crate::error::AgentError;
use ppaass_2025_common::{generate_base_runtime, init_log, BaseServer, BaseServerState};
use tokio::signal;
use tracing::{error, info};
async fn handle_connection(server_state: BaseServerState) -> Result<(), AgentError> {
    tunnel::process(server_state).await?;
    Ok(())
}
fn main() -> Result<(), AgentError> {
    let _log_guard = init_log(get_agent_config())?;
    let agent_runtime = generate_base_runtime(get_agent_config())?;
    agent_runtime.block_on(async move {
        let base_server = BaseServer::new(get_agent_config());
        let base_server_guard = base_server.start(handle_connection);
        if let Err(e) = signal::ctrl_c().await {
            error!("Failed to listen to shutdown signal: {}", e);
        }
        info!("Begin to stop Ppaass proxy.");
        base_server_guard.stop_single.cancel();
    });
    Ok(())
}
