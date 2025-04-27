use ppaass_2025_crypto::CryptoError;
use std::error::Error;
use thiserror::Error;
use tracing::metadata::ParseLevelError;
#[derive(Error, Debug)]
pub enum CoreError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    ParseLevel(#[from]ParseLevelError),
    #[error(transparent)]
    Crypto(#[from] CryptoError),
    #[error(transparent)]
    Other(#[from] Box<dyn Error>),
}