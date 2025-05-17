use crate::config::get_config;
use crate::error::Error;
use crate::tunnel::build_proxy_connection;
use common::proxy::ProxyConnectionDestinationType;
use common::ServerState;
use fast_socks5::server::Socks5ServerProtocol;
use fast_socks5::util::target_addr::TargetAddr;
use fast_socks5::Socks5Command;
use protocol::UnifiedAddress;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use tracing::{debug, error, info};
pub async fn process_socks5_tunnel(server_state: ServerState) -> Result<(), Error> {
    debug!(
        "Client connect to agent with socks 5 protocol: {}",
        server_state.incoming_connection_addr
    );

    let socks5_client_stream =
        Socks5ServerProtocol::accept_no_auth(server_state.incoming_stream).await?;
    let (socks5_client_stream, socks5_command, dst_addr) =
        socks5_client_stream.read_command().await?;

    match socks5_command {
        Socks5Command::TCPConnect => {
            debug!(
                "Receive socks5 CONNECT command: {}",
                server_state.incoming_connection_addr
            );
            let proxy_connection = build_proxy_connection(get_config()).await?;
            let destination_address = match &dst_addr {
                TargetAddr::Ip(dst_addr) => dst_addr.into(),
                TargetAddr::Domain(host, port) => UnifiedAddress::Domain {
                    host: host.clone(),
                    port: *port,
                },
            };
            let proxy_connection = proxy_connection.handshake().await?;
            let mut proxy_connection = proxy_connection
                .setup_destination(destination_address, ProxyConnectionDestinationType::Tcp)
                .await?;

            let mut socks5_client_stream = socks5_client_stream
                .reply_success(SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0)))
                .await?;

            // Proxying data
            let (from_client, from_proxy) = match tokio::io::copy_bidirectional(
                &mut socks5_client_stream,
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
        Socks5Command::TCPBind => {
            unimplemented!(
                "Socks5 bind protocol not supported, client_addr: {}",
                server_state.incoming_connection_addr
            )
        }
        Socks5Command::UDPAssociate => {
            unimplemented!(
                "Socks5 udp associate not supported, client_addr: {}",
                server_state.incoming_connection_addr
            )
        }
    }
    Ok(())
}
