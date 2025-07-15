use crate::destination::tcp::TcpDestEndpoint;
use crate::destination::udp::UdpDestEndpoint;
use common::proxy::{ProxyConnection, ProxyFramedReaderWriter};
use protocol::UnifiedAddress;
pub(crate) mod tcp;
pub(crate) mod udp;

/// Define the destination type in proxy side
pub enum Destination<'a> {
    /// The TCP destination, the agent data will send
    /// to TCP destination directly.
    Tcp(TcpDestEndpoint),
    /// The forward destination, the agent data will forward
    /// to the remote proxy through current proxy node.
    Forward(Box<ProxyConnection<ProxyFramedReaderWriter<'a>>>),
    /// The UDP destination
    #[allow(unused)]
    Udp {
        dst_udp_endpoint: UdpDestEndpoint,
        dst_addr: UnifiedAddress,
    },
}
