mod config;
mod error;
mod repo;
use chrono::{DateTime, Utc};
pub use config::FileSystemUserRepositoryConfig;
pub use config::UserRepositoryConfig;
pub use error::UserError;
pub use repo::fs::FileSystemUserRepository;
pub use repo::UserRepository;
/// The user information
pub trait UserInfo {
    /// The username
    fn username(&self) -> &str;
    /// The expired time
    fn expired_time(&self) -> Option<&DateTime<Utc>>;
}
