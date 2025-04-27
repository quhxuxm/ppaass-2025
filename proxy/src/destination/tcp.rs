use crate::error::ProxyError;
use futures_util::{Sink, SinkExt, Stream, StreamExt};
use ppaass_2025_protocol::UnifiedAddress;
use std::net::SocketAddr;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::net::TcpStream;
use tokio_util::bytes::{Bytes, BytesMut};
use tokio_util::codec::{BytesCodec, Framed};
pub struct TcpDestEndpoint {
    dest_framed: Framed<TcpStream, BytesCodec>,
}
impl TcpDestEndpoint {
    pub async fn connect(unified_dst_addr: UnifiedAddress) -> Result<Self, ProxyError> {
        let dst_addrs: Vec<SocketAddr> = unified_dst_addr.try_into()?;
        let tcp_stream = TcpStream::connect(&dst_addrs[..]).await?;
        Ok(Self {
            dest_framed: Framed::new(tcp_stream, BytesCodec::new())
        })
    }
}
impl Stream for TcpDestEndpoint {
    type Item = Result<Bytes, ProxyError>;
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.get_mut().dest_framed.poll_next_unpin(cx).map(|item| {
            item.map(|item| item.map(|e| e.freeze()))
        }).map_err(|e| ProxyError::Io(e))
    }
}
impl Sink<&[u8]> for TcpDestEndpoint {
    type Error = ProxyError;
    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        SinkExt::<Bytes>::poll_ready_unpin(&mut self.get_mut().dest_framed, cx).map_err(|e| ProxyError::Io(e))
    }
    fn start_send(self: Pin<&mut Self>, item: &[u8]) -> Result<(), Self::Error> {
        self.get_mut().dest_framed.start_send_unpin(BytesMut::from(item)).map_err(|e| ProxyError::Io(e))
    }
    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        SinkExt::<Bytes>::poll_flush_unpin(&mut self.get_mut().dest_framed, cx)
            .map_err(Into::into)
    }
    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        SinkExt::<Bytes>::poll_close_unpin(&mut self.get_mut().dest_framed, cx)
            .map_err(Into::into)
    }
}