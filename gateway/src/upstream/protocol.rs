use hyper::client::conn;
use serde::{Deserialize, Serialize};

/// Enum for all available protocols
#[derive(Clone, Debug, Deserialize, Serialize, Default, Copy)]
pub enum HttpVersion {
    #[default]
    HTTP1,
    HTTP2,
}

#[derive(Clone, Debug, Deserialize, Serialize, Default, PartialEq)]
pub enum Protocol {
    #[default]
    HTTP,
    HTTPS,
}

/// Enum holding send request of the protocols
pub enum PooledConnection {
    Http1(conn::http1::SendRequest<axum::body::Body>),
    Http2(conn::http2::SendRequest<axum::body::Body>),
}
