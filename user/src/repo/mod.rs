pub mod fs;
use crate::{UserError, UserInfo, UserRepositoryConfig};
use async_trait::async_trait;
use std::sync::Arc;
/// The user repository
#[async_trait]
pub trait UserRepository
where
    Self: Send + Sync + Sized + 'static,
{
    type UserInfoType: UserInfo + Send + Sync + 'static;
    type UserRepoConfigType: UserRepositoryConfig + Send + Sync + 'static;
    /// Create a user repository
    async fn new(config: Arc<Self::UserRepoConfigType>) -> Result<Self, UserError>;
    /// Find the user by username
    async fn find_user(&self, username: &str) -> Option<Arc<Self::UserInfoType>>;
    /// List all the users
    async fn list_users(&self) -> Vec<Arc<Self::UserInfoType>>;
    /// Save a user into the repository
    async fn save_user(&self, user: Self::UserInfoType);
}