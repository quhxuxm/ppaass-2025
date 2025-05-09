use thiserror::Error;
#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("Failed to decode bytes to data packet: {0}")]
    Decode(#[from] bincode::error::DecodeError),
    #[error("Failed to encode data packet to bytes: {0}")]
    Encode(#[from] bincode::error::EncodeError),
    #[error("Fail to parse: {0}")]
    Parse(String),
}
