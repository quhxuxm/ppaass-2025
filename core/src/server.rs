use crate::config::CoreServerConfig;
use crate::error::CoreError;
use crate::runtime::ServerRuntime;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info};
pub struct CoreServerState<C>
where
    C: CoreServerConfig + Send + Sync + 'static,
{
    pub client_stream: TcpStream,
    pub client_addr: SocketAddr,
    pub config: Arc<C>,
}
pub struct CoreServerGuard {
    pub stop_single: CancellationToken,
}
pub struct CoreServer<C, R>
where
    C: CoreServerConfig + Send + Sync + 'static,
    R: ServerRuntime + Send + Sync + 'static,
{
    config: Arc<C>,
    runtime: R,
}
impl<C, R> CoreServer<C, R>
where
    C: CoreServerConfig + Send + Sync + 'static,
    R: ServerRuntime + Send + Sync + 'static,
{
    pub fn new(config: C, runtime: R) -> Self {
        Self { config: Arc::new(config), runtime }
    }
    pub fn start<F, Fut>(self, connection_handler: F) -> CoreServerGuard
    where
        F: Fn(CoreServerState<C>) -> Fut + Send + Sync + Clone + 'static,
        Fut: Future<Output = Result<(), CoreError>> + Send + 'static,
    {
        let stop_single = CancellationToken::new();
        let server_guard = CoreServerGuard {
            stop_single: stop_single.clone(),
        };
        let config = self.config;
        self.runtime.block_on(async move {
            if let Err(e) = Self::process(config, connection_handler, stop_single).await {
                error!("Failed to start server: {}", e);
            }
        });
        server_guard
    }
    async fn process<F, Fut>(config: Arc<C>, connection_handler: F, stop_single: CancellationToken) -> Result<(), CoreError>
    where
        F: Fn(CoreServerState<C>) -> Fut + Send + Sync + Clone + 'static,
        Fut: Future<Output = Result<(), CoreError>> + Send + 'static,
    {
        let tcp_listener = TcpListener::bind(config.listening_address()).await?;
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
                    let config = config.clone();
                    R::spawn(async move {
                        let server_state = CoreServerState {
                            client_stream,
                            client_addr,
                            config,
                        };
                        if let Err(e) = connection_handler(server_state).await {
                            error!("Failed to handle client connection: {}", e);
                        }
                    });
                }
            }
        }
    }
}