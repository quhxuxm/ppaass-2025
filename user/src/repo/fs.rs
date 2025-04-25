use crate::config::FileSystemUserRepositoryConfig;
use crate::repo::UserRepository;
use crate::{UserError, UserInfo};
use async_trait::async_trait;
use ppaass_2025_crypto::RsaCrypto;
use serde::de::DeserializeOwned;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::sleep;
use tracing::error;
pub struct FileSystemUserRepository<U, C>
where
    U: UserInfo + Send + Sync + 'static,
    C: FileSystemUserRepositoryConfig + Send + Sync + 'static,
{
    storage: Arc<RwLock<HashMap<String, Arc<U>>>>,
    _config_mark: PhantomData<C>,
}
impl<U, C> FileSystemUserRepository<U, C>
where
    U: UserInfo + Send + Sync + DeserializeOwned + 'static,
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
                    error!("Fail to read sub entry from user repo directory [{user_repo_directory_path:?}] because of error: {e:?}");
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
            let user_dir_path = sub_entry.path();
            let public_key_file_path = user_dir_path.join(config.public_key_file_name());
            let public_key_file = match std::fs::File::open(public_key_file_path) {
                Ok(public_key_file) => public_key_file,
                Err(e) => {
                    error!("Fail to read public key file: {e:?}");
                    continue;
                }
            };
            let private_key_file_path = user_dir_path.join(config.private_key_file_name());
            let private_key_file = match std::fs::File::open(private_key_file_path) {
                Ok(private_key_file) => private_key_file,
                Err(e) => {
                    error!("Fail to read private key file: {e:?}");
                    continue;
                }
            };
            let user_rsa_crypto = match RsaCrypto::new(public_key_file, private_key_file) {
                Ok(user_rsa_crypto) => user_rsa_crypto,
                Err(e) => {
                    error!("Fail to create user rsa crypto: {e:?}");
                    continue;
                }
            };
            let user_info_file_path = user_dir_path.join(config.user_info_file_name());
            let user_info_file_content = match tokio::fs::read_to_string(&user_info_file_path).await {
                Ok(content) => content,
                Err(e) => {
                    error!("Fail to read user info file content: {e:?}");
                    continue;
                }
            };
            let mut user_info = match toml::from_str::<U>(&user_info_file_content) {
                Ok(user_info) => user_info,
                Err(e) => {
                    error!("Fail to deserialize the user info: {e:?}");
                    continue;
                }
            };
            user_info.attach_rsa_crypto(user_rsa_crypto);
            let mut storage = storage.write().await;
            storage.insert(user_info.username().to_owned(), Arc::new(user_info));
        }
        Ok(())
    }
}
#[async_trait]
impl<U, C> UserRepository for FileSystemUserRepository<U, C>
where
    U: UserInfo + Send + Sync + DeserializeOwned + 'static,
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
        tokio::spawn(async move {
            loop {
                if let Err(e) = Self::fill_storage(config.clone(), storage_clone.clone()).await {
                    error!("Failed to fill user repository storage: {}", e);
                };
                sleep(Duration::from_secs(config.refresh_interval_sec())).await;
            }
        });
        Ok(Self {
            storage,
            _config_mark: Default::default(),
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