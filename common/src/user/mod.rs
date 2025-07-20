pub mod repo;
use crate::Error;
use crate::config::WithUserRepositoryConfig;
use std::ops::Deref;
use std::sync::Arc;
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
}
/// The user repository
pub trait UserRepository
where
    Self: Send + Sync + Sized + 'static,
{
    type UserInfoType: User + Send + Sync + 'static;
    type UserRepoConfigType: WithUserRepositoryConfig + Send + Sync + 'static;
    /// Create a user repository
    fn new<T>(config: T) -> Result<Self, Error>
    where
        T: Deref<Target = Self::UserRepoConfigType> + Send + Sync + 'static;
    /// Find the user by username
    fn find_user(&self, username: &str) -> Option<Arc<Self::UserInfoType>>;
    /// List all the users
    fn list_users(&self) -> Vec<Arc<Self::UserInfoType>>;
    /// Save a user into the repository
    fn save_user(&self, user: Self::UserInfoType);
}
