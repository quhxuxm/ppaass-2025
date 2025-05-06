use ppaass_2025_crypto::CryptoError;
use ppaass_2025_protocol::UnifiedAddress;
use std::error::Error;
use std::net::SocketAddr;
use thiserror::Error;
use tracing::metadata::ParseLevelError;
#[derive(Error, Debug)]
pub enum BaseError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    ParseLevel(#[from] ParseLevelError),
    #[error(transparent)]
    Crypto(#[from] CryptoError),
    #[error("User not exist: [{0}]")]
    UserNotExist(String),
    #[error("User rsa crypto not exist: [{0}]")]
    UserRsaCryptoNotExist(String),
    #[error("Proxy connection exhausted: [{0}]")]
    ProxyConnectionExhausted(SocketAddr),
    #[error("Proxy connection fail to setup destination: [{0}]")]
    ProxyConnectionSetupDestination(UnifiedAddress),
    #[error(transparent)]
    Encode(#[from] bincode::error::EncodeError),
    #[error(transparent)]
    Decode(#[from] bincode::error::DecodeError),
    #[error(transparent)]
    Other(#[from] Box<dyn Error>),
}
impl From<BaseError> for std::io::Error {
    fn from(value: BaseError) -> Self {
        std::io::Error::new(std::io::ErrorKind::Other, format!("{value:?}"))
    }
}
