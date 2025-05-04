mod config;
mod error;
mod proxy;
mod tunnel;
mod user;
use crate::config::{AgentConfig, AGENT_CONFIG};
use crate::error::AgentError;
use crate::user::AgentUserInfo;
use ppaass_2025_common::{generate_core_runtime, init_log, BaseServer, BaseServerState};
use ppaass_2025_user::{FileSystemUserRepository, UserRepository};
use std::ops::Deref;
use std::sync::Arc;
use tokio::signal;
use tracing::{debug, error, info};
async fn handle_connection(
    core_server_state: BaseServerState<
        AgentConfig,
        &AgentConfig,
        FileSystemUserRepository<AgentUserInfo, AgentConfig>,
    >,
) -> Result<(), AgentError> {
    debug!(
        "Handle connection: {:?}, user_repository: {:?}",
        core_server_state.client_addr, core_server_state.user_repository
    );
    tunnel::process(core_server_state).await?;
    Ok(())
}
fn main() -> Result<(), AgentError> {
    let _log_guard = init_log(AGENT_CONFIG.deref())?;
    let agent_runtime = generate_core_runtime(AGENT_CONFIG.deref())?;
    agent_runtime.block_on(async {
        let user_repository =
            match FileSystemUserRepository::<AgentUserInfo, AgentConfig>::new(AGENT_CONFIG.deref())
                .await
            {
                Ok(user_repository) => user_repository,
                Err(e) => {
                    error!("Fail to create user repository from file system: {e:?}");
                    return;
                }
            };
        let user_repository = Arc::new(user_repository);
        let core_server = BaseServer::new(AGENT_CONFIG.deref(), user_repository);
        let server_guard = core_server.start(handle_connection);
        if let Err(e) = signal::ctrl_c().await {
            error!("Failed to listen to shutdown signal: {}", e);
        }
        info!("Begin to stop Ppaass proxy.");
        server_guard.stop_single.cancel();
    });
    Ok(())
}
