use std::sync::{Arc, LazyLock};

use axum::{
    body::Body,
    extract::{Path, Request, State},
    response::IntoResponse,
};
use hyper::StatusCode;
use parking_lot::RwLock;
use trie_rs::TrieBuilder;

use crate::{
    server::{
        app_state::AppState, load_balancing::balancing, request::requests,
    },
};


pub fn build_tree(state: Arc<RwLock<AppState>>) {
    let mut state_guard = state.write();
    let config_arc = state_guard.config.clone();
    let config_gaurd = config_arc.read();
    let route_conf = &config_gaurd.routes;
    let mut builder = TrieBuilder::new();

    for route in route_conf {
        let segments =
            route.0.split('/').map(|e| e.to_string()).collect::<Vec<_>>();

        builder.push(segments);
    }

    state_guard.route_trie = Arc::new(RwLock::new(builder.build()));
}

pub fn get_longest_macthing_route(state: Arc<RwLock<AppState>>,route: &str) -> String {
    let split_route =
        route.split('/').map(|s| s.to_string()).collect::<Vec<_>>();

    let state_guard = state.read();
    let trie = state_guard.route_trie.read();

    trie 
        .common_prefix_search(split_route)
        .max_by_key(|v: &Vec<String>| v.len())
        .map(|v| v.join("/"))
        .unwrap_or_else(|| "".to_string())
}

pub async fn reroute(
    Path(path): Path<String>,
    State(state): State<Arc<RwLock<AppState>>>,
    req: Request<Body>,
) -> impl IntoResponse {
    let route = get_longest_macthing_route(state.clone(), &path);
    if route.is_empty() {
        (StatusCode::OK, "TODO: None Found").into_response()
    } else {
        let server = balancing::p2c_pick(route, state.clone());
        let upstream = {
            let state_guard = state.read();
            state_guard
                .routes
                .get(&server)
                .expect("Upstream Should Exist")
                .clone()
        };

        let res = requests::handle_request(&upstream, req).await;
        match res {
            Ok(response) => response.into_response(),
            Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
        }
    }
}
