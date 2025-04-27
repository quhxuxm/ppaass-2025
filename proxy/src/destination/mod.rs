use crate::destination::tcp::TcpDestEndpoint;
use crate::destination::udp::UdpDestEndpoint;
pub(crate) mod tcp;
pub(crate) mod udp;
pub enum Destination {
    Tcp(TcpDestEndpoint),
    Udp(UdpDestEndpoint),
}