use std::net::SocketAddr;
pub trait CoreServerConfig {
    fn listening_address(&self) -> SocketAddr;
}