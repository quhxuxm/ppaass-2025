use crate::address::UnifiedAddress;
use bincode::{Decode, Encode};

/// The encryption type
#[derive(Debug, Encode, Decode, Clone)]
pub enum Encryption {
    /// No encryption happens
    Plain,
    /// Aes encryption happens
    Aes(Vec<u8>),
    /// Blowfish encryption happens
    Blowfish(Vec<u8>),
}

/// The client side handshake message
#[derive(Debug, Encode, Decode)]
pub struct ClientHandshake {
    /// The username of the client
    pub username: String,
    /// The encryption of the client
    pub encryption: Encryption,
}

/// The server side handshake message
#[derive(Debug, Encode, Decode)]
pub struct ServerHandshake {
    /// The encryption of the server
    pub encryption: Encryption,
}

/// The client side destination setup message
#[derive(Debug, Encode, Decode)]
pub enum ClientSetupDestination {
    /// TCP destination
    Tcp(UnifiedAddress),
    /// UDP destination
    Udp(UnifiedAddress),
}

/// The server side destination setup result
#[derive(Debug, Encode, Decode)]
pub enum ServerSetupDestination {
    Success,
    Fail,
}

/// The relay data between client and server
#[derive(Debug, Encode, Decode)]
pub enum Relay {
    /// The TCP data
    Tcp(Vec<u8>),
    /// The UDP data
    Udp {
        src_addr: UnifiedAddress,
        dst_addr: UnifiedAddress,
        payload: Vec<u8>,
    },
}
