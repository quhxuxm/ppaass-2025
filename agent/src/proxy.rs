use crate::config::AgentConfig;
use crate::error::AgentError;
use crate::user::AgentUserInfo;
use bincode::config::Configuration;
use futures_util::{SinkExt, StreamExt};
use ppaass_2025_common::user::UserRepository;
use ppaass_2025_common::user::repo::FileSystemUserRepository;
use ppaass_2025_common::user::user::BasicUser;
use ppaass_2025_common::{
    HANDSHAKE_ENCRYPTION, SecureLengthDelimitedCodec, random_generate_encryption,
    rsa_decrypt_encryption, rsa_encrypt_encryption,
};
use ppaass_2025_protocol::{
    ClientHandshake, ClientSetupDestination, Encryption, ServerHandshake, ServerSetupDestination,
    UnifiedAddress,
};
use std::borrow::Cow;
use std::io::Error;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio::net::TcpStream;
use tokio::pin;
use tokio_util::bytes::BytesMut;
use tokio_util::codec::Framed;
use tokio_util::io::{SinkWriter, StreamReader};
pub enum ProxyConnectionDestinationType {
    Tcp,
    #[allow(unused)]
    Udp,
}
pub struct Initial {
    proxy_stream: TcpStream,
    proxy_addr: SocketAddr,
    user_info: Arc<AgentUserInfo>,
    agent_config: Arc<AgentConfig>,
}

pub struct HandshakeReady {
    proxy_stream: TcpStream,
    proxy_addr: SocketAddr,
    proxy_encryption: Encryption,
    agent_encryption: Encryption,
}

pub struct DestinationReady<'a> {
    proxy_framed:
        SinkWriter<StreamReader<Framed<TcpStream, SecureLengthDelimitedCodec<'a>>, BytesMut>>,
}

pub struct ProxyConnection<T> {
    state: T,
}
impl ProxyConnection<Initial> {
    pub async fn new(
        agent_config: Arc<AgentConfig>,
        user_repository: &FileSystemUserRepository<AgentUserInfo, AgentConfig>,
    ) -> Result<Self, AgentError> {
        let user_info = user_repository
            .find_user(agent_config.username())
            .await
            .ok_or(AgentError::UserNotExist(agent_config.username().to_owned()))?;
        let proxy_stream = TcpStream::connect(user_info.proxy_servers()).await?;
        Ok(Self {
            state: Initial {
                proxy_addr: proxy_stream.peer_addr()?,
                proxy_stream,
                user_info,
                agent_config,
            },
        })
    }

    pub async fn handshake(mut self) -> Result<ProxyConnection<HandshakeReady>, AgentError> {
        let handshake_encryption = &*HANDSHAKE_ENCRYPTION;

        let mut handshake_framed = Framed::new(
            &mut self.state.proxy_stream,
            SecureLengthDelimitedCodec::new(
                Cow::Borrowed(handshake_encryption),
                Cow::Borrowed(handshake_encryption),
            ),
        );
        let agent_encryption = random_generate_encryption();
        let rsa_encrypted_agent_encryption = rsa_encrypt_encryption(
            &agent_encryption,
            self.state
                .user_info
                .rsa_crypto()
                .ok_or(AgentError::UserRsaCryptoNotExist(
                    self.state.agent_config.username().to_owned(),
                ))?,
        )?;
        let client_handshake = ClientHandshake {
            username: self.state.agent_config.username().to_owned(),
            encryption: rsa_encrypted_agent_encryption.into_owned(),
        };
        let client_handshake_bytes =
            bincode::encode_to_vec(client_handshake, bincode::config::standard())?;
        handshake_framed.send(&client_handshake_bytes).await?;
        let proxy_handshake_bytes = handshake_framed
            .next()
            .await
            .ok_or(AgentError::ProxyConnectionExhausted(self.state.proxy_addr))??;
        let (rsa_encrypted_proxy_handshake, _) =
            bincode::decode_from_slice::<ServerHandshake, Configuration>(
                &proxy_handshake_bytes,
                bincode::config::standard(),
            )?;
        let proxy_encryption = rsa_decrypt_encryption(
            &rsa_encrypted_proxy_handshake.encryption,
            self.state
                .user_info
                .rsa_crypto()
                .ok_or(AgentError::UserRsaCryptoNotExist(
                    self.state.agent_config.username().to_owned(),
                ))?,
        )?;

        Ok(ProxyConnection {
            state: HandshakeReady {
                proxy_stream: self.state.proxy_stream,
                proxy_addr: self.state.proxy_addr,
                proxy_encryption: proxy_encryption.into_owned(),
                agent_encryption,
            },
        })
    }
}

impl ProxyConnection<HandshakeReady> {
    pub async fn setup_destination<'a>(
        self,
        destination_addr: UnifiedAddress,
        destination_type: ProxyConnectionDestinationType,
    ) -> Result<ProxyConnection<DestinationReady<'a>>, AgentError> {
        let proxy_encryption = self.state.proxy_encryption;
        let agent_encryption = self.state.agent_encryption;
        let mut setup_destination_framed = Framed::new(
            self.state.proxy_stream,
            SecureLengthDelimitedCodec::new(
                Cow::Owned(proxy_encryption),
                Cow::Owned(agent_encryption),
            ),
        );
        let setup_destination = match destination_type {
            ProxyConnectionDestinationType::Tcp => ClientSetupDestination::Tcp {
                dst_addr: destination_addr.clone(),
            },
            ProxyConnectionDestinationType::Udp => {
                unimplemented!("Udp destination not supported.")
            }
        };
        let setup_destination_bytes =
            bincode::encode_to_vec(setup_destination, bincode::config::standard())?;
        setup_destination_framed
            .send(&setup_destination_bytes)
            .await?;
        let proxy_setup_destination_bytes = setup_destination_framed
            .next()
            .await
            .ok_or(AgentError::ProxyConnectionExhausted(self.state.proxy_addr))??;
        let (proxy_setup_destination, _) =
            bincode::decode_from_slice::<ServerSetupDestination, Configuration>(
                &proxy_setup_destination_bytes,
                bincode::config::standard(),
            )?;
        match proxy_setup_destination {
            ServerSetupDestination::Success => Ok(ProxyConnection {
                state: DestinationReady {
                    proxy_framed: SinkWriter::new(StreamReader::new(setup_destination_framed)),
                },
            }),
            ServerSetupDestination::Fail => Err(AgentError::ProxyConnectionSetupDestination(
                destination_addr,
            )),
        }
    }
}
impl<'a> AsyncRead for ProxyConnection<DestinationReady<'a>> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        let proxy_framed = &mut self.get_mut().state.proxy_framed;
        pin!(proxy_framed);
        proxy_framed.poll_read(cx, buf)
    }
}
impl<'a> AsyncWrite for ProxyConnection<DestinationReady<'a>> {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, Error>> {
        let proxy_framed = &mut self.get_mut().state.proxy_framed;
        pin!(proxy_framed);
        proxy_framed.poll_write(cx, buf)
    }
    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
        let proxy_framed = &mut self.get_mut().state.proxy_framed;
        pin!(proxy_framed);
        proxy_framed.poll_flush(cx)
    }
    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
        let proxy_framed = &mut self.get_mut().state.proxy_framed;
        pin!(proxy_framed);
        proxy_framed.poll_shutdown(cx)
    }
}
