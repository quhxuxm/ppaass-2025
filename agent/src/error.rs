use ppaass_2025_common::CoreError;
use ppaass_2025_protocol::UnifiedAddress;
use std::net::SocketAddr;
use thiserror::Error;
#[derive(Error, Debug)]
pub enum AgentError {
    #[error(transparent)]
    Core(#[from] CoreError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Hyper(#[from] hyper::Error),
    #[error(transparent)]
    Encode(#[from] bincode::error::EncodeError),
    #[error(transparent)]
    Decode(#[from] bincode::error::DecodeError),
    #[error("Proxy connection state invalid")]
    ProxyConnectionStateInvalid,
    #[error("Proxy connection exhausted: [{0}]")]
    ProxyConnectionExhausted(SocketAddr),
    #[error("Proxy connection fail to setup destination: [{0}]")]
    ProxyConnectionSetupDestination(UnifiedAddress),
    #[error("User not exist: [{0}]")]
    UserNotExist(String),
    #[error("{0}")]
    Other(String),
}

impl From<AgentError> for std::io::Error {
    fn from(value: AgentError) -> Self {
        std::io::Error::new(std::io::ErrorKind::Other, format!("{value:?}"))
    }
}
