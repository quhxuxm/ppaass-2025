use ppaass_2025_core::CoreError;
use ppaass_2025_protocol::ProtocolError;
use std::net::SocketAddr;
use thiserror::Error;
#[derive(Error, Debug)]
pub enum ProxyError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Toml(#[from] toml::de::Error),
    #[error(transparent)]
    Core(#[from] CoreError),
    #[error("Client connection exhausted: [{0}]")]
    ClientConnectionExhausted(SocketAddr),
    #[error(transparent)]
    Decode(#[from] bincode::error::DecodeError),
    #[error(transparent)]
    Encode(#[from] bincode::error::EncodeError),
    #[error("Rsa crypto not existing for user: [{0}]")]
    RsaCryptoNotExist(String),
    #[error("Client user not exist: [{0}]")]
    ClientUserNotExist(String),
    #[error(transparent)]
    Protocol(#[from]ProtocolError),
}