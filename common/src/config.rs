use std::net::SocketAddr;
use std::path::Path;
/// The configuration for the server.
pub trait WithServerConfig {
    /// The address the server is listening on.
    fn listening_address(&self) -> SocketAddr;
    /// The maximum number of connections the server can handle from clients.
    fn client_max_connections(&self) -> usize;
    /// The maximum number of worker threads the server will use.
    fn worker_threads(&self) -> usize;
}
/// The configuration for the server's logging.
pub trait WithLogConfig {
    /// The directory where the server logs will be stored.
    fn log_directory(&self) -> &Path;
    /// The prefix for the log file names.
    fn log_name_prefix(&self) -> &str;
    /// The maximum log level for the server.
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
