use std::sync::atomic::Ordering;

use axum::body::Body;
use hyper::client::conn;
use hyper_util::rt::TokioIo;
use tokio::net::TcpStream;

use crate::server::models::UpstreamServer;

async fn open_http1_connection(
    upstream: &UpstreamServer,
) -> Result<
    conn::http1::SendRequest<Body>,
    Box<dyn std::error::Error + Send + Sync>,
> {
    let stream = TcpStream::connect(upstream.address).await?;
    let io = TokioIo::new(stream);

    let (mut sender, conn) = conn::http1::handshake::<_, Body>(io).await?;
    tokio::task::spawn(async move {
        if let Err(err) = conn.await {
            println!("Connection Failed: {:?}", err);
        }
    });

    upstream.pool.total_connections.fetch_add(1, Ordering::Relaxed);

    Ok(sender)
}

async fn open_http2_connection(
    upstream: &UpstreamServer,
) -> Result<
    conn::http2::SendRequest<Body>,
    Box<dyn std::error::Error + Send + Sync>,
> {
    todo!()
}
