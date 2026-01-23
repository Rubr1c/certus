use std::{collections::HashMap, net::SocketAddr, sync::Arc};

use arc_swap::ArcSwap;
use matchit::Router;
use moka::sync::Cache;

use crate::config::models::Config;
use crate::server::models::{
    CacheKey, CachedResponse, Protocol, UpstreamServer,
};

//TODO: Add db connection in efficent way to use in metrics endpoints
pub struct AppState {
    pub routes: ArcSwap<HashMap<SocketAddr, Arc<UpstreamServer>>>,
    pub config: ArcSwap<Config>,
    pub router: ArcSwap<Router<String>>,
    pub cache: Cache<CacheKey, CachedResponse>,
}

impl AppState {
    pub fn new(config: Config) -> Self {
        Self {
            routes: ArcSwap::from_pointee(HashMap::new()),
            config: ArcSwap::from_pointee(config),
            router: ArcSwap::from_pointee(Router::new()),
            cache: Cache::new(1000),
        }
    }
}

pub fn init_server_state(state: Arc<AppState>) {
    let config = state.config.load();

    let mut new_routes_map = HashMap::new();

    for route_config in config.routes.values() {
        for server in &route_config.endpoints {
            let upstream = Arc::new(UpstreamServer::new(
                *server,
                //TODO: make dynamic from config
                100,
                Protocol::HTTP1,
            ));

            new_routes_map.insert(*server, upstream);
        }
    }

    state.routes.store(Arc::new(new_routes_map));
}
