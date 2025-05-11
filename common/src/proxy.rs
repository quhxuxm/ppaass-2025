use crate::user::UserWithProxyServers;
use crate::{
    get_handshake_encryption, random_generate_encryption, rsa_decrypt_encryption, rsa_encrypt_encryption,
    Error, SecureLengthDelimitedCodec,
};
use bincode::config::Configuration;
use futures_util::{SinkExt, StreamExt};
use protocol::{
    ClientHandshake, ClientSetupDestination, Encryption, ServerHandshake, ServerSetupDestination,
    UnifiedAddress,
};
use serde::de::DeserializeOwned;
use std::borrow::Cow;
use std::io::Error as StdIoError;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::Duration;
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio::net::TcpStream;
use tokio::pin;
use tokio::time::timeout;
use tokio_util::bytes::BytesMut;
use tokio_util::codec::Framed;
use tokio_util::io::{SinkWriter, StreamReader};
pub enum ProxyConnectionDestinationType {
    Tcp,
    #[allow(unused)]
    Udp,
}
pub struct Initial<U>
where
    U: UserWithProxyServers + Send + Sync + DeserializeOwned + 'static,
{
    proxy_stream: TcpStream,
    proxy_addr: SocketAddr,
    user_info: Arc<U>,
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
impl<U> ProxyConnection<Initial<U>>
where
    U: UserWithProxyServers + DeserializeOwned + Send + Sync + 'static,
{
    pub async fn new(user_info: Arc<U>, connect_timeout: u64) -> Result<Self, Error> {
        let proxy_stream = timeout(
            Duration::from_secs(connect_timeout),
            TcpStream::connect(user_info.proxy_servers()),
        )
        .await
        .map_err(|_| Error::ConnectTimeout(connect_timeout))??;
        Ok(Self {
            state: Initial {
                proxy_addr: proxy_stream.peer_addr()?,
                proxy_stream,
                user_info,
            },
        })
    }

    pub async fn handshake(mut self) -> Result<ProxyConnection<HandshakeReady>, Error> {
        let mut handshake_framed = Framed::new(
            &mut self.state.proxy_stream,
            SecureLengthDelimitedCodec::new(
                Cow::Borrowed(get_handshake_encryption()),
                Cow::Borrowed(get_handshake_encryption()),
            ),
        );
        let agent_encryption = random_generate_encryption();
        let rsa_encrypted_agent_encryption = rsa_encrypt_encryption(
            &agent_encryption,
            self.state
                .user_info
                .rsa_crypto()
                .ok_or(Error::UserRsaCryptoNotExist(
                    self.state.user_info.username().to_owned(),
                ))?,
        )?;
        let client_handshake = ClientHandshake {
            username: self.state.user_info.username().to_owned(),
            encryption: rsa_encrypted_agent_encryption.into_owned(),
        };
        let client_handshake_bytes =
            bincode::encode_to_vec(client_handshake, bincode::config::standard())?;
        handshake_framed.send(&client_handshake_bytes).await?;
        let proxy_handshake_bytes = handshake_framed
            .next()
            .await
            .ok_or(Error::ConnectionExhausted(self.state.proxy_addr))??;
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
                .ok_or(Error::UserRsaCryptoNotExist(
                    self.state.user_info.username().to_owned(),
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
    ) -> Result<ProxyConnection<DestinationReady<'a>>, Error> {
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
            .ok_or(Error::ConnectionExhausted(self.state.proxy_addr))??;
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
            ServerSetupDestination::Fail => Err(Error::SetupDestination(destination_addr)),
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
    ) -> Poll<Result<usize, StdIoError>> {
        let proxy_framed = &mut self.get_mut().state.proxy_framed;
        pin!(proxy_framed);
        proxy_framed.poll_write(cx, buf)
    }
    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), StdIoError>> {
        let proxy_framed = &mut self.get_mut().state.proxy_framed;
        pin!(proxy_framed);
        proxy_framed.poll_flush(cx)
    }
    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), StdIoError>> {
        let proxy_framed = &mut self.get_mut().state.proxy_framed;
        pin!(proxy_framed);
        proxy_framed.poll_shutdown(cx)
    }
}
