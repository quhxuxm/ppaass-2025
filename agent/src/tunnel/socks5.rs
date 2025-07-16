use crate::config::get_config;
use crate::error::Error;
use crate::tunnel::fetch_proxy_connection;
use common::proxy::DestinationType;
use common::{ServerState, WithServerConfig};
use fast_socks5::server::{Socks5ServerProtocol, SocksServerError, run_udp_proxy_custom};
use fast_socks5::util::target_addr::TargetAddr;
use fast_socks5::{Socks5Command, parse_udp_request};
use protocol::UnifiedAddress;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use tokio::io::{AsyncReadExt, AsyncWriteExt, copy_bidirectional};
use tokio::net::UdpSocket;
use tracing::{debug, error, info};
fn convert_address(address: &TargetAddr) -> UnifiedAddress {
    match address {
        TargetAddr::Ip(dst_addr) => dst_addr.into(),
        TargetAddr::Domain(host, port) => UnifiedAddress::Domain {
            host: host.clone(),
            port: *port,
        },
    }
}
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
            let proxy_connection = fetch_proxy_connection().await?;
            let destination_address = convert_address(&dst_addr);
            let mut proxy_connection = proxy_connection
                .setup_destination(destination_address, DestinationType::Tcp)
                .await?;

            let mut socks5_client_stream = socks5_client_stream
                .reply_success(SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0)))
                .await?;

            // Proxying data
            let (from_client, from_proxy) =
                match copy_bidirectional(&mut socks5_client_stream, &mut proxy_connection).await {
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
            run_udp_proxy_custom(
                socks5_client_stream,
                &dst_addr,
                None,
                get_config().listening_address().ip(),
                |client_udp_socket| async move {
                    let client_udp_socket =
                        UdpSocket::from_std(client_udp_socket.into()).map_err(|e| {
                            SocksServerError::Io {
                                source: e,
                                context: "Fail to create client udp socket.",
                            }
                        })?;
                    let mut client_udp_socks5_packet = vec![0u8; 8192];
                    client_udp_socket
                        .recv(&mut client_udp_socks5_packet)
                        .await
                        .map_err(|e| SocksServerError::Io {
                            source: e,
                            context: "Fail to read client udp data to proxy.",
                        })?;
                    let (_, dst_addr, client_udp_data) =
                        parse_udp_request(&client_udp_socks5_packet).await?;
                    let proxy_connection =
                        fetch_proxy_connection()
                            .await
                            .map_err(|e| SocksServerError::Io {
                                source: std::io::Error::other(format!(
                                    "Fail to build proxy connection: {e:?}"
                                )),
                                context: "Fail to build proxy connection.",
                            })?;

                    let destination_address = convert_address(&dst_addr);
                    let mut proxy_connection = proxy_connection
                        .setup_destination(destination_address, DestinationType::Udp)
                        .await
                        .map_err(|e| SocksServerError::Io {
                            source: std::io::Error::other(format!(
                                "Fail to do setup destination with proxy connection: {e:?}"
                            )),
                            context: "Fail to do setup destination with proxy connection.",
                        })?;
                    proxy_connection.write(client_udp_data).await.map_err(|e| {
                        SocksServerError::Io {
                            source: e,
                            context: "Fail to write client udp data to proxy.",
                        }
                    })?;
                    let mut proxy_udp_data_buf = vec![0u8; 8192];
                    proxy_connection
                        .read(&mut proxy_udp_data_buf)
                        .await
                        .map_err(|e| SocksServerError::Io {
                            source: e,
                            context: "Fail to read proxy udp data.",
                        })?;
                    client_udp_socket
                        .send(&proxy_udp_data_buf)
                        .await
                        .map_err(|e| SocksServerError::Io {
                            source: e,
                            context: "Fail to write proxy udp data to client.",
                        })?;
                    Ok(())
                },
            )
            .await?;
        }
    }
    Ok(())
}
