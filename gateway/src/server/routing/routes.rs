use std::sync::{Arc, LazyLock};

use axum::{
    body::Body,
    extract::{Path, Request, State},
    response::IntoResponse,
};
use hyper::StatusCode;
use parking_lot::RwLock;
use trie_rs::{Trie, TrieBuilder};

use crate::{
    server::{
        app_state::AppState, load_balancing::balancing, request::requests,
    },
};


pub fn build_tree(state: Arc<AppState>) {
    let config = state.config.load();
    let route_conf = &config.routes;
    let mut builder = TrieBuilder::new();

    for route in route_conf {
        let segments =
            route.0.split('/').map(|e| e.to_string()).collect::<Vec<_>>();

        builder.push(segments);
    }

    state.route_trie.store(Arc::new(builder.build()));
}

pub fn get_longest_macthing_route(trie: &Trie<String>, route: &str) -> String {

    let split_route =
        route.split('/').map(|s| s.to_string()).collect::<Vec<_>>();


    trie 
        .common_prefix_search(split_route)
        .max_by_key(|v: &Vec<String>| v.len())
        .map(|v| v.join("/"))
        .unwrap_or_else(|| "".to_string())
}

pub async fn reroute(
    Path(path): Path<String>,
    State(state): State<Arc<AppState>>,
    req: Request<Body>,
) -> impl IntoResponse {
    let trie = state.route_trie.load();
    let routes = state.routes.load();
    let route = get_longest_macthing_route(&trie, &path);
    let server = balancing::p2c_pick(route, state);
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
