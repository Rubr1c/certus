use std::sync::{Arc, LazyLock};

use axum::{
    body::Body,
    extract::{Path, Request, State},
    response::IntoResponse,
};
use hyper::StatusCode;
use matchit::Router;
use parking_lot::RwLock;

use crate::{
    server::{
        app_state::AppState, load_balancing::balancing, request::requests,
    },
};


pub fn build_tree(state: Arc<AppState>) {
    let config = state.config.load();
    let route_conf = &config.routes;

    let mut router = Router::new();

    for route in route_conf {
       if let Err(e) = router.insert(route.0, route.0.clone()) {
            eprintln!("Failed to insert route '{}': {}", route.0, e);
        } 
    }

    state.router.store(Arc::new(router));
}

pub async fn reroute(
    Path(path): Path<String>,
    State(state): State<Arc<AppState>>,
    req: Request<Body>,
) -> impl IntoResponse {
    let router = state.router.load();
    let routes = state.routes.load();
    let matched_route_key = match router.at(&path) {
        Ok(match_result) => match_result.value,
        Err(_) => return (StatusCode::NOT_FOUND, "No Route Found").into_response(),
    };

    let server = balancing::p2c_pick(matched_route_key, state);
        let upstream = routes
                .get(&server)
                .expect("Upstream Should Exist")
                .clone();
        

        let res = requests::handle_request(&upstream, req).await;
        match res {
            Ok(response) => response.into_response(),
            Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
        }
}
