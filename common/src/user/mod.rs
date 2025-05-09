pub mod repo;
mod user;
use crate::config::UserRepositoryConfig;
use crate::Error;
use std::ops::Deref;
use std::sync::Arc;
pub use user::*;
/// The user repository

pub trait UserRepository
where
    Self: Send + Sync + Sized + 'static,
{
    type UserInfoType: User + Send + Sync + 'static;
    type UserRepoConfigType: UserRepositoryConfig + Send + Sync + 'static;
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
