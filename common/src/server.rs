use crate::config::WithServerConfig;
use crate::error::Error;
use std::error::Error as StdError;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Semaphore;
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

/// Start a server with the given configuration and connection handler.
/// The connection handler is a function that takes a `ServerState` and returns a future.
/// The server will run until the stop signal is received.
/// The server will listen on the address specified in the configuration.
/// The connection handler will be called for each incoming connection.
/// The server will handle the connections concurrently.
/// The server will stop when the stop signal is received or when the server is dropped.
/// The server will return a `ServerGuard` that can be used to stop the server.
pub fn start_server<C, F, Fut, Err>(config: &C, connection_handler: F) -> ServerGuard
where
    C: WithServerConfig,
    F: Fn(ServerState) -> Fut + Send + Sync + Copy + 'static,
    Fut: Future<Output = Result<(), Err>> + Send + 'static,
    Err: StdError + From<Error>,
{
    let stop_single = CancellationToken::new();
    let server_guard = ServerGuard {
        stop_signal: stop_single.clone(),
    };
    let listening_address = config.listening_address();
    // The max connections from client, if the number of connections exceeds this limit, the server will waiting for the next available connection permit.
    let client_max_connections = Arc::new(Semaphore::new(config.client_max_connections()));
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
                    let client_connection_permit=match client_max_connections.clone().acquire_owned().await{
                        Ok(client_connection_permit) => client_connection_permit,
                        Err(e) => {
                            error!("Fail to acquire client connection permit because of error: {e:?}");
                            continue;
                        }
                    };
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
                        drop(client_connection_permit);
                    });
                }
            }
        }
    });
    server_guard
}
