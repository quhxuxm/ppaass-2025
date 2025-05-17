use common::Error as CommonError;
use fast_socks5::server::SocksServerError;
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
    #[error(transparent)]
    FastSocks(#[from] SocksServerError),
    #[error("No destination host: {0}")]
    NoDestinationHost(Uri),
}
