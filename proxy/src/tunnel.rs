use crate::client::ClientTcpRelayEndpoint;
use crate::config::get_config;
use crate::destination;
use crate::destination::Destination;
use crate::destination::udp::UdpDestEndpoint;
use crate::error::Error;
use crate::user::{get_forward_user_repo, get_user_repo};
use bincode::config::Configuration;
use common::Error as CommonError;
use common::config::WithUsernameConfig;
use common::proxy::{DestinationType, ProxyConnection};
use common::user::User;
use common::user::UserRepository;
use common::{
    SecureLengthDelimitedCodec, ServerState, get_handshake_encryption, random_generate_encryption,
    rsa_decrypt_encryption, rsa_encrypt_encryption,
};
use destination::tcp::TcpDestEndpoint;
use futures_util::{SinkExt, StreamExt};
use protocol::{
    ClientHandshake, ClientSetupDestination, Encryption, ServerHandshake, ServerSetupDestination,
};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt, copy_bidirectional};
use tokio_util::codec::Framed;
use tracing::debug;
struct HandshakeResult {
    client_username: String,
    client_encryption: Arc<Encryption>,
    server_encryption: Arc<Encryption>,
}
struct SetupDestinationResult {
    client_encryption: Arc<Encryption>,
    server_encryption: Arc<Encryption>,
    destination: Destination,
}
async fn process_handshake(server_state: &mut ServerState) -> Result<HandshakeResult, Error> {
    let mut handshake_framed = Framed::new(
        &mut server_state.incoming_stream,
        SecureLengthDelimitedCodec::new(get_handshake_encryption(), get_handshake_encryption()),
    );
    debug!(
        "Waiting for receive handshake from client [{}]",
        server_state.incoming_connection_addr
    );
    let handshake = handshake_framed
        .next()
        .await
        .ok_or(CommonError::ConnectionExhausted(format!(
            "Fail to read handshake message from agent: {}",
            server_state.incoming_connection_addr
        )))??;
    let (
        ClientHandshake {
            username: client_username,
            encryption: client_encryption,
        },
        _,
    ) = bincode::decode_from_slice::<ClientHandshake, Configuration>(
        &handshake,
        bincode::config::standard(),
    )
        .map_err(CommonError::Decode)?;
    debug!(
        "Receive client handshake, client username: {client_username}, client encryption: {client_encryption:?}"
    );
    let proxy_user_info = get_user_repo()
        .find_user(&client_username)
        .ok_or(CommonError::UserNotExist(client_username.clone()))?;
    let client_encryption = rsa_decrypt_encryption(
        client_encryption,
        proxy_user_info
            .rsa_crypto()
            .ok_or(CommonError::UserRsaCryptoNotExist(client_username.clone()))?,
    )?;
    let client_encryption = Arc::new(client_encryption);
    debug!(
        "Receive handshake from client [{}], username: {client_username}, client_encryption: {client_encryption:?}",
        server_state.incoming_connection_addr
    );
    let server_encryption = random_generate_encryption();
    let server_encryption = Arc::new(server_encryption);
    let rsa_encrypted_server_encryption = rsa_encrypt_encryption(
        &server_encryption,
        proxy_user_info
            .rsa_crypto()
            .ok_or(CommonError::UserRsaCryptoNotExist(client_username.clone()))?,
    )?;
    let server_handshake = ServerHandshake {
        encryption: rsa_encrypted_server_encryption.into_owned(),
    };
    let server_handshake_bytes =
        bincode::encode_to_vec(server_handshake, bincode::config::standard())
            .map_err(CommonError::Encode)?;
    handshake_framed.send(&server_handshake_bytes).await?;
    debug!(
        "Send handshake to client [{}], username: {client_username}, client_encryption: {client_encryption:?}, server_encryption: {server_encryption:?}",
        server_state.incoming_connection_addr
    );
    Ok(HandshakeResult {
        client_username,
        client_encryption,
        server_encryption,
    })
}
async fn process_setup_destination(
    server_state: &mut ServerState,
    handshake_result: HandshakeResult,
) -> Result<SetupDestinationResult, Error> {
    let HandshakeResult {
        client_username,
        client_encryption,
        server_encryption,
    } = handshake_result;
    debug!("Begin to setup destination for client user: {client_username}");
    let mut setup_destination_frame = Framed::new(
        &mut server_state.incoming_stream,
        SecureLengthDelimitedCodec::new(
            Arc::clone(&client_encryption),
            Arc::clone(&server_encryption),
        ),
    );
    let setup_destination_data_packet =
        setup_destination_frame
            .next()
            .await
            .ok_or(CommonError::ConnectionExhausted(format!(
                "Fail to read destination setup message from agent: {}",
                server_state.incoming_connection_addr
            )))??;
    let (setup_destination, _) =
        bincode::decode_from_slice::<ClientSetupDestination, Configuration>(
            &setup_destination_data_packet,
            bincode::config::standard(),
        )
            .map_err(CommonError::Decode)?;
    let destination = match (get_config().forward(), get_forward_user_repo()) {
        (Some(forward_config), Some(forward_user_repository)) => {
            let forward_user_info = forward_user_repository
                .find_user(forward_config.username())
                .ok_or(CommonError::UserNotExist(
                    forward_config.username().to_owned(),
                ))?;
            match setup_destination {
                ClientSetupDestination::Tcp(dst_addr) => {
                    let proxy_connection = ProxyConnection::new(
                        forward_user_info,
                        forward_config.proxy_connect_timeout(),
                    )
                        .await?;
                    let proxy_connection = proxy_connection
                        .setup_destination(dst_addr, DestinationType::Tcp)
                        .await?;
                    Destination::Forward(Box::new(proxy_connection))
                }
                ClientSetupDestination::Udp { .. } => {
                    unimplemented!("UDP still not support")
                }
            }
        }
        _ => match setup_destination {
            ClientSetupDestination::Tcp(dst_addr) => Destination::Tcp(
                TcpDestEndpoint::connect(dst_addr, get_config().destination_connect_timeout())
                    .await?,
            ),
            ClientSetupDestination::Udp(dst_addr) => Destination::Udp {
                dst_udp_endpoint: UdpDestEndpoint::bind().await?,
                dst_addr,
            },
        },
    };
    let server_setup_destination_data_packet = ServerSetupDestination::Success;
    let server_setup_destination_data_packet = bincode::encode_to_vec(
        server_setup_destination_data_packet,
        bincode::config::standard(),
    )
        .map_err(CommonError::Encode)?;
    setup_destination_frame
        .send(&server_setup_destination_data_packet)
        .await?;
    Ok(SetupDestinationResult {
        client_encryption,
        server_encryption,
        destination,
    })
}
async fn process_relay(
    server_state: ServerState,
    setup_target_endpoint_result: SetupDestinationResult,
) -> Result<(), Error> {
    let SetupDestinationResult {
        client_encryption,
        server_encryption,
        destination,
    } = setup_target_endpoint_result;
    let ServerState {
        incoming_stream: client_stream,
        incoming_connection_addr: client_addr,
    } = server_state;
    match destination {
        Destination::Tcp(mut dst_tcp_endpoint) => {
            debug!(
                "Begin to relay tcp data from client [{client_addr}] to destination [{}]",
                dst_tcp_endpoint.dst_addr()
            );
            let mut client_tcp_relay_endpoint =
                ClientTcpRelayEndpoint::new(client_stream, client_encryption, server_encryption);
            copy_bidirectional(&mut client_tcp_relay_endpoint, &mut dst_tcp_endpoint).await?;
        }
        Destination::Forward(mut forward_proxy_connection) => {
            let mut client_tcp_relay_endpoint =
                ClientTcpRelayEndpoint::new(client_stream, client_encryption, server_encryption);
            copy_bidirectional(
                &mut client_tcp_relay_endpoint,
                &mut forward_proxy_connection,
            )
                .await?;
        }
        Destination::Udp {
            dst_udp_endpoint,
            dst_addr,
        } => {
            let mut client_tcp_relay_endpoint =
                ClientTcpRelayEndpoint::new(client_stream, client_encryption, server_encryption);
            let mut client_data = [0u8; 65536];
            AsyncReadExt::read(&mut client_tcp_relay_endpoint, &mut client_data).await?;
            let dst_sock_addrs: Vec<SocketAddr> = (&dst_addr).try_into()?;
            let dst_udp_data = dst_udp_endpoint
                .replay_to(&dst_sock_addrs[..], &client_data)
                .await?;
            AsyncWriteExt::write(&mut client_tcp_relay_endpoint, &dst_udp_data).await?;
        }
    }
    Ok(())
}
pub async fn process(mut server_state: ServerState) -> Result<(), Error> {
    // Process handshake
    let handshake_result = process_handshake(&mut server_state).await?;
    // Process destination setup
    let setup_target_endpoint_result =
        process_setup_destination(&mut server_state, handshake_result).await?;
    // Process relay
    process_relay(server_state, setup_target_endpoint_result).await?;
    Ok(())
}
