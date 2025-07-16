use crate::error::Error;
use crate::tunnel::fetch_proxy_connection;
use common::ServerState;
use common::proxy::DestinationType;
use http_body_util::combinators::BoxBody;
use http_body_util::{BodyExt, Empty};
use hyper::body::Incoming;
use hyper::client::conn::http1::Builder;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Method, Request, Response};
use hyper_util::rt::TokioIo;
use protocol::UnifiedAddress;
use std::net::SocketAddr;
use tokio_util::bytes::Bytes;
use tower::ServiceBuilder;
use tracing::{debug, error, info};
pub async fn process_http_tunnel(server_state: ServerState) -> Result<(), Error> {
    let client_tcp_io = TokioIo::new(server_state.incoming_stream);
    let service_fn = ServiceBuilder::new().service(service_fn(|request| async {
        client_http_request_handler(server_state.incoming_connection_addr, request)
            .await
            .map_err(|e| format!("{e:?}"))
    }));
    http1::Builder::new()
        .preserve_header_case(true)
        .title_case_headers(true)
        .serve_connection(client_tcp_io, service_fn)
        .with_upgrades()
        .await?;
    Ok(())
}

fn success_empty_body() -> BoxBody<Bytes, hyper::Error> {
    Empty::<Bytes>::new()
        .map_err(|never| match never {})
        .boxed()
}

async fn client_http_request_handler(
    client_addr: SocketAddr,
    client_http_request: Request<Incoming>,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, Error> {
    let destination_uri = client_http_request.uri();
    let destination_host = destination_uri
        .host()
        .ok_or(Error::NoDestinationHost(destination_uri.clone()))?;
    let destination_port = destination_uri.port().map(|port| port.as_u16());
    let destination_address = if client_http_request.method() == Method::CONNECT {
        UnifiedAddress::Domain {
            host: destination_host.to_string(),
            port: destination_port.unwrap_or(443),
        }
    } else {
        UnifiedAddress::Domain {
            host: destination_host.to_string(),
            port: destination_port.unwrap_or(80),
        }
    };
    debug!(
        "Receive client http request to destination: {destination_address:?}, client socket address: {client_addr}"
    );

    let proxy_connection = fetch_proxy_connection().await?;
    let mut proxy_connection = proxy_connection
        .setup_destination(destination_address, DestinationType::Tcp)
        .await?;

    if Method::CONNECT == client_http_request.method() {
        // Received an HTTP request like:
        // ```
        // CONNECT www.domain.com:443 HTTP/1.1
        // Host: www.domain.com:443
        // Proxy-Connection: Keep-Alive
        // ```
        //
        // When HTTP method is CONNECT we should return an empty body
        // then we can eventually upgrade the connection and talk a new protocol.
        //
        // Note: only after client received an empty body with STATUS_OK can the
        // connection be upgraded, so we can't return a response inside
        // `on_upgrade` future.
        tokio::task::spawn(async move {
            match hyper::upgrade::on(client_http_request).await {
                Err(e) => {
                    error!("Failed to upgrade client http request: {e}");
                }
                Ok(upgraded_client_io) => {
                    // Connect to remote server
                    let mut upgraded_client_io = TokioIo::new(upgraded_client_io);
                    // Proxying data
                    let (from_client, from_proxy) = match tokio::io::copy_bidirectional(
                        &mut upgraded_client_io,
                        &mut proxy_connection,
                    )
                    .await
                    {
                        Err(e) => {
                            error!("Fail to proxy data between agent and proxy: {e:?}");
                            return;
                        }
                        Ok((from_client, from_proxy)) => (from_client, from_proxy),
                    };

                    // Print message when done
                    info!(
                        "Agent wrote {} bytes to proxy, received {} bytes from proxy",
                        from_client, from_proxy
                    );
                }
            }
        });
        Ok(Response::new(success_empty_body()))
    } else {
        let proxy_connection = TokioIo::new(proxy_connection);
        let (mut proxy_connection_sender, proxy_connection_obj) = Builder::new()
            .preserve_header_case(true)
            .title_case_headers(true)
            .handshake(proxy_connection)
            .await?;
        tokio::spawn(async move {
            if let Err(err) = proxy_connection_obj.await {
                error!("Proxy tcp connection failed: {:?}", err);
            }
        });

        let proxy_response = proxy_connection_sender
            .send_request(client_http_request)
            .await?;
        Ok(proxy_response.map(|b| b.boxed()))
    }
}
