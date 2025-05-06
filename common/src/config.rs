use std::net::SocketAddr;
use std::path::Path;
pub trait ServerConfig {
    fn listening_address(&self) -> SocketAddr;
}
pub trait RuntimeConfig {
    fn worker_threads(&self) -> usize;
}
pub trait LogConfig {
    fn log_directory(&self) -> &Path;
    fn log_name_prefix(&self) -> &str;
    fn max_log_level(&self) -> &str;
}

pub trait UserRepositoryConfig {
    fn refresh_interval_sec(&self) -> u64;
}
pub trait FileSystemUserRepositoryConfig: UserRepositoryConfig {
    fn user_repo_directory(&self) -> &Path;
    fn public_key_file_name(&self) -> &str;
    fn private_key_file_name(&self) -> &str;
    fn user_info_file_name(&self) -> &str;
}
