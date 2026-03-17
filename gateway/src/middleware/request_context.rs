use std::{
    net::{IpAddr, SocketAddr},
    sync::Arc,
};

use hyper::header;

use crate::config::Config;

use super::{auth, ip};

pub struct RequestContext {
    pub path: Arc<str>,
    pub query: Option<Arc<str>>,
    pub ip: IpAddr,
    pub token: Option<String>,
    pub bytes_in: u64,
    pub method: hyper::Method,
}

impl RequestContext {
    #[inline]
    pub fn extract(
        req: &hyper::Request<axum::body::Body>,
        addr: SocketAddr,
        config: &Config,
    ) -> Self {
        let headers = req.headers();

        Self {
            path: req.uri().path().into(),
            query: req.uri().query().map(Arc::from),
            ip: ip::extract(headers, addr.ip()),
            token: auth::gate::extract(config, headers),
            bytes_in: headers
                .get(header::CONTENT_LENGTH)
                .and_then(|h| h.to_str().ok())
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(0),
            method: req.method().clone(),
        }
    }
}
