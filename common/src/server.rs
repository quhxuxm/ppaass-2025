use crate::config::ServerConfig;
use crate::error::BaseError;
use crate::user::UserRepository;
use std::error::Error;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info};
pub struct BaseServerState<C, UR>
where
    C: ServerConfig + Send + Sync + 'static,
    UR: UserRepository + Send + Sync + 'static,
{
    pub client_stream: TcpStream,
    pub client_addr: SocketAddr,
    pub config: Arc<C>,
    pub user_repository: Arc<UR>,
}
pub struct BaseServerGuard {
    pub stop_single: CancellationToken,
}
pub struct BaseServer<C, UR>
where
    C: ServerConfig + Send + Sync + 'static,
    UR: UserRepository + Send + Sync + 'static,
{
    config: Arc<C>,
    user_repository: Arc<UR>,
}
impl<C, UR> BaseServer<C, UR>
where
    C: ServerConfig + Send + Sync + 'static,
    UR: UserRepository + Send + Sync + 'static,
{
    pub fn new(config: Arc<C>, user_repository: Arc<UR>) -> Self {
        Self {
            config,
            user_repository,
        }
    }
    pub fn start<F, Fut, ImplErr>(self, connection_handler: F) -> BaseServerGuard
    where
        F: Fn(BaseServerState<C, UR>) -> Fut + Send + Sync + Copy + 'static,
        Fut: Future<Output = Result<(), ImplErr>> + Send + 'static,
        ImplErr: Error + From<BaseError>,
    {
        let stop_single = CancellationToken::new();
        let server_guard = BaseServerGuard {
            stop_single: stop_single.clone(),
        };
        let config = self.config;
        let user_repository = self.user_repository.clone();
        tokio::spawn(async move {
            if let Err(e) =
                Self::process(config, user_repository, connection_handler, stop_single).await
            {
                error!("Failed to start server: {}", e);
            }
        });
        server_guard
    }
    async fn process<F, Fut, ImplErr>(
        config: Arc<C>,
        user_repository: Arc<UR>,
        connection_handler: F,
        stop_single: CancellationToken,
    ) -> Result<(), BaseError>
    where
        F: Fn(BaseServerState<C, UR>) -> Fut + Send + Sync + Clone + 'static,
        Fut: Future<Output = Result<(), ImplErr>> + Send + 'static,
        ImplErr: Error + From<BaseError>,
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
                    let user_repository=user_repository.clone();
                    let config=config.clone();
                    tokio::spawn(async move {
                        let server_state = BaseServerState {
                            client_stream,
                            client_addr,
                            config,
                            user_repository
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
