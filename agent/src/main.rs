mod command;
mod config;
mod error;
mod tunnel;
mod user;
use crate::config::get_agent_config;
use crate::error::Error;
use common::{build_server_runtime, init_log, start_server, ServerState};
use tokio::signal;
use tracing::{debug, error, info};
async fn handle_connection(server_state: ServerState) -> Result<(), Error> {
    debug!("Handling agent connection: {server_state:?}.");
    tunnel::process(server_state).await?;
    Ok(())
}
fn main() -> Result<(), Error> {
    let _log_guard = init_log(get_agent_config())?;
    let server_runtime = build_server_runtime(get_agent_config())?;
    server_runtime.block_on(async move {
        let server_guard = start_server(get_agent_config(), handle_connection);
        if let Err(e) = signal::ctrl_c().await {
            error!("Error happen when listening stop signal: {}", e);
            return;
        }
        info!("Receive stop signal, going to stop server.");
        server_guard.stop_signal.cancel();
    });
    Ok(())
}
