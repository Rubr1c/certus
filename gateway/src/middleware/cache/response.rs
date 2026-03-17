use axum::response::IntoResponse;

#[derive(Debug, Clone)]
pub struct CachedResponse {
    pub status: hyper::StatusCode,
    pub headers: hyper::HeaderMap,
    pub body: axum::body::Bytes,
}

impl IntoResponse for CachedResponse {
    fn into_response(self) -> hyper::Response<axum::body::Body> {
        let mut response =
            hyper::Response::new(axum::body::Body::from(self.body));

        *response.status_mut() = self.status;
        *response.headers_mut() = self.headers;

        response
    }
}
