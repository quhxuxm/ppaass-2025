mod http;
mod socks5;

use crate::config::Config;
use crate::error::Error;
use crate::user::get_agent_user_repo;
use common::Error as CommonError;
use common::ServerState;
use common::config::WithUsernameConfig;
use common::proxy::Init;

use common::proxy::{ProxyConnection, ProxyFramed};
use common::user::UserRepository;
use tokio::io::AsyncWriteExt;
use tracing::{debug, error};
const SOCKS4_VERSION_FLAG: u8 = 4;
const SOCKS5_VERSION_FLAG: u8 = 5;

pub async fn process(mut server_state: ServerState) -> Result<(), Error> {
    let mut protocol_flag_buf = [0u8; 1];
    let flag_size = server_state
        .incoming_stream
        .peek(&mut protocol_flag_buf)
        .await?;
    if flag_size == 0 {
        return Ok(());
    }
    let protocol_flag = protocol_flag_buf[0];
    match protocol_flag {
        SOCKS4_VERSION_FLAG => {
            error!("Socks 4 protocol not supported");
            server_state.incoming_stream.shutdown().await?;
        }
        SOCKS5_VERSION_FLAG => {
            debug!(
                "Accept socks 5 protocol client connection [{}].",
                server_state.incoming_connection_addr
            );
            socks5::process_socks5_tunnel(server_state).await?;
        }
        _ => {
            debug!(
                "Accept http/https protocol client connection [{}].",
                server_state.incoming_connection_addr
            );
            http::process_http_tunnel(server_state).await?;
        }
    }

    Ok(())
}

/// Build a proxy connection
async fn build_proxy_connection<'a>(
    agent_config: &Config,
) -> Result<ProxyConnection<ProxyFramed<'a>>, Error> {
    let agent_user = get_agent_user_repo()
        .find_user(agent_config.username())
        .ok_or(CommonError::UserNotExist(
            agent_config.username().to_owned(),
        ))?;
    ProxyConnection::<Init>::new(agent_user, agent_config.proxy_connect_timeout())
        .await
        .map_err(Into::into)
}
