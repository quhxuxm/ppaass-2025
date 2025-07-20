use crate::command::CommandArgs;
use clap::Parser;
use common_macro::{
    FileSystemUserRepoConfig, LogConfig, ServerConfig,  UserRepositoryConfig,
    UsernameConfig,
};
use core::panic;
use serde::{Deserialize, Serialize};
use std::fs::read_to_string;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::OnceLock;
const DEFAULT_CONFIG_FILE: &str = "./resources/proxy.toml";
static CONFIG: OnceLock<Config> = OnceLock::new();
pub fn get_config() -> &'static Config {
    CONFIG.get_or_init(|| {
        let command_line = CommandArgs::parse();
        let config_content = match &command_line.config_file_path {
            None => read_to_string(DEFAULT_CONFIG_FILE).unwrap_or_else(|_| {
                panic!(
                    "Fail to read proxy configuration file content from: {:?}",
                    DEFAULT_CONFIG_FILE
                )
            }),
            Some(path) => read_to_string(path).unwrap_or_else(|_| {
                panic!(
                    "Fail to read proxy configuration file content from: {:?}",
                    path
                )
            }),
        };
        let mut config = toml::from_str::<Config>(&config_content)
            .expect("Fail to initialize proxy configuration");
        config.merge_command_args(command_line);
        config
    })
}
#[derive(
    Debug,
    Serialize,
    Deserialize,
    Clone,
    UsernameConfig,
    UserRepositoryConfig,
    FileSystemUserRepoConfig,
)]
pub(crate) struct ForwardConfig {
    #[serde(default = "default_forward_proxy_connect_timeout")]
    proxy_connect_timeout: u64,
    #[serde(default = "default_forward_user_info_file_name")]
    user_info_file_name: String,
    #[serde(default = "default_forward_user_info_private_key_file_name")]
    user_info_private_key_file_name: String,
    #[serde(default = "default_forward_user_info_public_key_file_name")]
    user_info_public_key_file_name: String,
    #[serde(default = "default_forward_user_repo_directory")]
    user_repo_directory: PathBuf,
    #[serde(default = "default_forward_user_repo_refresh_interval")]
    user_repo_refresh_interval: u64,
    username: String,
}
impl ForwardConfig {
    pub fn proxy_connect_timeout(&self) -> u64 {
        self.proxy_connect_timeout
    }
}
#[derive(
    Debug,
    Serialize,
    Deserialize,
    ServerConfig,
    LogConfig,
    UserRepositoryConfig,
    FileSystemUserRepoConfig,
)]
pub(crate) struct Config {
    #[serde(default = "default_listening_address")]
    listening_address: SocketAddr,
    #[serde(default = "default_client_max_connections")]
    client_max_connections: usize,
    #[serde(default = "default_worker_thread")]
    worker_threads: usize,
    #[serde(default = "default_log_directory")]
    log_directory: PathBuf,
    #[serde(default = "default_log_name_prefix")]
    log_name_prefix: String,
    #[serde(default = "default_max_log_level")]
    max_log_level: String,
    #[serde(default = "default_user_repo_directory")]
    user_repo_directory: PathBuf,
    #[serde(default = "default_user_repo_refresh_interval")]
    user_repo_refresh_interval: u64,
    #[serde(default = "default_user_info_file_name")]
    user_info_file_name: String,
    #[serde(default = "default_user_info_public_key_file_name")]
    user_info_public_key_file_name: String,
    #[serde(default = "default_user_info_private_key_file_name")]
    user_info_private_key_file_name: String,
    #[serde(default = "default_destination_connect_timeout")]
    destination_connect_timeout: u64,
    forward: Option<ForwardConfig>,
}
impl Config {
    pub fn destination_connect_timeout(&self) -> u64 {
        self.destination_connect_timeout
    }
    pub fn merge_command_args(&mut self, command: CommandArgs) {
        if let Some(listening_address) = command.listening_address {
            self.listening_address = listening_address;
        }
        if let Some(worker_threads) = command.worker_threads {
            self.worker_threads = worker_threads;
        }
        if let Some(log_directory) = command.log_directory {
            self.log_directory = log_directory;
        }
        if let Some(max_log_level) = command.max_log_level {
            self.max_log_level = max_log_level;
        }
        if let Some(user_repo_directory) = command.user_repo_directory {
            self.user_repo_directory = user_repo_directory;
        }
        if let Some(user_repo_refresh_interval) = command.user_repo_refresh_interval {
            self.user_repo_refresh_interval = user_repo_refresh_interval;
        }
    }
    pub fn forward(&self) -> Option<&ForwardConfig> {
        self.forward.as_ref()
    }
}
fn default_listening_address() -> SocketAddr {
    SocketAddr::from_str("0.0.0.0:80").expect("Wrong default listening address")
}
fn default_worker_thread() -> usize {
    256
}
fn default_log_directory() -> PathBuf {
    PathBuf::from_str("./logs").expect("Wrong default log directory")
}
fn default_log_name_prefix() -> String {
    "ppaass-proxy.log".to_string()
}
fn default_max_log_level() -> String {
    "error".to_string()
}
fn default_user_repo_directory() -> PathBuf {
    PathBuf::from_str("./resources/proxy/user").expect("Wrong user repository directory")
}
fn default_user_repo_refresh_interval() -> u64 {
    10
}
fn default_user_info_file_name() -> String {
    "user_info.toml".to_string()
}
fn default_user_info_public_key_file_name() -> String {
    "AgentPublicKey.pem".to_string()
}
fn default_user_info_private_key_file_name() -> String {
    "ProxyPublicKey.pem".to_string()
}
fn default_forward_user_repo_directory() -> PathBuf {
    PathBuf::from_str("./resources/proxy/forward_user")
        .expect("Wrong forward user repository directory")
}
fn default_forward_user_repo_refresh_interval() -> u64 {
    10
}
fn default_forward_user_info_file_name() -> String {
    "user_info.toml".to_string()
}
fn default_forward_user_info_public_key_file_name() -> String {
    "ProxyPublicKey.pem".to_string()
}
fn default_forward_user_info_private_key_file_name() -> String {
    "AgentPublicKey.pem".to_string()
}
fn default_forward_proxy_connect_timeout() -> u64 {
    10
}
fn default_destination_connect_timeout() -> u64 {
    10
}
fn default_client_max_connections() -> usize {
    1024
}
