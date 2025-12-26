use std::sync::atomic::Ordering;

use axum::body::Body;
use hyper::client::conn;
use hyper_util::rt::{TokioExecutor, TokioIo};
use tokio::net::TcpStream;

use crate::server::models::{PooledConnection, Protocol, UpstreamServer};

pub async fn open_connection(
    upstream: &UpstreamServer,
) -> Result<PooledConnection, Box<dyn std::error::Error + Send + Sync>> {
    //TODO: timeout connection
    let stream = TcpStream::connect(upstream.pool.server_addr).await?;
    let io = TokioIo::new(stream);

    let sender = match upstream.pool.protocol {
        Protocol::HTTP1 => {
            let (sender, conn) = conn::http1::handshake::<_, Body>(io).await?;
            tokio::task::spawn(async move {
                if let Err(err) = conn.await {
                    eprintln!("Connection Failed: {:?}", err);
                }
            });
            PooledConnection::Http1(sender)
        }
        Protocol::HTTP2 => {
            let exec = TokioExecutor::new();
            let (sender, conn) = conn::http2::handshake(exec, io).await?;
            tokio::task::spawn(async move {
                if let Err(err) = conn.await {
                    eprintln!("Connection Failed: {:?}", err);
                }
            });
            PooledConnection::Http2(sender)
        }
    };

    upstream.pool.total_connections.fetch_add(1, Ordering::Release);

    Ok(sender)
}

pub async fn borrow_connection(
    upstream: &UpstreamServer,
) -> Result<PooledConnection, &'static str> {
    if let Some(sender) = upstream.pool.idle_connections.pop() {
        upstream.pool.total_connections.fetch_add(1, Ordering::Release);
        return Ok(sender);
    }

    let total = upstream.pool.total_connections.load(Ordering::Acquire);
    if total >= upstream.pool.max_connections {
        return Err("upstream overloaded");
    }

    let sender =
        open_connection(upstream).await.map_err(|_| "Failed To Connect")?;

    upstream.pool.total_connections.fetch_add(1, Ordering::Release);
    upstream.active_connctions.fetch_add(1, Ordering::Release);

    Ok(sender)
}

pub async fn release_connection(
    upstream: &UpstreamServer,
    sender: PooledConnection,
    reusable: bool,
) {
    upstream.pool.total_connections.fetch_sub(1, Ordering::Release);

    if reusable {
        upstream.pool.idle_connections.push(sender);
    } else {
        upstream.pool.total_connections.fetch_sub(1, Ordering::Release);
    }
}
