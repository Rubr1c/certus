use std::time::Duration;

use axum::body::Body;
use hyper::client::conn;
use hyper_util::rt::TokioExecutor;
use hyper_util::rt::TokioIo;
use tokio::net::TcpStream;
use tracing::instrument;

use crate::upstream::{
    protocol::{HttpVersion, PooledConnection, Protocol},
    server::UpstreamServer,
};

use super::tls::tls_connector;

/// Opens a connection to a tcp stream
///
/// # Arguments
///
/// * `upstream` - target server to conenct to
/// * `timeout` - when to timeout trying to connect
///
/// # Errors
///
/// Returns an error if:
/// * Failed to connect to server
/// * Timed out while trying to connect
/// * Failed to perform handshake with server
#[instrument(skip_all, fields(http_version = ?upstream.pool.http_version))]
pub async fn open_connection(
    upstream: &UpstreamServer,
    timeout: u64,
) -> Result<PooledConnection, Box<dyn std::error::Error + Send + Sync>> {
    tracing::info!("Connecting to new upstream");
    let connect_future = TcpStream::connect(upstream.pool.server_addr.as_ref());
    let stream =
        tokio::time::timeout(Duration::from_secs(timeout), connect_future)
            .await??;

    let sender = match upstream.pool.http_version {
        HttpVersion::HTTP1 => match upstream.pool.protocol {
            Protocol::HTTPS => {
                let connector = tls_connector(HttpVersion::HTTP1);
                let domain = rustls::pki_types::ServerName::try_from(
                    upstream.pool.hostname.as_str(),
                )?
                .to_owned();
                let tls_stream = connector.connect(domain, stream).await?;
                let io = TokioIo::new(tls_stream);
                let (sender, conn) =
                    conn::http1::handshake::<_, Body>(io).await?;
                tokio::task::spawn(async move {
                    if let Err(err) = conn.await {
                        tracing::error!(?err, "Connection failed");
                    }
                });
                PooledConnection::Http1(sender)
            }
            Protocol::HTTP => {
                let io = TokioIo::new(stream);
                let (sender, conn) =
                    conn::http1::handshake::<_, Body>(io).await?;
                tokio::task::spawn(async move {
                    if let Err(err) = conn.await {
                        tracing::error!(?err, "Connection failed");
                    }
                });
                PooledConnection::Http1(sender)
            }
        },
        HttpVersion::HTTP2 => {
            let exec = TokioExecutor::new();
            match upstream.pool.protocol {
                Protocol::HTTPS => {
                    let connector = tls_connector(HttpVersion::HTTP2);
                    let domain = rustls::pki_types::ServerName::try_from(
                        upstream.pool.hostname.as_str(),
                    )?
                    .to_owned();
                    let tls_stream = connector.connect(domain, stream).await?;
                    let io = TokioIo::new(tls_stream);
                    let (sender, conn) =
                        conn::http2::handshake(exec, io).await?;
                    tokio::task::spawn(async move {
                        if let Err(err) = conn.await {
                            tracing::error!(?err, "Connection failed");
                        }
                    });
                    PooledConnection::Http2(sender)
                }
                Protocol::HTTP => {
                    let io = TokioIo::new(stream);
                    let (sender, conn) =
                        conn::http2::handshake(exec, io).await?;
                    tokio::task::spawn(async move {
                        if let Err(err) = conn.await {
                            tracing::error!(?err, "Connection failed");
                        }
                    });
                    PooledConnection::Http2(sender)
                }
            }
        }
    };

    tracing::info!("Connetion made");

    Ok(sender)
}
