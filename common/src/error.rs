use crypto::Error as CryptoError;
use protocol::UnifiedAddress;
use thiserror::Error;
use tracing::metadata::ParseLevelError;
#[derive(Error, Debug)]
pub enum Error {
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
    #[error("Connection exhausted: [{0}]")]
    ConnectionExhausted(String),
    #[error("Fail to setup destination: [{0}]")]
    SetupDestination(UnifiedAddress),
    #[error(transparent)]
    Encode(#[from] bincode::error::EncodeError),
    #[error(transparent)]
    Decode(#[from] bincode::error::DecodeError),
    #[error("Connect to remote endpoint timeout in {0} seconds.")]
    ConnectTimeout(u64),
    #[error("Lock error: [{0}]")]
    Lock(String),
}
impl From<Error> for std::io::Error {
    fn from(value: Error) -> Self {
        std::io::Error::other(format!("{value:?}"))
    }
}
