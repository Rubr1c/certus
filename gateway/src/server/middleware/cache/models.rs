use axum::{
    body::{Body, Bytes},
    response::IntoResponse,
};
use hyper::{HeaderMap, Response, StatusCode};

#[derive(Hash, Eq, PartialEq)]
pub struct CacheKey {
    pub user_id: Option<u64>,
    //TODO: get roles from config
    pub user_role: Option<String>,
    pub path: String,
}

#[derive(Clone)]
pub struct CachedResponse {
    pub status: StatusCode,
    pub headers: HeaderMap,
    pub body: Bytes,
}

impl IntoResponse for CachedResponse {
    fn into_response(self) -> Response<Body> {
        let mut response = Response::new(Body::from(self.body));

        *response.status_mut() = self.status;
        *response.headers_mut() = self.headers;

        response
    }
}
