mod command;
mod config;
mod error;
mod proxy;
mod tunnel;
mod user;
use crate::command::AgentCommandArgs;
use crate::config::AgentConfig;
use crate::error::AgentError;
use crate::user::AgentUserInfo;
use clap::Parser;
use ppaass_2025_common::{generate_core_runtime, init_log, BaseServer, BaseServerState};
use ppaass_2025_user::{FileSystemUserRepository, UserRepository};
use std::fs::read_to_string;
use std::sync::Arc;
use tokio::signal;
use tracing::{debug, error, info};
async fn handle_connection(
    base_server_state: BaseServerState<
        AgentConfig,
        FileSystemUserRepository<AgentUserInfo, AgentConfig>,
    >,
) -> Result<(), AgentError> {
    debug!(
        "Handle connection: {:?}, user_repository: {:?}",
        base_server_state.client_addr, base_server_state.user_repository
    );
    tunnel::process(base_server_state).await?;
    Ok(())
}
fn main() -> Result<(), AgentError> {
    let command_line = AgentCommandArgs::parse();
    let agent_config_content = read_to_string(config::AGENT_CONFIG_FILE)
        .expect("Fail to read agent configuration file content");
    let mut agent_config = toml::from_str::<AgentConfig>(&agent_config_content)
        .expect("Fail to initialize agent configuration");
    agent_config.merge_command_args(command_line);
    let agent_config = Arc::new(agent_config);
    let _log_guard = init_log(&*agent_config)?;
    let agent_runtime = generate_core_runtime(&*agent_config)?;
    agent_runtime.block_on(async move {
        let user_repository =
            match FileSystemUserRepository::<AgentUserInfo, AgentConfig>::new(agent_config.clone())
                .await
            {
                Ok(user_repository) => user_repository,
                Err(e) => {
                    error!("Fail to create user repository from file system: {e:?}");
                    return;
                }
            };
        let user_repository = Arc::new(user_repository);
        let base_server = BaseServer::new(agent_config, user_repository);
        let base_server_guard = base_server.start(handle_connection);
        if let Err(e) = signal::ctrl_c().await {
            error!("Failed to listen to shutdown signal: {}", e);
        }
        info!("Begin to stop Ppaass proxy.");
        base_server_guard.stop_single.cancel();
    });
    Ok(())
}
