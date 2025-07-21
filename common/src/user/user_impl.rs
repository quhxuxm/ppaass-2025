use crate::user::repo::UserActiveProxyAddressRepo;
use chrono::{DateTime, Utc};
use crypto::RsaCrypto;
use std::net::SocketAddr;
/// The base user
pub trait User {
    /// The username
    fn username(&self) -> &str;
    /// Get the rsa crypto of the user
    fn rsa_crypto(&self) -> Option<&RsaCrypto>;
    /// Attach the rsa crypto to user
    fn set_rsa_crypto(&mut self, rsa_crypto: RsaCrypto);
}

/// The user with expired time
pub trait UserWithExpiredTime: User {
    /// The expired time
    fn expired_time(&self) -> Option<&DateTime<Utc>>;
}

/// The user with proxy servers
pub trait UserWithProxyServers: User {
    /// The proxy server addresses
    fn proxy_servers(&self) -> &[SocketAddr];

    fn set_active_proxy_addr_repo(
        &mut self,
        user_active_proxy_address_repo: UserActiveProxyAddressRepo,
    );
}
