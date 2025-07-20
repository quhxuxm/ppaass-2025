use crate::user::UserWithProxyServers;
use crate::{
    Error, SecureLengthDelimitedCodec, get_handshake_encryption, random_generate_encryption,
    rsa_decrypt_encryption, rsa_encrypt_encryption,
};
use bincode::config::Configuration;
use futures_util::{SinkExt, StreamExt};
use protocol::{
    ClientHandshake, ClientSetupDestination, ServerHandshake, ServerSetupDestination,
    UnifiedAddress,
};
use std::io::Error as StdIoError;
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
pub type ProxyFramed = Framed<TcpStream, SecureLengthDelimitedCodec>;
pub type ProxyFramedReaderWriter = SinkWriter<StreamReader<ProxyFramed, BytesMut>>;
pub enum DestinationType {
    Tcp,
    #[allow(unused)]
    Udp,
}
pub struct Init;
/// The proxy connection.
pub struct ProxyConnection<T> {
    state: T,
}
impl ProxyConnection<Init> {
    /// Create a new proxy connection
    pub async fn new<U>(
        user_info: Arc<U>,
        connect_timeout: u64,
    ) -> Result<ProxyConnection<ProxyFramed>, Error>
    where
        U: UserWithProxyServers + Send + Sync + 'static,
    {
        let mut proxy_stream = timeout(
            Duration::from_secs(connect_timeout),
            TcpStream::connect(user_info.proxy_servers()),
        )
            .await
            .map_err(|_| Error::ConnectTimeout(connect_timeout))??;
        let mut handshake_framed = Framed::new(
            &mut proxy_stream,
            SecureLengthDelimitedCodec::new(get_handshake_encryption(), get_handshake_encryption()),
        );
        let agent_encryption = random_generate_encryption();
        let rsa_encrypted_agent_encryption = rsa_encrypt_encryption(
            &agent_encryption,
            user_info.rsa_crypto().ok_or(Error::UserRsaCryptoNotExist(
                user_info.username().to_owned(),
            ))?,
        )?;
        let client_handshake = ClientHandshake {
            username: user_info.username().to_owned(),
            encryption: rsa_encrypted_agent_encryption.into_owned(),
        };
        let client_handshake_bytes =
            bincode::encode_to_vec(client_handshake, bincode::config::standard())?;
        handshake_framed.send(&client_handshake_bytes).await?;
        let proxy_handshake_bytes =
            handshake_framed
                .next()
                .await
                .ok_or(Error::ConnectionExhausted(format!(
                    "Fail to read handshke message from proxy: {}",
                    proxy_stream.peer_addr()?
                )))??;
        let (rsa_encrypted_proxy_handshake, _) =
            bincode::decode_from_slice::<ServerHandshake, Configuration>(
                &proxy_handshake_bytes,
                bincode::config::standard(),
            )?;
        let proxy_encryption = rsa_decrypt_encryption(
            rsa_encrypted_proxy_handshake.encryption,
            user_info.rsa_crypto().ok_or(Error::UserRsaCryptoNotExist(
                user_info.username().to_owned(),
            ))?,
        )?;
        let proxy_framed = Framed::new(
            proxy_stream,
            SecureLengthDelimitedCodec::new(Arc::new(proxy_encryption), Arc::new(agent_encryption)),
        );
        Ok(ProxyConnection {
            state: proxy_framed,
        })
    }
}
/// After handshake complete, the proxy connection can do
/// setup destination
impl ProxyConnection<ProxyFramed> {
    /// Setup the destination, in this process
    /// server side will build tcp connection with
    /// the destination.
    pub async fn setup_destination(
        self,
        destination_addr: UnifiedAddress,
        destination_type: DestinationType,
    ) -> Result<ProxyConnection<ProxyFramedReaderWriter>, Error> {
        let mut proxy_framed = self.state;
        let setup_destination = match destination_type {
            DestinationType::Tcp => ClientSetupDestination::Tcp(destination_addr.clone()),
            DestinationType::Udp => ClientSetupDestination::Udp(destination_addr.clone()),
        };
        let setup_destination_bytes =
            bincode::encode_to_vec(setup_destination, bincode::config::standard())?;
        proxy_framed.send(&setup_destination_bytes).await?;
        let proxy_setup_destination_bytes = proxy_framed
            .next()
            .await
            .ok_or(Error::ConnectionExhausted(format!("Fail to read setup destination connection message from proxy, destination address: {destination_addr:?}")))??;
        let (proxy_setup_destination, _) =
            bincode::decode_from_slice::<ServerSetupDestination, Configuration>(
                &proxy_setup_destination_bytes,
                bincode::config::standard(),
            )?;
        match proxy_setup_destination {
            ServerSetupDestination::Success => Ok(ProxyConnection {
                state: SinkWriter::new(StreamReader::new(proxy_framed)),
            }),
            ServerSetupDestination::Fail => Err(Error::SetupDestination(destination_addr)),
        }
    }
}
/// After setup destinition on proxy connection success,
/// the proxy connection will become reader & writer,
/// and this is the reader part.
impl AsyncRead for ProxyConnection<ProxyFramedReaderWriter> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        let proxy_framed = &mut self.get_mut().state;
        pin!(proxy_framed);
        proxy_framed.poll_read(cx, buf)
    }
}
/// After setup destinition on proxy connection success,
/// the proxy connection will become reader & writer,
/// and this is the writer part.
impl AsyncWrite for ProxyConnection<ProxyFramedReaderWriter> {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, StdIoError>> {
        let proxy_framed = &mut self.get_mut().state;
        pin!(proxy_framed);
        proxy_framed.poll_write(cx, buf)
    }
    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), StdIoError>> {
        let proxy_framed = &mut self.get_mut().state;
        pin!(proxy_framed);
        proxy_framed.poll_flush(cx)
    }
    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), StdIoError>> {
        let proxy_framed = &mut self.get_mut().state;
        pin!(proxy_framed);
        proxy_framed.poll_shutdown(cx)
    }
}
