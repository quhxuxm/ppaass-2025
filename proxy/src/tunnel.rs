use crate::config::{ProxyConfig, PROXY_SERVER_CONFIG};
use crate::destination;
use crate::destination::udp::UdpDestEndpoint;
use crate::destination::Destination;
use crate::error::ProxyError;
use crate::user::ProxyUserInfo;
use bincode::config::Configuration;
use destination::tcp::TcpDestEndpoint;
use futures_util::{SinkExt, StreamExt};
use ppaass_2025_core::{random_generate_encryption, rsa_decrypt_encryption, rsa_encrypt_encryption, CoreServerConfig, CoreServerState, SecureLengthDelimitedCodec};
use ppaass_2025_protocol::{ClientHandshake, ClientSetupTargetEndpoint, Encryption, ServerHandshake, ServerSetupTargetEndpoint};
use ppaass_2025_user::{FileSystemUserRepository, UserInfo, UserRepository};
use std::sync::Arc;
use tokio_util::bytes::BytesMut;
use tokio_util::codec::Framed;
use tracing::debug;
type ServerState<'a> = CoreServerState<ProxyConfig, &'a ProxyConfig, FileSystemUserRepository<ProxyUserInfo, ProxyConfig>>;
struct HandshakeResult {
    client_username: String,
    client_encryption: Encryption,
    server_encryption: Encryption,
}
async fn process_handshake(core_server_state: &mut ServerState<'_>) -> Result<HandshakeResult, ProxyError> {
    let mut handshake_framed = Framed::new(&mut core_server_state.client_stream, SecureLengthDelimitedCodec::new(PROXY_SERVER_CONFIG.handshake_decoder_encryption(), PROXY_SERVER_CONFIG.handshake_encoder_encryption()));
    debug!("Waiting for receive handshake from client [{}]", core_server_state.client_addr);
    let handshake = handshake_framed.next().await.ok_or(ProxyError::ClientConnectionExhausted(core_server_state.client_addr))??;
    let (ClientHandshake { username: client_username, encryption: client_encryption }, _) = bincode::decode_from_slice::<ClientHandshake, bincode::config::Configuration>(&handshake, bincode::config::standard())?;
    let proxy_user_info = core_server_state.user_repository.find_user(&client_username).await.ok_or(ProxyError::ClientUserNotExist(client_username.clone()))?;
    let client_encryption = rsa_decrypt_encryption(&client_encryption, proxy_user_info.rsa_crypto().ok_or(ProxyError::RsaCryptoNotExist(client_username.clone()))?)?;
    debug!("Receive handshake from client [{}], username: {client_username}, client_encryption: {client_encryption:?}", core_server_state.client_addr);
    let server_encryption = random_generate_encryption();
    let rsa_encrypted_server_encryption = rsa_encrypt_encryption(&server_encryption, proxy_user_info.rsa_crypto().ok_or(ProxyError::RsaCryptoNotExist(client_username.clone()))?)?;
    let server_handshake = ServerHandshake {
        encryption: rsa_encrypted_server_encryption.into_owned()
    };
    let server_handshake = bincode::encode_to_vec(server_handshake, bincode::config::standard())?;
    handshake_framed.send(BytesMut::from_iter(server_handshake)).await?;
    debug!("Send handshake to client [{}], username: {client_username}, client_encryption: {server_encryption:?}",core_server_state.client_addr);
    Ok(HandshakeResult {
        client_username,
        client_encryption: client_encryption.into_owned(),
        server_encryption,
    })
}
async fn process_setup_target_endpoint(core_server_state: &mut ServerState<'_>, handshake_result: HandshakeResult) -> Result<Destination, ProxyError> {
    let HandshakeResult {
        client_username, client_encryption, server_encryption
    } = handshake_result;
    let client_encryption = Arc::new(client_encryption);
    let server_encryption = Arc::new(server_encryption);
    let mut setup_target_endpoint_frame = Framed::new(&mut core_server_state.client_stream, SecureLengthDelimitedCodec::new(client_encryption.clone(), server_encryption.clone()));
    let setup_endpoint = setup_target_endpoint_frame.next().await.ok_or(ProxyError::ClientConnectionExhausted(core_server_state.client_addr))??;
    let (setup_target_endpoint, _) = bincode::decode_from_slice::<ClientSetupTargetEndpoint, Configuration>(&setup_endpoint, bincode::config::standard())?;
    let destination = match setup_target_endpoint {
        ClientSetupTargetEndpoint::Tcp { dst_addr } => {
            Destination::Tcp(TcpDestEndpoint::connect(dst_addr).await?)
        }
        ClientSetupTargetEndpoint::Udp { .. } => {
            unimplemented!("UDP still not support")
        }
    };
    let server_setup_target_endpoint = ServerSetupTargetEndpoint::Success;
    let server_setup_target_endpoint = bincode::encode_to_vec(server_setup_target_endpoint, bincode::config::standard())?;
    setup_target_endpoint_frame.send(BytesMut::from_iter(server_setup_target_endpoint)).await?;
    Ok(destination)
}
async fn process_tcp_relay(core_server_state: &mut ServerState<'_>, tcp_dest_endpoint: TcpDestEndpoint) -> Result<(), ProxyError> {
    Ok(())
}
async fn process_udp_relay(core_server_state: &mut ServerState<'_>, udp_dest_endpoint: UdpDestEndpoint) -> Result<(), ProxyError> {
    unimplemented!("UDP still not support")
}
pub async fn process(mut core_server_state: CoreServerState<ProxyConfig, &ProxyConfig, FileSystemUserRepository<ProxyUserInfo, ProxyConfig>>) -> Result<(), ProxyError> {
    let handshake_result = process_handshake(&mut core_server_state).await?;
    let destination = process_setup_target_endpoint(&mut core_server_state, handshake_result).await?;
    match destination {
        Destination::Tcp(tcp_destination_endpoint) => {
            process_tcp_relay(&mut core_server_state, tcp_destination_endpoint).await?;
        }
        Destination::Udp(udp_dest_endpoint) => {
            process_udp_relay(&mut core_server_state, udp_dest_endpoint).await?;
        }
    }
    Ok(())
}