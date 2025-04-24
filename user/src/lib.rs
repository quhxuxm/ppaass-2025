mod config;
mod error;
mod repo;
use chrono::{DateTime, Utc};
pub use config::FileSystemUserRepositoryConfig;
pub use config::UserRepositoryConfig;
pub use error::UserError;
use ppaass_2025_crypto::RsaCrypto;
pub use repo::fs::FileSystemUserRepository;
pub use repo::UserRepository;
/// The user information
pub trait UserInfo {
    /// Create the user info with rsa crypto
    fn new(rsa_crypto: RsaCrypto) -> Self;
    /// The username
    fn username(&self) -> &str;
    /// The expired time
    fn expired_time(&self) -> Option<&DateTime<Utc>>;
    /// The rsa crypto
    fn rsa_crypto(&self) -> &RsaCrypto;
}
