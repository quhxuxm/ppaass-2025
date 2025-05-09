use common::Error as CommonError;
use protocol::Error as ProtocolError;
use std::net::SocketAddr;
use thiserror::Error;
#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Toml(#[from] toml::de::Error),
    #[error(transparent)]
    Common(#[from] CommonError),
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
    Protocol(#[from] ProtocolError),
}
impl From<Error> for std::io::Error {
    fn from(value: Error) -> Self {
        std::io::Error::new(std::io::ErrorKind::Other, format!("{value:?}"))
    }
}
