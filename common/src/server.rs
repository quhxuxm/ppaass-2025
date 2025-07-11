use crate::config::WithServerConfig;
use crate::error::Error;
use std::error::Error as StdError;
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info};
/// The state of the server
#[derive(Debug)]
pub struct ServerState {
    /// The incoming stream
    pub incoming_stream: TcpStream,
    /// The incoming connection's address
    pub incoming_connection_addr: SocketAddr,
}

/// The guard of the server
pub struct ServerGuard {
    /// The signal to stop the server
    pub stop_signal: CancellationToken,
}

pub fn start_server<C, F, Fut, ImplErr>(config: &C, connection_handler: F) -> ServerGuard
where
    C: WithServerConfig,
    F: Fn(ServerState) -> Fut + Send + Sync + Copy + 'static,
    Fut: Future<Output = Result<(), ImplErr>> + Send + 'static,
    ImplErr: StdError + From<Error>,
{
    let stop_single = CancellationToken::new();
    let server_guard = ServerGuard {
        stop_signal: stop_single.clone(),
    };
    let listening_address = config.listening_address();
    tokio::spawn(async move {
        let tcp_listener = match TcpListener::bind(listening_address).await {
            Ok(tcp_listener) => tcp_listener,
            Err(e) => {
                error!("Fail to bind server [{listening_address}] because of error: {e:?}");
                return;
            }
        };
        loop {
            tokio::select! {
                _ = stop_single.cancelled() => {
                    info!("Receive stop signal, stop server success.");
                    return;
                }
                client_connection = tcp_listener.accept() => {
                    let (incoming_stream, incoming_connection_addr) = match client_connection {
                        Ok((incoming_stream, incoming_connection_addr)) => (incoming_stream, incoming_connection_addr),
                        Err(e) => {
                            error!("Failed to accept incoming connection: {}", e);
                            continue;
                        }
                    };
                    debug!("Accept incoming connection from {}", incoming_connection_addr);
                    tokio::spawn(async move {
                        let server_state = ServerState {
                            incoming_stream,
                            incoming_connection_addr,
                        };
                        if let Err(e) = connection_handler(server_state).await {
                            error!("Failed to handle incoming connection: {:?}", e);
                        }
                    });
                }
            }
        }
    });
    server_guard
}
