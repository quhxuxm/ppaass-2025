mod config;
mod error;
mod repo;
use chrono::{DateTime, Utc};
pub use config::FileSystemUserRepositoryConfig;
pub use config::UserRepositoryConfig;
pub use error::UserError;
use ppaass_2025_crypto::RsaCrypto;
pub use repo::UserRepository;
pub use repo::fs::FileSystemUserRepository;
/// The user information
pub trait UserInfo {
    /// The username
    fn username(&self) -> &str;
    /// The expired time
    fn expired_time(&self) -> Option<&DateTime<Utc>>;
    /// The rsa crypto
    fn rsa_crypto(&self) -> Option<&RsaCrypto>;
    fn attach_rsa_crypto(&mut self, rsa_crypto: RsaCrypto);
}
