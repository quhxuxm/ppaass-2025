use clap::Parser;
use std::net::SocketAddr;
use std::path::PathBuf;
#[derive(Parser)]
#[command(version, about, long_about = None)]
pub(crate) struct AgentCommandArgs {
    #[arg(short = 'p', long)]
    pub listening_address: Option<SocketAddr>,
    #[arg(short = 't', long)]
    pub worker_threads: Option<usize>,
    #[arg(short = 'l', long)]
    pub log_directory: Option<PathBuf>,
    #[arg(short = 'm', long)]
    pub max_log_level: Option<String>,
    #[arg(short = 'r', long)]
    pub user_repo_directory: Option<PathBuf>,
    #[arg(short = 'i', long)]
    pub user_repo_refresh_interval: Option<u64>,
    #[arg(short = 'u', long)]
    pub username: String,
}
