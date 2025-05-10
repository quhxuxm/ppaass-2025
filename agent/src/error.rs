use common::Error as CommonError;
use hyper::Uri;
use thiserror::Error;
#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Common(#[from] CommonError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Hyper(#[from] hyper::Error),
    #[error("No destination host: {0}")]
    NoDestinationHost(Uri),
}
