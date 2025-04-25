use ppaass_2025_core::{CoreRuntimeConfig, CoreServerConfig};
use ppaass_2025_user::{FileSystemUserRepositoryConfig, UserRepositoryConfig};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct ProxyConfig {
    listening_address: SocketAddr,
    worker_threads: usize,
    log_directory: PathBuf,
    log_name_prefix: String,
    max_log_level: String,
    user_repo_directory: PathBuf,
    user_repo_refresh_interval: u64,
    user_info_file_name: String,
    user_info_public_key_file_name: String,
    user_info_private_key_file_name: String,
}
impl ProxyConfig {
    pub fn log_directory(&self) -> &Path {
        &self.log_directory
    }
    pub fn log_name_prefix(&self) -> &str {
        &self.log_name_prefix
    }
    pub fn max_log_level(&self) -> &str {
        &self.max_log_level
    }
}
impl CoreServerConfig for ProxyConfig {
    fn listening_address(&self) -> SocketAddr {
        self.listening_address
    }
}
impl CoreRuntimeConfig for ProxyConfig {
    fn worker_threads(&self) -> usize {
        self.worker_threads
    }
}
impl UserRepositoryConfig for ProxyConfig {
    fn refresh_interval_sec(&self) -> u64 {
        self.user_repo_refresh_interval
    }
}
impl FileSystemUserRepositoryConfig for ProxyConfig {
    fn user_repo_directory(&self) -> &Path {
        &self.user_repo_directory
    }
    fn public_key_file_name(&self) -> &str {
        &self.user_info_public_key_file_name
    }
    fn private_key_file_name(&self) -> &str {
        &self.user_info_private_key_file_name
    }
    fn user_info_file_name(&self) -> &str {
        &self.user_info_file_name
    }
}