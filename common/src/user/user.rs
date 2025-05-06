use chrono::{DateTime, Utc};
use ppaass_2025_crypto::RsaCrypto;
use std::net::SocketAddr;
pub trait BasicUser {
    /// The username
    fn username(&self) -> &str;
    /// Get the rsa crypto of the user
    fn rsa_crypto(&self) -> Option<&RsaCrypto>;
    /// Add the rsa crypto to user
    fn attach_rsa_crypto(&mut self, rsa_crypto: RsaCrypto);
}

pub trait ExpiredUser: BasicUser {
    /// The expired time
    fn expired_time(&self) -> Option<&DateTime<Utc>>;
}

pub trait ProxyConnectionUser: BasicUser {
    /// The proxy server addresses
    fn proxy_servers(&self) -> &[SocketAddr];
}
