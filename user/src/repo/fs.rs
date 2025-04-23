use crate::config::FileSystemUserRepositoryConfig;
use crate::repo::UserRepository;
use crate::{UserError, UserInfo, UserRepositoryConfig};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::sleep;
use tracing::{debug, error};
pub struct FileSystemUserRepository<U, C>
where
    U: UserInfo + Send + Sync + 'static,
    C: FileSystemUserRepositoryConfig + Send + Sync + 'static,
{
    storage: Arc<RwLock<HashMap<String, Arc<U>>>>,
    config: Arc<C>,
}
impl<U, C> FileSystemUserRepository<U, C>
where
    U: UserInfo + Send + Sync + 'static,
    C: FileSystemUserRepositoryConfig + Send + Sync + 'static,
{
    async fn fill_storage(config: Arc<C>, storage: Arc<RwLock<HashMap<String, Arc<U>>>>) -> Result<(), UserError>
    {
        let user_repo_directory_path = config.user_repo_directory();
        let mut user_repo_directory = tokio::fs::read_dir(user_repo_directory_path).await?;
        while let Some(sub_entry) = user_repo_directory.next_entry().await? {
            let file_type = match sub_entry.file_type().await {
                Ok(file_type) => file_type,
                Err(e) => {
                    error!("Fail to read sub entry from user repo directory: {user_repo_directory_path:?}");
                    continue;
                }
            };
            if !file_type.is_dir() {
                continue;
            }
            let file_name = sub_entry.file_name();
            let file_name = match file_name.to_str() {
                Some(file_name) => file_name,
                None => {
                    continue;
                }
            };
            if file_name.starts_with("\\.") {
                continue;
            }
            let user_dir = match tokio::fs::read_dir(sub_entry.path()).await {
                Ok(user_dir) => user_dir,
                Err(e) => {
                    error!("Fail to read user directory: {e:?}");
                    continue;
                }
            };
        }
        Ok(())
    }
}
#[async_trait]
impl<U, C> UserRepository for FileSystemUserRepository<U, C>
where
    U: UserInfo + Send + Sync + 'static,
    C: FileSystemUserRepositoryConfig + Send + Sync + 'static,
{
    type UserInfoType = U;
    type UserRepoConfigType = C;
    async fn new(config: Arc<Self::UserRepoConfigType>) -> Result<Self, UserError>
    {
        let storage = Arc::new(RwLock::new(HashMap::new()));
        if let Err(e) = Self::fill_storage(config.clone(), storage.clone()).await {
            error!("Failed to fill user repository storage: {}", e);
        };
        let storage_clone = storage.clone();
        let config_clone = config.clone();
        tokio::spawn(async move {
            loop {
                if let Err(e) = Self::fill_storage(config_clone.clone(), storage_clone.clone()).await {
                    error!("Failed to fill user repository storage: {}", e);
                };
                sleep(Duration::from_secs(config_clone.refresh_interval_sec())).await;
            }
        });
        Ok(Self {
            storage,
            config,
        })
    }
    async fn find_user(&self, username: &str) -> Option<Arc<Self::UserInfoType>> {
        let lock = self.storage.read().await;
        let user_info = lock.get(username)?;
        Some(user_info.clone())
    }
    async fn list_users(&self) -> Vec<Arc<Self::UserInfoType>> {
        let lock = self.storage.read().await;
        lock.values().map(|v| v.clone()).collect()
    }
    async fn save_user(&self, user: Self::UserInfoType) {
        let mut lock = self.storage.write().await;
        lock.insert(user.username().to_owned(), Arc::new(user));
    }
}