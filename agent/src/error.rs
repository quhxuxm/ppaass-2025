use hyper::Uri;
use ppaass_2025_common::BaseError;
use thiserror::Error;
#[derive(Error, Debug)]
pub enum AgentError {
    #[error(transparent)]
    Base(#[from] BaseError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Hyper(#[from] hyper::Error),
    #[error("Proxy connection state invalid")]
    ProxyConnectionStateInvalid,
    #[error("No destination host: {0}")]
    NoDestinationHost(Uri),
}

impl From<AgentError> for std::io::Error {
    fn from(value: AgentError) -> Self {
        std::io::Error::new(std::io::ErrorKind::Other, format!("{value:?}"))
    }
}
