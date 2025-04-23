use ppaass_2025_core::{CoreRuntimeConfig, CoreServerConfig};
use ppaass_2025_user::{FileSystemUserRepositoryConfig, UserRepositoryConfig};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct ProxyConfig {
    listening_address: SocketAddr,
    worker_threads: usize,
    user_repo_directory: PathBuf,
    user_repo_refresh_interval: u64,
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
}