use ppaass_2025_protocol::Encryption;
use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;
pub trait CoreServerConfig {
    fn listening_address(&self) -> SocketAddr;
    fn handshake_encoder_encryption(&self) -> Arc<Encryption>;
    fn handshake_decoder_encryption(&self) -> Arc<Encryption>;
}
pub trait CoreRuntimeConfig {
    fn worker_threads(&self) -> usize;
}
pub trait CoreLogConfig {
    fn log_directory(&self) -> &Path;
    fn log_name_prefix(&self) -> &str;
    fn max_log_level(&self) -> &str;
}
impl<C> CoreServerConfig for &C
where
    C: CoreServerConfig,
{
    fn listening_address(&self) -> SocketAddr {
        CoreServerConfig::listening_address(*self)
    }
    fn handshake_encoder_encryption(&self) -> Arc<Encryption> {
        CoreServerConfig::handshake_encoder_encryption(*self)
    }
    fn handshake_decoder_encryption(&self) -> Arc<Encryption> {
        CoreServerConfig::handshake_decoder_encryption(*self)
    }
}