use ppaass_2025_core::SecureLengthDelimitedCodec;
use ppaass_2025_protocol::Encryption;
use std::io::Error;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio::net::TcpStream;
use tokio::pin;
use tokio_util::bytes::BytesMut;
use tokio_util::codec::Framed;
use tokio_util::io::{SinkWriter, StreamReader};
pub struct ClientTcpRelayEndpoint {
    client_read_write:
        SinkWriter<StreamReader<Framed<TcpStream, SecureLengthDelimitedCodec>, BytesMut>>,
}
impl ClientTcpRelayEndpoint {
    pub fn new(
        client_stream: TcpStream,
        client_encryption: Arc<Encryption>,
        server_encryption: Arc<Encryption>,
    ) -> Self {
        let client_framed = Framed::new(
            client_stream,
            SecureLengthDelimitedCodec::new(client_encryption, server_encryption),
        );
        Self {
            client_read_write: SinkWriter::new(StreamReader::new(client_framed)),
        }
    }
}

impl AsyncRead for ClientTcpRelayEndpoint {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        let client_read_write = &mut self.get_mut().client_read_write;
        pin!(client_read_write);
        client_read_write.poll_read(cx, buf)
    }
}
impl AsyncWrite for ClientTcpRelayEndpoint {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, Error>> {
        let client_read_write = &mut self.get_mut().client_read_write;
        pin!(client_read_write);
        client_read_write.poll_write(cx, buf)
    }
    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
        let client_read_write = &mut self.get_mut().client_read_write;
        pin!(client_read_write);
        client_read_write.poll_flush(cx)
    }
    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
        let client_read_write = &mut self.get_mut().client_read_write;
        pin!(client_read_write);
        client_read_write.poll_shutdown(cx)
    }
}
