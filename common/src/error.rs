use ppaass_2025_crypto::CryptoError;
use std::error::Error;
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
    #[error(transparent)]
    Other(#[from] Box<dyn Error>),
}
impl From<BaseError> for std::io::Error {
    fn from(value: BaseError) -> Self {
        std::io::Error::new(std::io::ErrorKind::Other, format!("{value:?}"))
    }
}
