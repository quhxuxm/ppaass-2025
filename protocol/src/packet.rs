use crate::address::UnifiedAddress;
use bincode::{Decode, Encode};
#[derive(Debug, Encode, Decode, Clone)]
pub enum Encryption {
    Plain,
    Aes(Vec<u8>),
    Blowfish(Vec<u8>),
}
#[derive(Debug, Encode, Decode)]
pub struct ClientHandshake {
    pub username: String,
    pub encryption: Encryption,
}
#[derive(Debug, Encode, Decode)]
pub struct ServerHandshake {
    pub encryption: Encryption,
}
#[derive(Debug, Encode, Decode)]
pub enum ClientSetupDestination {
    Tcp(UnifiedAddress),
    Udp(UnifiedAddress),
}
#[derive(Debug, Encode, Decode)]
pub enum ServerSetupDestination {
    Success,
    Fail,
}
#[derive(Debug, Encode, Decode)]
pub enum Relay {
    Tcp(Vec<u8>),
    Udp {
        src_addr: UnifiedAddress,
        dst_addr: UnifiedAddress,
        payload: Vec<u8>,
    },
}
