use crate::client::ClientTcpRelayEndpoint;
use crate::config::get_proxy_config;
use crate::destination;
use crate::destination::Destination;
use crate::error::ProxyError;
use crate::user::get_proxy_user_repo;
use bincode::config::Configuration;
use destination::tcp::TcpDestEndpoint;
use futures_util::{SinkExt, StreamExt};
use ppaass_2025_common::config::UserRepositoryConfig;
use ppaass_2025_common::proxy::{ProxyConnection, ProxyConnectionDestinationType};
use ppaass_2025_common::user::UserRepository;
use ppaass_2025_common::user::{BasicUser, ProxyConnectionUser};
use ppaass_2025_common::{
    random_generate_encryption, rsa_decrypt_encryption, rsa_encrypt_encryption, BaseServerState,
    SecureLengthDelimitedCodec, HANDSHAKE_ENCRYPTION,
};
use ppaass_2025_protocol::{
    ClientHandshake, ClientSetupDestination, Encryption, ServerHandshake, ServerSetupDestination,
};
use serde::de::DeserializeOwned;
use std::borrow::Cow;
use std::sync::Arc;
use tokio::io::copy_bidirectional;
use tokio_util::codec::Framed;
use tracing::debug;
struct HandshakeResult {
    client_username: String,
    client_encryption: Encryption,
    server_encryption: Encryption,
}
struct SetupDestinationResult<'a> {
    client_encryption: Arc<Encryption>,
    server_encryption: Arc<Encryption>,
    destination: Destination<'a>,
}
async fn process_handshake(
    server_state: &mut BaseServerState,
) -> Result<HandshakeResult, ProxyError> {
    let handshake_encryption = &*HANDSHAKE_ENCRYPTION;
    let mut handshake_framed = Framed::new(
        &mut server_state.client_stream,
        SecureLengthDelimitedCodec::new(
            Cow::Borrowed(handshake_encryption),
            Cow::Borrowed(handshake_encryption),
        ),
    );
    debug!(
        "Waiting for receive handshake from client [{}]",
        server_state.client_addr
    );
    let handshake =
        handshake_framed
            .next()
            .await
            .ok_or(ProxyError::ClientConnectionExhausted(
                server_state.client_addr,
            ))??;
    let (
        ClientHandshake {
            username: client_username,
            encryption: client_encryption,
        },
        _,
    ) = bincode::decode_from_slice::<ClientHandshake, Configuration>(
        &handshake,
        bincode::config::standard(),
    )?;
    debug!(
        "Receive client handshake, client username: {client_username}, client encryption: {client_encryption:?}"
    );
    let proxy_user_info = get_proxy_user_repo()
        .find_user(&client_username)
        .ok_or(ProxyError::ClientUserNotExist(client_username.clone()))?;
    let client_encryption = rsa_decrypt_encryption(
        &client_encryption,
        proxy_user_info
            .rsa_crypto()
            .ok_or(ProxyError::RsaCryptoNotExist(client_username.clone()))?,
    )?;
    debug!(
        "Receive handshake from client [{}], username: {client_username}, client_encryption: {client_encryption:?}",
        server_state.client_addr
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
    let server_handshake_bytes =
        bincode::encode_to_vec(server_handshake, bincode::config::standard())?;
    handshake_framed.send(&server_handshake_bytes).await?;
    debug!(
        "Send handshake to client [{}], username: {client_username}, client_encryption: {client_encryption:?}, server_encryption: {server_encryption:?}",
        server_state.client_addr
    );
    Ok(HandshakeResult {
        client_username,
        client_encryption: client_encryption.into_owned(),
        server_encryption,
    })
}
async fn process_setup_destination<'a, ForwardUserRepo, ProxyConnUser, ForwardUserRepoConfig>(
    server_state: &mut BaseServerState,
    forward_user_repository: Option<&ForwardUserRepo>,
    handshake_result: HandshakeResult,
) -> Result<SetupDestinationResult<'a>, ProxyError>
where
    ForwardUserRepo:
        UserRepository<UserInfoType = ProxyConnUser, UserRepoConfigType = ForwardUserRepoConfig>,
    ProxyConnUser: ProxyConnectionUser + DeserializeOwned + Send + Sync + 'static,
    ForwardUserRepoConfig: UserRepositoryConfig,
{
    let HandshakeResult {
        client_username,
        client_encryption,
        server_encryption,
    } = handshake_result;
    debug!("Begin to setup destination for client user: {client_username}");
    let client_encryption = Arc::new(client_encryption);
    let server_encryption = Arc::new(server_encryption);
    let mut setup_destination_frame = Framed::new(
        &mut server_state.client_stream,
        SecureLengthDelimitedCodec::new(
            Cow::Borrowed(&client_encryption),
            Cow::Borrowed(&server_encryption),
        ),
    );
    let setup_destination_data_packet =
        setup_destination_frame
            .next()
            .await
            .ok_or(ProxyError::ClientConnectionExhausted(
                server_state.client_addr,
            ))??;
    let (setup_destination, _) = bincode::decode_from_slice::<ClientSetupDestination, Configuration>(
        &setup_destination_data_packet,
        bincode::config::standard(),
    )?;
    let destination = match (get_proxy_config().forward(), forward_user_repository) {
        (Some(forward_config), Some(forward_user_repository)) => match setup_destination {
            ClientSetupDestination::Tcp { dst_addr } => {
                let proxy_connection =
                    ProxyConnection::new(forward_config, forward_user_repository).await?;
                let proxy_connection = proxy_connection.handshake().await?;
                let proxy_connection = proxy_connection
                    .setup_destination(dst_addr, ProxyConnectionDestinationType::Tcp)
                    .await?;
                Destination::Forward(proxy_connection)
            }
            ClientSetupDestination::Udp { .. } => {
                unimplemented!("UDP still not support")
            }
        },
        _ => match setup_destination {
            ClientSetupDestination::Tcp { dst_addr } => {
                Destination::Tcp(TcpDestEndpoint::connect(dst_addr).await?)
            }
            ClientSetupDestination::Udp { .. } => {
                unimplemented!("UDP still not support")
            }
        },
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
    server_state: BaseServerState,
    setup_target_endpoint_result: SetupDestinationResult<'_>,
) -> Result<(), ProxyError> {
    let SetupDestinationResult {
        client_encryption,
        server_encryption,
        destination,
    } = setup_target_endpoint_result;
    let BaseServerState {
        client_stream,
        client_addr,
    } = server_state;
    match destination {
        Destination::Tcp(mut dst_tcp_endpoint) => {
            debug!(
                "Begin to relay tcp data from client [{client_addr}] to destination [{}]",
                dst_tcp_endpoint.dst_addr()
            );
            let mut client_tcp_relay_endpoint = ClientTcpRelayEndpoint::new(
                client_stream,
                &*client_encryption,
                &*server_encryption,
            );
            copy_bidirectional(&mut client_tcp_relay_endpoint, &mut dst_tcp_endpoint).await?;
        }
        Destination::Forward(mut forward_proxy_connection) => {
            let mut client_tcp_relay_endpoint = ClientTcpRelayEndpoint::new(
                client_stream,
                &*client_encryption,
                &*server_encryption,
            );
            copy_bidirectional(
                &mut client_tcp_relay_endpoint,
                &mut forward_proxy_connection,
            )
            .await?;
        }
        Destination::Udp(_) => {
            unimplemented!("UDP still not support")
        }
    }
    Ok(())
}
pub async fn process<FUR, PU, FURC>(
    mut server_state: BaseServerState,
    forward_user_repository: Option<&FUR>,
) -> Result<(), ProxyError>
where
    FUR: UserRepository<UserInfoType = PU, UserRepoConfigType = FURC>,
    PU: ProxyConnectionUser + DeserializeOwned + Send + Sync + 'static,
    FURC: UserRepositoryConfig,
{
    let handshake_result = process_handshake(&mut server_state).await?;
    let setup_target_endpoint_result =
        process_setup_destination(&mut server_state, forward_user_repository, handshake_result)
            .await?;
    process_relay(server_state, setup_target_endpoint_result).await?;
    Ok(())
}
