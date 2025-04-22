use std::net::SocketAddr;
use bincode::{Decode, Encode};
use crate::error::ProtocolError;
/// The relay payload
///
/// - Tcp: The relay data.
/// - Udp: The udp relay data.
#[derive(Debug, Encode, Decode)]
pub enum RelayPayload{
    Tcp(Vec<u8>),
    Udp{
        src_addr: SocketAddr,
        dst_addr: SocketAddr,
        payload: Vec<u8>,
    }
}

/// The target endpoint.
///
/// - Tcp: The tcp endpoint.
/// - Udp: The udp endpoint.
#[derive(Debug, Encode, Decode)]
pub enum TargetEndpoint{
    Tcp{
        dst_addr: SocketAddr,
    },
    Udp{
        src_addr: SocketAddr,
        dst_addr: SocketAddr,
    }
}

/// The data packet transfer between agent
/// and proxy.
///
/// - InitConnection: Init the connection between agent and proxy, without target information, these connections can be pooled.
/// - SetupTargetEndpoint: Setup the connection to target endpoint
/// - Relay: Relay the data between agent and target endpoint
#[derive(Debug, Encode, Decode)]
pub enum DataPacket{
    InitConnection{
        username: String
    },
    SetupTargetEndpoint(TargetEndpoint),
    Relay(RelayPayload)
}

impl TryFrom<Vec<u8>> for DataPacket{
    type Error = ProtocolError;
    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        let (result, size) =bincode::decode_from_slice(&value,bincode::config::standard())?;
        Ok(result)
    }
}

impl TryFrom<DataPacket> for Vec<u8>{
    type Error = ProtocolError;
    fn try_from(value: DataPacket) -> Result<Self, Self::Error> {
        let result = bincode::encode_to_vec(value, bincode::config::standard())?;
        Ok(result)
    }
}
