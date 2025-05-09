use crate::command::ProxyCommandArgs;
use clap::Parser;
use ppaass_2025_common::config::{
    FileSystemUserRepositoryConfig, ProxyUserConfig, UserRepositoryConfig,
};
use ppaass_2025_common::{LogConfig, RuntimeConfig, ServerConfig};
use serde::{Deserialize, Serialize};
use std::fs::read_to_string;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::OnceLock;
const DEFAULT_PROXY_CONFIG_FILE: &str = "./resources/proxy.toml";
static PROXY_CONFIG: OnceLock<ProxyConfig> = OnceLock::new();

pub fn get_proxy_config() -> &'static ProxyConfig {
    PROXY_CONFIG.get_or_init(|| {
        let command_line = ProxyCommandArgs::parse();
        let proxy_config_content = match &command_line.config_file_path {
            None => read_to_string(DEFAULT_PROXY_CONFIG_FILE).expect(&format!(
                "Fail to read proxy configuration file content from: {:?}",
                DEFAULT_PROXY_CONFIG_FILE
            )),
            Some(path) => read_to_string(path).expect(&format!(
                "Fail to read proxy configuration file content from: {:?}",
                path
            )),
        };
        let mut proxy_config = toml::from_str::<ProxyConfig>(&proxy_config_content)
            .expect("Fail to initialize proxy configuration");
        proxy_config.merge_command_args(command_line);
        proxy_config
    })
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct ForwardConfig {
    username: String,
    #[serde(default = "default_forward_user_repo_directory")]
    user_repo_directory: PathBuf,
    #[serde(default = "default_forward_user_repo_refresh_interval")]
    user_repo_refresh_interval: u64,
    #[serde(default = "default_forward_user_info_file_name")]
    user_info_file_name: String,
    #[serde(default = "default_forward_user_info_public_key_file_name")]
    user_info_public_key_file_name: String,
    #[serde(default = "default_forward_user_info_private_key_file_name")]
    user_info_private_key_file_name: String,
}

impl ProxyUserConfig for ForwardConfig {
    fn username(&self) -> &str {
        &self.username
    }
}
impl UserRepositoryConfig for ForwardConfig {
    fn refresh_interval_sec(&self) -> u64 {
        self.user_repo_refresh_interval
    }
}

impl FileSystemUserRepositoryConfig for ForwardConfig {
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

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct ProxyConfig {
    #[serde(default = "default_listening_address")]
    listening_address: SocketAddr,
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
    forward: Option<ForwardConfig>,
}
impl ProxyConfig {
    pub fn merge_command_args(&mut self, command: ProxyCommandArgs) {
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
impl LogConfig for ProxyConfig {
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
impl ServerConfig for ProxyConfig {
    fn listening_address(&self) -> SocketAddr {
        self.listening_address
    }
}
impl RuntimeConfig for ProxyConfig {
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
