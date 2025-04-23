use std::net::SocketAddr;
pub trait CoreServerConfig {
    fn listening_address(&self) -> SocketAddr;
}
pub trait CoreRuntimeConfig {
    fn worker_threads(&self) -> usize;
}