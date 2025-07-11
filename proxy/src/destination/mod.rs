use crate::destination::tcp::TcpDestEndpoint;
use crate::destination::udp::UdpDestEndpoint;
use common::proxy::{DestinationReady, ProxyConnection};
use protocol::UnifiedAddress;
pub(crate) mod tcp;
pub(crate) mod udp;
pub enum Destination<'a> {
    Tcp(TcpDestEndpoint),
    Forward(Box<ProxyConnection<DestinationReady<'a>>>),
    #[allow(unused)]
    Udp {
        dst_udp_endpoint: UdpDestEndpoint,
        dst_addr: UnifiedAddress,
    },
}
