use cipher::InvalidLength;
use thiserror::Error;
#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    InvalidLength(#[from] InvalidLength),
    #[error(transparent)]
    Unpad(#[from] block_padding::UnpadError),
    #[error(transparent)]
    Rsa(#[from] rsa::errors::Error),
    #[error(transparent)]
    Spki(#[from] spki::Error),
    #[error(transparent)]
    Pkcs8(#[from] pkcs8::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}
