use crate::error::ProxyError;
use futures_util::stream::Map;
use futures_util::{Sink, StreamExt};
use ppaass_2025_protocol::UnifiedAddress;
use std::io::Error;
use std::net::SocketAddr;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio::net::TcpStream;
use tokio::pin;
use tokio_util::bytes::BytesMut;
use tokio_util::codec::{BytesCodec, Framed};
use tokio_util::io::{SinkWriter, StreamReader};
pub struct TcpDestEndpoint<'a> {
    dst_read_write: SinkWriter<
        StreamReader<
            Map<
                Framed<TcpStream, BytesCodec>,
                fn(Result<BytesMut, Error>) -> Result<&'a [u8], Error>,
            >,
            &'a [u8],
        >,
    >,
    dst_addr: SocketAddr,
}
impl<'a> TcpDestEndpoint<'a> {
    pub async fn connect(unified_dst_addr: UnifiedAddress) -> Result<Self, ProxyError> {
        let dst_addrs: Vec<SocketAddr> = unified_dst_addr.try_into()?;
        let tcp_stream = TcpStream::connect(&dst_addrs[..]).await?;
        let dst_addr = tcp_stream.peer_addr()?;
        let dst_framed = Framed::new(tcp_stream, BytesCodec::new()).map(|item| Ok(&item?[..]));
        let dst_read_write = SinkWriter::new(StreamReader::new(dst_framed));
        Ok(Self {
            dst_addr,
            dst_read_write,
        })
    }
    pub fn dst_addr(&self) -> SocketAddr {
        self.dst_addr
    }
}
impl AsyncRead for TcpDestEndpoint {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        let dst_read_write = &mut self.get_mut().dst_read_write;
        pin!(dst_read_write);
        dst_read_write.poll_read(cx, buf)
    }
}
impl AsyncWrite for TcpDestEndpoint {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, Error>> {
        let dst_read_write = &mut self.get_mut().dst_read_write;
        pin!(dst_read_write);
        todo!()
    }
    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
        let dst_read_write = &mut self.get_mut().dst_read_write;
        pin!(dst_read_write);
        dst_read_write.poll_
    }
    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
        todo!()
    }
}
