use std::sync::atomic::Ordering;

use axum::{body::Body, extract::Request, response::Response};
use hyper::body::Incoming;

use crate::server::{
    connection,
    models::{PooledConnection, UpstreamServer},
};

async fn forward_request(
    conn: PooledConnection,
    req: Request<Body>,
) -> Result<(Response<Incoming>, PooledConnection), &'static str> {
    let (res, sender) = match conn {
        PooledConnection::Http1(mut sender) => {
            let res = sender
                .send_request(req)
                .await
                .map_err(|_| "Failed to Send Request")?;
            (res, PooledConnection::Http1(sender))
        }
        PooledConnection::Http2(mut sender) => {
            let res = sender
                .send_request(req)
                .await
                .map_err(|_| "Failed to Send Request")?;
            (res, PooledConnection::Http2(sender))
        }
    };

    Ok((res, sender))
}

pub async fn handle_request(
    upstream: &UpstreamServer,
    req: Request<Body>,
) -> Result<Response<Incoming>, &'static str> {
    let sender = connection::borrow_connection(&upstream).await?;

    let (res, sender) = match forward_request(sender, req).await {
        Ok((res, sender)) => (res, sender),
        Err(e) => {
            upstream.pool.total_connections.fetch_sub(1, Ordering::Relaxed);
            return Err(e);
        }
    };

    let reusable =
        !res.headers().get("connection").is_some_and(|v| v == "close");

    connection::release_connection(&upstream, sender, reusable).await;

    Ok(res)
}
