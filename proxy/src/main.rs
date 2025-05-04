use crate::config::{ProxyConfig, PROXY_SERVER_CONFIG};
use crate::error::ProxyError;
use crate::user::ProxyUserInfo;
use ppaass_2025_common::{generate_core_runtime, init_log, CoreServer, CoreServerState};
use ppaass_2025_user::{FileSystemUserRepository, UserRepository};
use std::ops::Deref;
use std::sync::Arc;
use tokio::signal;
use tracing::{debug, error, info};
pub(crate) mod client;
mod config;
pub(crate) mod destination;
mod error;
mod tunnel;
mod user;
async fn handle_connection(
    core_server_state: CoreServerState<
        ProxyConfig,
        &ProxyConfig,
        FileSystemUserRepository<ProxyUserInfo, ProxyConfig>,
    >,
) -> Result<(), ProxyError> {
    debug!(
        "Handle connection: {:?}, user_repository: {:?}",
        core_server_state.client_addr, core_server_state.user_repository
    );
    tunnel::process(core_server_state).await?;
    Ok(())
}
fn main() -> Result<(), ProxyError> {
    let _log_guard = init_log(PROXY_SERVER_CONFIG.deref())?;
    let proxy_runtime = generate_core_runtime(PROXY_SERVER_CONFIG.deref())?;
    proxy_runtime.block_on(async {
        let user_repository = match FileSystemUserRepository::<ProxyUserInfo, ProxyConfig>::new(
            PROXY_SERVER_CONFIG.deref(),
        )
        .await
        {
            Ok(user_repository) => user_repository,
            Err(e) => {
                error!("Fail to create user repository from file system: {e:?}");
                return;
            }
        };
        let user_repository = Arc::new(user_repository);
        let core_server = CoreServer::new(PROXY_SERVER_CONFIG.deref(), user_repository);
        let server_guard = core_server.start(handle_connection);
        if let Err(e) = signal::ctrl_c().await {
            error!("Failed to listen to shutdown signal: {}", e);
        }
        info!("Begin to stop Ppaass proxy.");
        server_guard.stop_single.cancel();
    });
    Ok(())
}
