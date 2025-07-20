use crate::command::CommandArgs;
use clap::Parser;
use common_macro::{
    FileSystemUserRepoConfig, LogConfig, ServerConfig, UserRepositoryConfig,
    UsernameConfig,
};
use core::panic;
use serde::{Deserialize, Serialize};
use std::fs::read_to_string;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::OnceLock;
/// The default configuration file patch
const DEFAULT_CONFIG_FILE: &str = "./resources/agent.toml";
/// The global configuration object
static CONFIG: OnceLock<Config> = OnceLock::new();
/// Get the configuration
pub fn get_config() -> &'static Config {
    CONFIG.get_or_init(|| {
        let command_line = CommandArgs::parse();
        let config_content = match &command_line.config_file_path {
            None => read_to_string(DEFAULT_CONFIG_FILE).unwrap_or_else(|_| {
                panic!(
                    "Fail to read agent configuration file content from: {:?}",
                    DEFAULT_CONFIG_FILE
                )
            }),
            Some(path) => read_to_string(path).unwrap_or_else(|_| {
                panic!(
                    "Fail to read agent configuration file content from: {:?}",
                    path
                )
            }),
        };
        let mut config = toml::from_str::<Config>(&config_content)
            .expect("Fail to initialize agent configuration");
        config.merge_command_args(command_line);
        config
    })
}
/// The configuration object
#[derive(
    Serialize,
    Deserialize,
    Debug,
    ServerConfig,
    UsernameConfig,
    LogConfig,
    UserRepositoryConfig,
    FileSystemUserRepoConfig,
)]
pub struct Config {
    #[serde(default = "default_client_max_connections")]
    client_max_connections: usize,
    #[serde(default = "default_listening_address")]
    listening_address: SocketAddr,
    #[serde(default = "default_log_directory")]
    log_directory: PathBuf,
    #[serde(default = "default_log_name_prefix")]
    log_name_prefix: String,
    #[serde(default = "default_max_log_level")]
    max_log_level: String,
    #[serde(default = "default_proxy_connect_timeout")]
    proxy_connect_timeout: u64,
    #[serde(default = "default_user_info_file_name")]
    user_info_file_name: String,
    #[serde(default = "default_user_info_private_key_file_name")]
    user_info_private_key_file_name: String,
    #[serde(default = "default_user_info_public_key_file_name")]
    user_info_public_key_file_name: String,
    #[serde(default = "default_user_repo_directory")]
    user_repo_directory: PathBuf,
    #[serde(default = "default_user_repo_refresh_interval")]
    user_repo_refresh_interval: u64,
    #[serde(default = "default_username")]
    username: String,
    #[serde(default = "default_worker_thread")]
    worker_threads: usize,
}
impl Config {
    pub fn proxy_connect_timeout(&self) -> u64 {
        self.proxy_connect_timeout
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
        if let Some(username) = command.username {
            self.username = username;
        }
    }
}
/// The default agent listening address
fn default_listening_address() -> SocketAddr {
    SocketAddr::from_str("0.0.0.0:80").expect("Wrong default listening address")
}
/// The default worker thread number
fn default_worker_thread() -> usize {
    256
}
/// The default log file directory
fn default_log_directory() -> PathBuf {
    PathBuf::from_str("./logs").expect("Wrong default log directory")
}
/// The default log file name prefix
fn default_log_name_prefix() -> String {
    "ppaass-agent.log".to_string()
}
/// The default log level
fn default_max_log_level() -> String {
    "error".to_string()
}
/// The default directory used to story the user
/// information
fn default_user_repo_directory() -> PathBuf {
    PathBuf::from_str("./resources/agent/user").expect("Wrong user repository directory")
}
/// The default user repository refresh interval
fn default_user_repo_refresh_interval() -> u64 {
    10
}
/// The default user information file name
fn default_user_info_file_name() -> String {
    "user_info.toml".to_string()
}
/// The default proxy public key file name
fn default_user_info_public_key_file_name() -> String {
    "ProxyPublicKey.pem".to_string()
}
/// The default agent public key file name
fn default_user_info_private_key_file_name() -> String {
    "AgentPublicKey.pem".to_string()
}
/// The defaul username
fn default_username() -> String {
    "user1".to_string()
}
/// The default proxy connect timeout.
fn default_proxy_connect_timeout() -> u64 {
    10
}
/// The default max client connection number
/// If the incoming client connection exceed this
/// number, the client will waiting until there
/// is a connection available for it.
fn default_client_max_connections() -> usize {
    1024
}

