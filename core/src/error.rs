use thiserror::Error;
use tracing::metadata::ParseLevelError;
#[derive(Error, Debug)]
pub enum CoreError {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    ParseLevel(#[from]ParseLevelError),
}