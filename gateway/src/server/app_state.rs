use std::{
    collections::HashMap,
    net::{IpAddr, SocketAddr},
    sync::Arc,
};

use arc_swap::ArcSwap;
use dashmap::DashMap;
use matchit::Router;
use moka::sync::Cache;

use crate::{
    config::models::Config,
    server::{
        middleware::{
            cache::{
                models::{CacheKey, CachedResponse},
                static_cache,
            },
            rate_limit::TokenBucket,
            router,
        },
        upstream::models::UpstreamServer,
    },
};

pub struct RoutingTable {
    pub router: Router<String>,
    pub routes: HashMap<SocketAddr, Arc<UpstreamServer>>,
}

//TODO: Add db connection in efficent way to use in metrics endpoints
pub struct AppState {
    pub routing_table: ArcSwap<RoutingTable>,
    pub config: ArcSwap<Config>,
    pub cache: Cache<CacheKey, CachedResponse>,
    pub static_cache: DashMap<String, CachedResponse>,
    pub user_tokens: DashMap<IpAddr, TokenBucket>,
}

impl AppState {
    pub fn new(config: Config) -> Self {
        Self {
            routing_table: ArcSwap::from_pointee(RoutingTable {
                router: Router::new(),
                routes: HashMap::new(),
            }),
            config: ArcSwap::from_pointee(config),
            cache: Cache::new(1000),
            static_cache: DashMap::new(),
            user_tokens: DashMap::new(),
        }
    }
}

pub async fn init_server_state(state: Arc<AppState>) {
    let config = state.config.load();

    let mut new_routes_map = HashMap::new();

    for (route, route_config) in config.routes.iter() {
        let mut is_static_and_not_fetched =
            route_config.is_static.is_some_and(|c| c);
        for server in &route_config.endpoints {
            let upstream = Arc::new(UpstreamServer::new(
                *server,
                route_config.max_connections,
                route_config.protocol,
                route_config.needs_auth.is_some_and(|c| c),
            ));
            if is_static_and_not_fetched {
                static_cache::send_and_save(
                    &state.static_cache,
                    &upstream,
                    route,
                    config.connection.connect_timeout,
                )
                .await;
                is_static_and_not_fetched = false;
            }
            new_routes_map.insert(*server, upstream);
        }
    }

    let new_router = router::build_tree(state.clone());

    let new_table = RoutingTable { router: new_router, routes: new_routes_map };

    state.routing_table.store(Arc::new(new_table));
}
