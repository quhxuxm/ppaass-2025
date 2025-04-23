use ppaass_2025_core::CoreError;
use thiserror::Error;
#[derive(Error, Debug)]
pub enum ProxyError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Toml(#[from] toml::de::Error),
    #[error(transparent)]
    Core(#[from] CoreError),
}