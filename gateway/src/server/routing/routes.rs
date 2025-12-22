use std::sync::{Arc, LazyLock};

use axum::{
    extract::{Path, State},
    response::IntoResponse,
};
use hyper::StatusCode;
use parking_lot::RwLock;
use trie_rs::{Trie, TrieBuilder};

use crate::{
    config::cfg_utils::CONFIG,
    server::{app_state::AppState, load_balancing::balancing},
};

pub static ROUTE_TRIE: LazyLock<RwLock<Trie<String>>> =
    LazyLock::new(|| RwLock::new(TrieBuilder::new().build()));

pub fn build_tree() {
    let route_conf = &CONFIG.read().routes;
    let mut builder = TrieBuilder::new();

    for route in route_conf {
        let segments =
            route.0.split('/').map(|s| s.to_string()).collect::<Vec<_>>();

        builder.push(segments);
    }

    *ROUTE_TRIE.write() = builder.build();
}

pub fn get_longest_macthing_route(route: &str) -> String {
    let split_route =
        route.split('/').map(|s| s.to_string()).collect::<Vec<_>>();

    let guard = ROUTE_TRIE.read();

    guard
        .common_prefix_search(split_route)
        .max_by_key(|v: &Vec<String>| v.len())
        .map(|v| v.join("/"))
        .unwrap_or_else(|| "".to_string())
}

pub async fn reroute(
    Path(path): Path<String>,
    State(state): State<Arc<RwLock<AppState>>>,
) -> impl IntoResponse {
    let route = get_longest_macthing_route(path.as_str());
    if route.is_empty() {
        (StatusCode::OK, "TODO: None Found").into_response()
    } else {
        let server = balancing::p2c_pick(route, state);
        (
            StatusCode::OK,
            //etrun selected server for testing
            server.to_string(),
        )
            .into_response()
    }
}
