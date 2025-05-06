use std::net::SocketAddr;
use std::path::Path;
pub trait ServerConfig {
    fn listening_address(&self) -> SocketAddr;
}
pub trait BaseRuntimeConfig {
    fn worker_threads(&self) -> usize;
}
pub trait LogConfig {
    fn log_directory(&self) -> &Path;
    fn log_name_prefix(&self) -> &str;
    fn max_log_level(&self) -> &str;
}
