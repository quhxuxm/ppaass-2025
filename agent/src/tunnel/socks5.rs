use crate::config::AgentConfig;
use crate::error::AgentError;
use crate::proxy::ProxyConnectionDestinationType;
use crate::tunnel::build_proxy_connection;
use crate::user::AgentUserInfo;
use ppaass_2025_common::user::repo::FileSystemUserRepository;
use ppaass_2025_common::BaseServerState;
use ppaass_2025_protocol::UnifiedAddress;
use socks5_impl::protocol::handshake::Request as Socks5HandshakeRequest;
use socks5_impl::protocol::handshake::Response as Socks5HandshakeResponse;
use socks5_impl::protocol::{Address, AsyncStreamOperation, AuthMethod, Reply};
use socks5_impl::protocol::{
    Command as Socks5InitCommand, Request as Socks5InitRequest, Response as Socks5InitResponse,
};
use tracing::{debug, error, info};
pub async fn process_socks5_tunnel(
    mut base_server_state: BaseServerState<
        AgentConfig,
        FileSystemUserRepository<AgentUserInfo, AgentConfig>,
    >,
) -> Result<(), AgentError> {
    debug!(
        "Client connect to agent with socks 5 protocol: {}",
        base_server_state.client_addr
    );
    let auth_request =
        Socks5HandshakeRequest::retrieve_from_async_stream(&mut base_server_state.client_stream)
            .await?;
    debug!("Receive client socks5 handshake auth request: {auth_request:?}");
    let auth_response = Socks5HandshakeResponse::new(AuthMethod::NoAuth);
    auth_response
        .write_to_async_stream(&mut base_server_state.client_stream)
        .await?;
    let init_request =
        Socks5InitRequest::retrieve_from_async_stream(&mut base_server_state.client_stream).await?;
    debug!("Receive client socks5 handshake init request: {init_request:?}");

    match init_request.command {
        Socks5InitCommand::Connect => {
            debug!(
                "Receive socks5 CONNECT command: {}",
                base_server_state.client_addr
            );
            let proxy_connection = build_proxy_connection(
                base_server_state.config,
                &base_server_state.user_repository,
            )
            .await?;

            let destination_address = match &init_request.address {
                Address::SocketAddress(dst_addr) => dst_addr.into(),
                Address::DomainAddress(host, port) => UnifiedAddress::Domain {
                    host: host.clone(),
                    port: *port,
                },
            };
            let proxy_connection = proxy_connection.handshake().await?;
            let mut proxy_connection = proxy_connection
                .setup_destination(destination_address, ProxyConnectionDestinationType::Tcp)
                .await?;

            let init_response = Socks5InitResponse::new(Reply::Succeeded, init_request.address);
            init_response
                .write_to_async_stream(&mut base_server_state.client_stream)
                .await?;

            // Proxying data
            let (from_client, from_proxy) = match tokio::io::copy_bidirectional(
                &mut base_server_state.client_stream,
                &mut proxy_connection,
            )
            .await
            {
                Err(e) => {
                    error!("Fail to proxy data between agent and proxy: {e:?}");
                    return Ok(());
                }
                Ok((from_client, from_proxy)) => (from_client, from_proxy),
            };
            info!(
                "Agent wrote {} bytes to proxy, received {} bytes from proxy",
                from_client, from_proxy
            );
        }
        Socks5InitCommand::Bind => {
            unimplemented!(
                "Socks5 bind protocol not supported, client_addr: {}",
                base_server_state.client_addr
            )
        }
        Socks5InitCommand::UdpAssociate => {
            unimplemented!(
                "Socks5 udp associate protocol not supported, client_addr: {}",
                base_server_state.client_addr
            )
        }
    }
    Ok(())
}
