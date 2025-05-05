mod http;
use crate::config::AgentConfig;
use crate::error::AgentError;
use crate::proxy::{Initial, ProxyConnection};
use crate::user::AgentUserInfo;
use ppaass_2025_common::BaseServerState;
use ppaass_2025_user::FileSystemUserRepository;
use tracing::debug;
const SOCKS4_VERSION_FLAG: u8 = 4;
const SOCKS5_VERSION_FLAG: u8 = 5;

pub async fn process(
    base_server_state: BaseServerState<
        AgentConfig,
        &AgentConfig,
        FileSystemUserRepository<AgentUserInfo, AgentConfig>,
    >,
) -> Result<(), AgentError> {
    let mut protocol_flag_buf = [0u8; 1];
    let flag_size = base_server_state
        .client_stream
        .peek(&mut protocol_flag_buf)
        .await?;
    if flag_size == 0 {
        return Ok(());
    }
    let protocol_flag = protocol_flag_buf[0];
    match protocol_flag {
        SOCKS4_VERSION_FLAG => {
            unimplemented!("Socks 4 protocol not supported")
        }
        SOCKS5_VERSION_FLAG => {
            debug!(
                "Accept socks 5 protocol client connection [{}].",
                base_server_state.client_addr
            );
        }
        _ => {
            debug!(
                "Accept http/https protocol client connection [{}].",
                base_server_state.client_addr
            );
            http::process_http_tunnel(base_server_state).await?;
        }
    }

    Ok(())
}

async fn build_proxy_connection(
    user_repository: &FileSystemUserRepository<AgentUserInfo, AgentConfig>,
) -> Result<ProxyConnection<Initial>, AgentError> {
    ProxyConnection::new(user_repository).await
}
