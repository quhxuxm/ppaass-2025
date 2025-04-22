use thiserror::Error;
#[derive(Error, Debug)]
pub enum CoreError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
}