use crate::error::Error;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use tokio::net::{ToSocketAddrs, UdpSocket};
pub struct UdpDestEndpoint {
    udp_socket: UdpSocket,
}

impl UdpDestEndpoint {
    pub async fn bind() -> Result<Self, Error> {
        let udp_socket =
            UdpSocket::bind(SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 0)).await?;
        Ok(Self { udp_socket })
    }

    pub async fn replay_to<A: ToSocketAddrs>(
        &self,
        dst_addr: A,
        buf: &[u8],
    ) -> Result<Vec<u8>, Error> {
        self.udp_socket.send_to(buf, dst_addr).await?;
        let mut dst_udp_data = [0u8; 65536];
        self.udp_socket.recv(&mut dst_udp_data).await?;
        Ok(dst_udp_data.to_vec())
    }
}
