use ppaass_2025_common::{BaseServerConfig, CoreLogConfig, CoreRuntimeConfig};
use ppaass_2025_user::{FileSystemUserRepositoryConfig, UserRepositoryConfig};
use serde::{Deserialize, Serialize};
use std::fs::read_to_string;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;
const PROXY_CONFIG_FILE: &str = "resources/proxy.toml";
pub static PROXY_SERVER_CONFIG: LazyLock<ProxyConfig> = LazyLock::new(|| {
    let proxy_config_content =
        read_to_string(PROXY_CONFIG_FILE).expect("Fail to read proxy configuration file content");
    toml::from_str::<ProxyConfig>(&proxy_config_content)
        .expect("Fail to initialize proxy configuration")
});
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
impl CoreLogConfig for ProxyConfig {
    fn log_directory(&self) -> &Path {
        &self.log_directory
    }
    fn log_name_prefix(&self) -> &str {
        &self.log_name_prefix
    }
    fn max_log_level(&self) -> &str {
        &self.max_log_level
    }
}
impl BaseServerConfig for ProxyConfig {
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
