use common::Error as CommonError;
use protocol::Error as ProtocolError;
use thiserror::Error;
#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Toml(#[from] toml::de::Error),
    #[error(transparent)]
    Common(#[from] CommonError),
    #[error(transparent)]
    Protocol(#[from] ProtocolError),
}
