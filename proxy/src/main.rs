use crate::command::ProxyCommandArgs;
use crate::config::ProxyConfig;
use crate::error::ProxyError;
use crate::user::ProxyUserInfo;
use clap::Parser;
use ppaass_2025_common::{generate_base_runtime, init_log, BaseServer, BaseServerState};
use ppaass_2025_user::{FileSystemUserRepository, UserRepository};
use std::fs::read_to_string;
use std::sync::Arc;
use tokio::signal;
use tracing::{debug, error, info};
pub(crate) mod client;
mod command;
mod config;
pub(crate) mod destination;
mod error;
mod tunnel;
mod user;

/// Handle the incoming client connection
async fn handle_connection(
    base_server_state: BaseServerState<
        ProxyConfig,
        FileSystemUserRepository<ProxyUserInfo, ProxyConfig>,
    >,
) -> Result<(), ProxyError> {
    debug!(
        "Handle connection: {:?}, user_repository: {:?}",
        base_server_state.client_addr, base_server_state.user_repository
    );
    tunnel::process(base_server_state).await?;
    Ok(())
}

/// Start the proxy server
fn main() -> Result<(), ProxyError> {
    let command_line = ProxyCommandArgs::parse();
    let proxy_config_content = match &command_line.config_file_path {
        None => read_to_string(config::DEFAULT_PROXY_CONFIG_FILE).expect(&format!(
            "Fail to read proxy configuration file content from: {:?}",
            config::DEFAULT_PROXY_CONFIG_FILE
        )),
        Some(path) => read_to_string(path).expect(&format!(
            "Fail to read proxy configuration file content from: {:?}",
            path
        )),
    };
    let mut proxy_config = toml::from_str::<ProxyConfig>(&proxy_config_content)
        .expect("Fail to initialize proxy configuration");
    proxy_config.merge_command_args(command_line);
    let proxy_config = Arc::new(proxy_config);
    let _log_guard = init_log(&*proxy_config)?;
    let proxy_runtime = generate_base_runtime(&*proxy_config)?;
    proxy_runtime.block_on(async move {
        let user_repository =
            match FileSystemUserRepository::<ProxyUserInfo, ProxyConfig>::new(proxy_config.clone())
                .await
            {
                Ok(user_repository) => user_repository,
                Err(e) => {
                    error!("Fail to create user repository from file system: {e:?}");
                    return;
                }
            };
        let user_repository = Arc::new(user_repository);
        let base_server = BaseServer::new(proxy_config, user_repository);
        let base_server_guard = base_server.start(handle_connection);
        if let Err(e) = signal::ctrl_c().await {
            error!("Failed to listen to shutdown signal: {}", e);
        }
        info!("Begin to stop Ppaass proxy.");
        base_server_guard.stop_single.cancel();
    });
    Ok(())
}
