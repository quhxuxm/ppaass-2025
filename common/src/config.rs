use std::net::SocketAddr;
use std::path::Path;
pub trait WithServerConfig {
    fn listening_address(&self) -> SocketAddr;
}
pub trait WithServerRuntimeConfig {
    fn worker_threads(&self) -> usize;
}
pub trait WithLogConfig {
    fn log_directory(&self) -> &Path;
    fn log_name_prefix(&self) -> &str;
    fn max_log_level(&self) -> &str;
}

pub trait WithUserRepositoryConfig {
    fn refresh_interval_sec(&self) -> u64;
}

pub trait WithUsernameConfig {
    fn username(&self) -> &str;
}
pub trait WithFileSystemUserRepoConfig: WithUserRepositoryConfig {
    fn user_repo_directory(&self) -> &Path;
    fn public_key_file_name(&self) -> &str;
    fn private_key_file_name(&self) -> &str;
    fn user_info_file_name(&self) -> &str;
}
