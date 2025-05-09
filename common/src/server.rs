use crate::config::ServerConfig;
use crate::error::Error;
use std::error::Error as StdError;
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info};
#[derive(Debug)]
pub struct ServerState {
    pub client_stream: TcpStream,
    pub client_addr: SocketAddr,
}
pub struct ServerGuard {
    pub stop_single: CancellationToken,
}
pub struct Server {
    listening_address: SocketAddr,
}
impl Server {
    pub fn new<C>(config: &C) -> Self
    where
        C: ServerConfig,
    {
        Self {
            listening_address: config.listening_address(),
        }
    }
    pub fn start<F, Fut, ImplErr>(self, connection_handler: F) -> ServerGuard
    where
        F: Fn(ServerState) -> Fut + Send + Sync + Copy + 'static,
        Fut: Future<Output = Result<(), ImplErr>> + Send + 'static,
        ImplErr: StdError + From<Error>,
    {
        let stop_single = CancellationToken::new();
        let server_guard = ServerGuard {
            stop_single: stop_single.clone(),
        };
        let listening_address = self.listening_address;
        tokio::spawn(async move {
            if let Err(e) = Self::process(listening_address, connection_handler, stop_single).await
            {
                error!("Failed to start server: {}", e);
            }
        });
        server_guard
    }
    async fn process<F, Fut, ImplErr>(
        listening_address: SocketAddr,
        connection_handler: F,
        stop_single: CancellationToken,
    ) -> Result<(), Error>
    where
        F: Fn(ServerState) -> Fut + Send + Sync + Clone + 'static,
        Fut: Future<Output = Result<(), ImplErr>> + Send + 'static,
        ImplErr: StdError + From<Error>,
    {
        let tcp_listener = TcpListener::bind(listening_address).await?;
        loop {
            tokio::select! {
                _ = stop_single.cancelled() => {
                    info!("Server stopped success.");
                    return Ok(());
                }
                client_connection = tcp_listener.accept() => {
                    let (client_stream, client_addr) = match client_connection {
                        Ok((client_stream, client_addr)) => (client_stream, client_addr),
                        Err(e) => {
                            error!("Failed to accept client connection: {}", e);
                            continue;
                        }
                    };
                    debug!("Accept client connection from {}", client_addr);
                    let connection_handler = connection_handler.clone();
                    tokio::spawn(async move {
                        let server_state = ServerState {
                            client_stream,
                            client_addr,
                        };
                        if let Err(e) = connection_handler(server_state).await {
                            error!("Failed to handle client connection: {:?}", e);
                        }
                    });
                }
            }
        }
    }
}
