use crate::config::{AgentConfig, AGENT_CONFIG};
use crate::error::AgentError;
use crate::user::AgentUserInfo;
use bincode::config::Configuration;
use futures_util::{SinkExt, StreamExt};
use ppaass_2025_common::{
    random_generate_encryption, CoreServerConfig, SecureLengthDelimitedCodec,
};
use ppaass_2025_protocol::{
    ClientHandshake, ClientSetupDestination, Encryption, ServerHandshake, ServerSetupDestination,
    UnifiedAddress,
};
use ppaass_2025_user::{FileSystemUserRepository, UserRepository};
use std::io::Error;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio::net::TcpStream;
use tokio_util::codec::Framed;
pub enum ProxyConnectionDestinationType {
    Tcp,
    Udp,
}
pub struct Initial {
    proxy_stream: TcpStream,
    proxy_addr: SocketAddr,
}

pub struct HandshakeReady {
    proxy_stream: TcpStream,
    proxy_addr: SocketAddr,
    proxy_encryption: Encryption,
    agent_encryption: Encryption,
}

pub struct DestinationReady {
    proxy_stream: TcpStream,
    proxy_addr: SocketAddr,
    proxy_encryption: Arc<Encryption>,
    agent_encryption: Arc<Encryption>,
}

pub struct ProxyConnection<T> {
    state: T,
}
impl ProxyConnection<Initial> {
    pub async fn new(
        user_repository: &FileSystemUserRepository<AgentUserInfo, AgentConfig>,
    ) -> Result<Self, AgentError> {
        let user_info = user_repository
            .find_user(AGENT_CONFIG.username())
            .await
            .ok_or(AgentError::UserNotExist(AGENT_CONFIG.username().to_owned()))?;
        let proxy_stream = TcpStream::connect(user_info.proxy_servers()).await?;
        Ok(Self {
            state: Initial {
                proxy_addr: proxy_stream.peer_addr()?,
                proxy_stream,
            },
        })
    }

    pub async fn handshake(mut self) -> Result<ProxyConnection<HandshakeReady>, AgentError> {
        let mut handshake_framed = Framed::new(
            &mut self.state.proxy_stream,
            SecureLengthDelimitedCodec::new(
                AGENT_CONFIG.handshake_decoder_encryption(),
                AGENT_CONFIG.handshake_encoder_encryption(),
            ),
        );
        let agent_encryption = random_generate_encryption();
        let client_handshake = ClientHandshake {
            username: AGENT_CONFIG.username().to_owned(),
            encryption: agent_encryption.clone(),
        };
        let client_handshake_bytes =
            bincode::encode_to_vec(client_handshake, bincode::config::standard())?;
        handshake_framed.send(&client_handshake_bytes).await?;
        let proxy_handshake_bytes = handshake_framed
            .next()
            .await
            .ok_or(AgentError::ProxyConnectionExhausted(self.state.proxy_addr))??;
        let (proxy_handshake, _) = bincode::decode_from_slice::<ServerHandshake, Configuration>(
            &proxy_handshake_bytes,
            bincode::config::standard(),
        )?;

        Ok(ProxyConnection {
            state: HandshakeReady {
                proxy_stream: self.state.proxy_stream,
                proxy_addr: self.state.proxy_addr,
                proxy_encryption: proxy_handshake.encryption,
                agent_encryption,
            },
        })
    }
}

impl ProxyConnection<HandshakeReady> {
    pub async fn setup_destination(
        mut self,
        destination_addr: UnifiedAddress,
        destination_type: ProxyConnectionDestinationType,
    ) -> Result<ProxyConnection<DestinationReady>, AgentError> {
        let agent_encryption = Arc::new(self.state.agent_encryption);
        let proxy_encryption = Arc::new(self.state.proxy_encryption);
        let mut setup_destination_framed = Framed::new(
            &mut self.state.proxy_stream,
            SecureLengthDelimitedCodec::new(proxy_encryption.clone(), agent_encryption.clone()),
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
                    proxy_stream: self.state.proxy_stream,
                    proxy_addr: self.state.proxy_addr,
                    proxy_encryption,
                    agent_encryption,
                },
            }),
            ServerSetupDestination::Fail => Err(AgentError::ProxyConnectionSetupDestination(
                destination_addr,
            )),
        }
    }
}
impl AsyncRead for ProxyConnection<DestinationReady> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        todo!()
    }
}
impl AsyncWrite for ProxyConnection<DestinationReady> {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, Error>> {
        todo!()
    }
    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
        todo!()
    }
    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
        todo!()
    }
}
