use std::{
    collections::HashMap,
    net::{IpAddr, SocketAddr},
    sync::Arc,
};

use arc_swap::ArcSwap;
use dashmap::DashMap;
use jsonwebtoken::DecodingKey;
use matchit::Router;
use moka::sync::Cache;
use parking_lot::Mutex;
use rusqlite::Connection;

use crate::{
    config::{AuthType, CmdArgs, Config},
    db::db_utils,
    server::{
        connection,
        middleware::{
            cache::{CacheKey, CachedResponse, static_cache},
            rate_limit::TokenBucket,
            router,
        },
        upstream::UpstreamServer,
    },
};

/// Struct holding route related data that should be
/// updated at the same time
pub struct RoutingTable {
    pub router: Router<String>,
    pub routes: HashMap<SocketAddr, Arc<UpstreamServer>>,
}

//TODO: Add db connection in efficent way to use in metrics endpoints
//
//TODO: make sure reloading changes related things too if changed like
//      decoding_key. also some of the data here is probably duplicated
//      and saved in more than one place in memeory this should be reduced

/// Holds state of whole app passed to the reroute function
pub struct AppState {
    pub routing_table: ArcSwap<RoutingTable>,
    pub config: ArcSwap<Config>,
    pub cache: Cache<CacheKey, CachedResponse>,
    pub static_cache: DashMap<String, CachedResponse>,
    pub user_tokens: DashMap<IpAddr, TokenBucket>,
    pub decoding_key: ArcSwap<Option<DecodingKey>>,
    pub db_conn: Arc<Mutex<Connection>>,
}

impl AppState {
    pub fn new(config: Config, conn: Arc<Mutex<Connection>>) -> Self {
        Self {
            routing_table: ArcSwap::from_pointee(RoutingTable {
                router: Router::new(),
                routes: HashMap::new(),
            }),
            cache: Cache::new(config.cache.size),
            static_cache: DashMap::new(),
            user_tokens: DashMap::new(),
            decoding_key: match &config.auth.method {
                AuthType::JWT { secret } => ArcSwap::from_pointee(Some(
                    DecodingKey::from_secret(secret.as_ref()),
                )),
                _ => ArcSwap::from_pointee(None),
            },
            config: ArcSwap::from_pointee(config),
            db_conn: conn,
        }
    }
}

pub async fn init_server_state(state: Arc<AppState>, args: Arc<CmdArgs>) {
    let config = state.config.load();

    let mut new_routes_map = HashMap::new();

    for (route, route_config) in config.routes.iter() {
        let mut is_static_and_not_fetched = route_config.is_static;

        for server in &route_config.endpoints {
            let upstream = Arc::new(UpstreamServer::new(
                *server,
                route_config.max_connections,
                route_config.protocol,
                route_config.needs_auth,
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

            let ok = connection::health_ok(&upstream).await;

            if ok {
                new_routes_map.insert(*server, upstream);
            } else {
                tracing::error!(server = ?server, "Health not ok for server")
            }
        }
    }

    let new_key = match &config.auth.method {
        AuthType::JWT { secret } => {
            Some(DecodingKey::from_secret(secret.as_ref()))
        }
        _ => None,
    };

    state.decoding_key.store(Arc::new(new_key));

    let new_router = router::build_tree(state.clone());

    let new_table = RoutingTable { router: new_router, routes: new_routes_map };

    state.routing_table.store(Arc::new(new_table));

    if args.save {
        let conn_guard = state.db_conn.lock();
        if let Err(e) = db_utils::save_config(&conn_guard, &config) {
            tracing::error!(err = ?e, "Failed to save config");
        }
    }
}
