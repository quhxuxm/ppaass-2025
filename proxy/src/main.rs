use crate::config::ProxyConfig;
use crate::error::ProxyError;
use ppaass_2025_core::{generate_core_runtime, CoreError, CoreServer, CoreServerState};
use std::fs::read_to_string;
use std::str::FromStr;
use std::sync::Arc;
use tokio::signal;
use tracing::{debug, error, info, Level};
use tracing_subscriber::fmt::time::ChronoUtc;
mod config;
mod error;
mod user;
const PROXY_CONFIG_FILE: &str = "resource/proxy.toml";
async fn handle_connection(core_server_state: CoreServerState<ProxyConfig>) -> Result<(), CoreError> {
    debug!("Handle connection: {:?}", core_server_state.client_addr);
    Ok(())
}
fn main() -> Result<(), ProxyError> {
    let proxy_config_content = read_to_string(PROXY_CONFIG_FILE)?;
    let proxy_config = Arc::new(toml::from_str::<ProxyConfig>(&proxy_config_content)?);
    let (trace_file_appender, _trace_appender_guard) = tracing_appender::non_blocking(
        tracing_appender::rolling::daily(proxy_config.log_directory(), proxy_config.log_name_prefix()),
    );
    tracing_subscriber::fmt()
        .with_max_level(Level::from_str(proxy_config.max_log_level())?)
        .with_writer(trace_file_appender)
        .with_line_number(true)
        .with_level(true)
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_timer(ChronoUtc::rfc_3339())
        .with_ansi(false)
        .init();
    let proxy_runtime = generate_core_runtime(proxy_config.as_ref())?;
    let _runtime_guard = proxy_runtime.enter();
    let core_server = CoreServer::new(proxy_config);
    let server_guard = core_server.start(handle_connection);
    proxy_runtime.block_on(async {
        if let Err(e) = signal::ctrl_c().await {
            error!("Failed to listen to shutdown signal: {}", e);
        }
        info!("Begin to stop Ppaass proxy.");
        server_guard.stop_single.cancel();
    });
    Ok(())
}
