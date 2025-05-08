pub mod repo;
mod user;
use crate::BaseError;
use crate::config::UserRepositoryConfig;
use std::sync::Arc;
pub use user::*;
/// The user repository

pub trait UserRepository
where
    Self: Send + Sync + Sized + 'static,
{
    type UserInfoType: BasicUser + Send + Sync + 'static;
    type UserRepoConfigType: UserRepositoryConfig + Send + Sync + 'static;
    /// Create a user repository
    fn new(config: Arc<Self::UserRepoConfigType>) -> Result<Self, BaseError>;
    /// Find the user by username
    fn find_user(&self, username: &str) -> Option<Arc<Self::UserInfoType>>;
    /// List all the users
    fn list_users(&self) -> Vec<Arc<Self::UserInfoType>>;
    /// Save a user into the repository
    fn save_user(&self, user: Self::UserInfoType);
}
