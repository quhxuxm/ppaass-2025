use ppaass_2025_common::{CoreLogConfig, CoreRuntimeConfig, CoreServerConfig};
use ppaass_2025_protocol::Encryption;
use ppaass_2025_user::{FileSystemUserRepositoryConfig, UserRepositoryConfig};
use serde::{Deserialize, Serialize};
use std::fs::read_to_string;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::{Arc, LazyLock};
const AGENT_CONFIG_FILE: &str = "resource/proxy.toml";
pub static AGENT_CONFIG: LazyLock<AgentConfig> = LazyLock::new(|| {
    let proxy_config_content =
        read_to_string(AGENT_CONFIG_FILE).expect("Fail to read agent configuration file content");
    toml::from_str::<AgentConfig>(&proxy_config_content)
        .expect("Fail to initialize agent configuration")
});

#[derive(Serialize, Deserialize, Debug)]
pub struct AgentConfig {
    listening_address: SocketAddr,
    log_directory: PathBuf,
    log_name_prefix: String,
    max_log_level: String,
    worker_threads: usize,
    refresh_interval_sec: u64,
    user_repo_directory: PathBuf,
    user_info_file_name: String,
    user_info_public_key_file_name: String,
    user_info_private_key_file_name: String,
    handshake_encoder_encryption: String,
    handshake_decoder_encryption: String,
    username: String,
}

impl AgentConfig {
    pub fn username(&self) -> &str {
        &self.username
    }
}

impl CoreServerConfig for AgentConfig {
    fn listening_address(&self) -> SocketAddr {
        self.listening_address
    }
    fn handshake_encoder_encryption(&self) -> Arc<Encryption> {
        Arc::new(Encryption::Blowfish(
            self.handshake_encoder_encryption.as_bytes().to_vec(),
        ))
    }
    fn handshake_decoder_encryption(&self) -> Arc<Encryption> {
        Arc::new(Encryption::Blowfish(
            self.handshake_decoder_encryption.as_bytes().to_vec(),
        ))
    }
}

impl CoreLogConfig for AgentConfig {
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

impl CoreRuntimeConfig for AgentConfig {
    fn worker_threads(&self) -> usize {
        self.worker_threads
    }
}

impl UserRepositoryConfig for AgentConfig {
    fn refresh_interval_sec(&self) -> u64 {
        self.refresh_interval_sec
    }
}

impl FileSystemUserRepositoryConfig for AgentConfig {
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
