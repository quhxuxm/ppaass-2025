use crate::destination::tcp::TcpDestEndpoint;
use crate::destination::udp::UdpDestEndpoint;
use common::proxy::{DestinationReady, ProxyConnection};
pub(crate) mod tcp;
pub(crate) mod udp;
pub enum Destination<'a> {
    Tcp(TcpDestEndpoint),
    Forward(ProxyConnection<DestinationReady<'a>>),
    #[allow(unused)]
    Udp(UdpDestEndpoint),
}
