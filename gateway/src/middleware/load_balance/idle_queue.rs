use std::sync::Arc;

use axum::{body::to_bytes, response::IntoResponse};
use crossbeam::queue::SegQueue;

use crate::{
    server::state::app_state::AppState, upstream::server::UpstreamServer,
};

//TODO: make faster
pub async fn set_idle(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    req: axum::extract::Request,
) -> impl IntoResponse {
    let bytes = match to_bytes(req.into_body(), 1024 * 1024).await {
        Ok(b) => b,
        Err(_) => return axum::http::StatusCode::BAD_REQUEST.into_response(),
    };

    let body_string = String::from_utf8_lossy(&bytes).into_owned();

    let config = state.config.load();
    let routing_table = state.routing_table.load();
    let mut upstream: Option<Arc<UpstreamServer>> = None;

    for (addr, u_server) in &routing_table.routes {
        if *addr == body_string {
            upstream = Some(u_server.clone());
        }
    }

    for (route, route_config) in &config.routes {
        if route_config.endpoints.contains(&body_string) {
            if let Some(server) = &upstream {
                let queue = state.idle_queue.get_mut(route);
                match queue {
                    Some(q) => {
                        tracing::info!("Server pushed to idle");
                        q.push(server.clone());
                    }
                    _ => {
                        let q = SegQueue::<Arc<UpstreamServer>>::new();
                        q.push(server.clone());
                        tracing::info!("Server pushed to idle");
                        state.idle_queue.insert(route.clone(), q);
                    }
                }
            }
        }
    }

    axum::http::StatusCode::OK.into_response()
}
