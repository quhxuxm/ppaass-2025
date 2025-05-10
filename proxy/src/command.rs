use clap::Parser;
use std::net::SocketAddr;
use std::path::PathBuf;
/// The proxy server
#[derive(Parser)]
#[command(version, about, long_about = None)]
pub(crate) struct CommandArgs {
    /// The configuration file path
    #[arg(short = 'c', long)]
    pub config_file_path: Option<PathBuf>,
    /// The listening address of the proxy server
    #[arg(short = 'a', long)]
    pub listening_address: Option<SocketAddr>,
    /// The worker thread number
    #[arg(short = 't', long)]
    pub worker_threads: Option<usize>,
    /// The log directory path
    #[arg(short = 'l', long)]
    pub log_directory: Option<PathBuf>,
    /// The max log level
    #[arg(short = 'm', long)]
    pub max_log_level: Option<String>,
    /// The user repository directory
    #[arg(short = 'r', long)]
    pub user_repo_directory: Option<PathBuf>,
    /// The interval to refresh the user repository
    #[arg(short = 'i', long)]
    pub user_repo_refresh_interval: Option<u64>,
}
