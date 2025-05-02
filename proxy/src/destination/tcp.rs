use crate::error::ProxyError;
use ppaass_2025_protocol::UnifiedAddress;
use std::io::Error;
use std::net::SocketAddr;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio::net::TcpStream;
use tokio::pin;
pub struct TcpDestEndpoint {
    tcp_stream: TcpStream,
    dst_addr: SocketAddr,
}
impl TcpDestEndpoint {
    pub async fn connect(unified_dst_addr: UnifiedAddress) -> Result<Self, ProxyError> {
        let dst_addrs: Vec<SocketAddr> = unified_dst_addr.try_into()?;
        let tcp_stream = TcpStream::connect(&dst_addrs[..]).await?;
        let dst_addr = tcp_stream.peer_addr()?;
        Ok(Self {
            dst_addr,
            tcp_stream,
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
        let tcp_stream = &mut self.get_mut().tcp_stream;
        pin!(tcp_stream);
        tcp_stream.poll_read(cx, buf)
    }
}
impl AsyncWrite for TcpDestEndpoint {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, Error>> {
        let tcp_stream = &mut self.get_mut().tcp_stream;
        pin!(tcp_stream);
        tcp_stream.poll_write(cx, buf)
    }
    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
        let tcp_stream = &mut self.get_mut().tcp_stream;
        pin!(tcp_stream);
        tcp_stream.poll_flush(cx)
    }
    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
        let tcp_stream = &mut self.get_mut().tcp_stream;
        pin!(tcp_stream);
        tcp_stream.poll_shutdown(cx)
    }
}
