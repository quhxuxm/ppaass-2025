use crate::client::ClientTcpRelayEndpoint;
use crate::config::{ProxyConfig, PROXY_SERVER_CONFIG};
use crate::destination;
use crate::destination::Destination;
use crate::error::ProxyError;
use crate::user::ProxyUserInfo;
use bincode::config::Configuration;
use destination::tcp::TcpDestEndpoint;
use futures_util::{SinkExt, StreamExt};
use ppaass_2025_core::{
    random_generate_encryption, rsa_decrypt_encryption, rsa_encrypt_encryption, CoreServerConfig,
    CoreServerState, SecureLengthDelimitedCodec,
};
use ppaass_2025_protocol::{
    ClientHandshake, ClientSetupDestination, Encryption, ServerHandshake, ServerSetupDestination,
};
use ppaass_2025_user::{FileSystemUserRepository, UserInfo, UserRepository};
use std::sync::Arc;
use tokio::io::copy_bidirectional;
use tokio_util::bytes::BytesMut;
use tokio_util::codec::Framed;
use tracing::debug;
type ServerState<'a> = CoreServerState<
    ProxyConfig,
    &'a ProxyConfig,
    FileSystemUserRepository<ProxyUserInfo, ProxyConfig>,
>;
struct HandshakeResult {
    client_username: String,
    client_encryption: Encryption,
    server_encryption: Encryption,
}
struct SetupDestinationResult {
    client_encryption: Arc<Encryption>,
    server_encryption: Arc<Encryption>,
    destination: Destination,
}
async fn process_handshake(
    core_server_state: &mut ServerState<'_>,
) -> Result<HandshakeResult, ProxyError> {
    let mut handshake_framed = Framed::new(
        &mut core_server_state.client_stream,
        SecureLengthDelimitedCodec::new(
            PROXY_SERVER_CONFIG.handshake_decoder_encryption(),
            PROXY_SERVER_CONFIG.handshake_encoder_encryption(),
        ),
    );
    debug!(
        "Waiting for receive handshake from client [{}]",
        core_server_state.client_addr
    );
    let handshake =
        handshake_framed
            .next()
            .await
            .ok_or(ProxyError::ClientConnectionExhausted(
                core_server_state.client_addr,
            ))??;
    let (
        ClientHandshake {
            username: client_username,
            encryption: client_encryption,
        },
        _,
    ) = bincode::decode_from_slice::<ClientHandshake, bincode::config::Configuration>(
        &handshake,
        bincode::config::standard(),
    )?;
    let proxy_user_info = core_server_state
        .user_repository
        .find_user(&client_username)
        .await
        .ok_or(ProxyError::ClientUserNotExist(client_username.clone()))?;
    let client_encryption = rsa_decrypt_encryption(
        &client_encryption,
        proxy_user_info
            .rsa_crypto()
            .ok_or(ProxyError::RsaCryptoNotExist(client_username.clone()))?,
    )?;
    debug!(
        "Receive handshake from client [{}], username: {client_username}, client_encryption: {client_encryption:?}",
        core_server_state.client_addr
    );
    let server_encryption = random_generate_encryption();
    let rsa_encrypted_server_encryption = rsa_encrypt_encryption(
        &server_encryption,
        proxy_user_info
            .rsa_crypto()
            .ok_or(ProxyError::RsaCryptoNotExist(client_username.clone()))?,
    )?;
    let server_handshake = ServerHandshake {
        encryption: rsa_encrypted_server_encryption.into_owned(),
    };
    let server_handshake = bincode::encode_to_vec(server_handshake, bincode::config::standard())?;
    handshake_framed.send(&server_handshake).await?;
    debug!(
        "Send handshake to client [{}], username: {client_username}, client_encryption: {server_encryption:?}",
        core_server_state.client_addr
    );
    Ok(HandshakeResult {
        client_username,
        client_encryption: client_encryption.into_owned(),
        server_encryption,
    })
}
async fn process_setup_destination(
    core_server_state: &mut ServerState<'_>,
    handshake_result: HandshakeResult,
) -> Result<SetupDestinationResult, ProxyError> {
    let HandshakeResult {
        client_username,
        client_encryption,
        server_encryption,
    } = handshake_result;
    debug!("Begin to setup destination for client user: {client_username}");
    let client_encryption = Arc::new(client_encryption);
    let server_encryption = Arc::new(server_encryption);
    let mut setup_destination_frame = Framed::new(
        &mut core_server_state.client_stream,
        SecureLengthDelimitedCodec::new(client_encryption.clone(), server_encryption.clone()),
    );
    let setup_destination_data_packet =
        setup_destination_frame
            .next()
            .await
            .ok_or(ProxyError::ClientConnectionExhausted(
                core_server_state.client_addr,
            ))??;
    let (setup_destination, _) = bincode::decode_from_slice::<ClientSetupDestination, Configuration>(
        &setup_destination_data_packet,
        bincode::config::standard(),
    )?;
    let destination = match setup_destination {
        ClientSetupDestination::Tcp { dst_addr } => {
            Destination::Tcp(TcpDestEndpoint::connect(dst_addr).await?)
        }
        ClientSetupDestination::Udp { .. } => {
            unimplemented!("UDP still not support")
        }
    };
    let server_setup_destination_data_packet = ServerSetupDestination::Success;
    let server_setup_destination_data_packet = bincode::encode_to_vec(
        server_setup_destination_data_packet,
        bincode::config::standard(),
    )?;
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
    core_server_state: ServerState<'_>,
    setup_target_endpoint_result: SetupDestinationResult,
) -> Result<(), ProxyError> {
    let SetupDestinationResult {
        client_encryption,
        server_encryption,
        destination,
    } = setup_target_endpoint_result;
    let ServerState {
        client_stream,
        client_addr,
        ..
    } = core_server_state;
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
        Destination::Udp(_) => {
            unimplemented!("UDP still not support")
        }
    }
    Ok(())
}
pub async fn process(
    mut core_server_state: CoreServerState<
        ProxyConfig,
        &ProxyConfig,
        FileSystemUserRepository<ProxyUserInfo, ProxyConfig>,
    >,
) -> Result<(), ProxyError> {
    let handshake_result = process_handshake(&mut core_server_state).await?;
    let setup_target_endpoint_result =
        process_setup_destination(&mut core_server_state, handshake_result).await?;
    process_relay(core_server_state, setup_target_endpoint_result).await?;
    Ok(())
}
